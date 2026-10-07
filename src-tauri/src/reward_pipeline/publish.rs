//! Reward overlay lifecycle: inventory application, dismissal, and the
//! auto-dismiss / safety-cleanup timers.

use std::sync::atomic::Ordering;
use tauri::{Emitter, Manager};

use crate::app_state::AppState;
use crate::diagnostics::append_to_diag;
use crate::events;
use crate::append_to_file;

/// Mutable reward-screen watch state owned by the EE.log reader loop.
pub(crate) struct DismissState<'a> {
    pub(crate) active_since: &'a mut Option<std::time::Instant>,
    pub(crate) last_dismiss_at: &'a mut Option<std::time::Instant>,
    pub(crate) session_relics: &'a mut Vec<String>,
    pub(crate) projection_state: &'a mut super::VoidProjectionState,
}

/// Apply the local player's EE.log reward before the next memory scan completes.
pub(crate) fn apply_reward_inventory_update(
    app: &tauri::AppHandle,
    store_path: String,
    session_log_path: &std::path::Path,
) {
    let inv_path = crate::worldstate::store_to_unique(&store_path);
    let state: tauri::State<AppState> = app.state();
    let (old_qty, new_qty) = {
        let mut quantities = state.current_quantities.lock().unwrap_or_else(|e| e.into_inner());
        let old = *quantities.get(&inv_path).unwrap_or(&0);
        let new = old + 1;
        quantities.insert(inv_path.clone(), new);
        (old, new)
    };
    let item_name = inv_path.split('/').next_back().unwrap_or("?").to_string();
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&state.changes_log_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "[{}] EE.log Reward | {} | {} → {} (gets reward)",
            timestamp, item_name, old_qty, new_qty
        );
    }
    let _ = app.emit(events::INVENTORY_REWARD, serde_json::json!({ "path": inv_path, "qty": new_qty }));
    append_to_diag(
        session_log_path,
        &format!(
            "[REWARD] Inventory updated from EE.log\n\\
             ├─ Store path : {}\n\\
             ├─ Inv path   : {}\n\\
             └─ Qty        : {} → {}\n\n",
            store_path, inv_path, old_qty, new_qty
        ),
    );
}

/// Handle an EE.log reward-screen dismissal and schedule the overlay cleanup.
pub(crate) fn dismiss_relic_rewards(
    app: &tauri::AppHandle,
    text: &str,
    session_log_path: &std::path::Path,
    diag_dir: &std::sync::Arc<std::sync::Mutex<Option<std::path::PathBuf>>>,
    reward_screen_active: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    rewards_emitted_ms: &std::sync::Arc<std::sync::atomic::AtomicU64>,
    state: DismissState<'_>,
) -> bool {
    let DismissState {
        active_since,
        last_dismiss_at,
        session_relics,
        projection_state,
    } = state;
    let lower = text.to_lowercase();
    let is_dismiss = lower.contains("relic reward screen shut down")
        || lower.contains("closevoidprojectionrewardscreen")
        || lower.contains("matchingservice::endsession");
    if !is_dismiss {
        return false;
    }

    let dismiss_line = text.lines()
        .find(|line| {
            let line = line.to_lowercase();
            line.contains("relic reward screen shut down")
                || line.contains("closevoidprojectionrewardscreen")
                || line.contains("matchingservice::endsession")
        })
        .unwrap_or("<unknown dismiss line>")
        .trim();
    let elapsed = active_since.map(|time| time.elapsed().as_secs_f64());
    append_to_diag(
        session_log_path,
        &format!(
            "[STEP 4] DISMISS\n\\
             ├─ Time     : {}\n\\
             ├─ Line     : \"{}\"\n\\
             └─ Open for : {}\n\n",
            chrono::Local::now().format("%H:%M:%S%.3f"),
            dismiss_line,
            elapsed.map(|seconds| format!("{seconds:.1}s")).unwrap_or_else(|| "(unknown)".to_string()),
        ),
    );
    if let Ok(mut guard) = diag_dir.lock() {
        if let Some(folder) = guard.take() {
            let _ = std::fs::copy(session_log_path, folder.join("ocr_session_log.txt"));
        }
    }
    reward_screen_active.store(false, Ordering::SeqCst);
    *active_since = None;
    *last_dismiss_at = Some(std::time::Instant::now());
    if lower.contains("matchingservice::endsession") {
        session_relics.clear();
    }
    if let Some(store_path) = projection_state.take_own_item() {
        apply_reward_inventory_update(app, store_path, session_log_path);
    }

    const MIN_DISPLAY_MS: u64 = 5_000;
    let emitted_at = rewards_emitted_ms.load(Ordering::SeqCst);
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0);
    let delay_ms = if emitted_at > 0 {
        MIN_DISPLAY_MS.saturating_sub(now_ms.saturating_sub(emitted_at))
    } else {
        0
    };
    rewards_emitted_ms.store(0, Ordering::SeqCst);

    let dismiss_app = app.clone();
    std::thread::spawn(move || {
        if delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
        if let Some(window) = dismiss_app.get_webview_window("relic-overlay") {
            let _ = window.set_position(tauri::Position::Physical(
                tauri::PhysicalPosition { x: 0, y: -3000 },
            ));
        }
        if let Ok(mut rewards) = dismiss_app.state::<AppState>().pending_relic_rewards.lock() {
            *rewards = None;
        }
        let _ = dismiss_app.emit(events::RELIC_REWARDS, serde_json::Value::Null);
    });
    true
}

/// Dismiss a reward overlay that remained active beyond its maximum window.
pub(crate) fn auto_dismiss_relic_rewards(
    app: &tauri::AppHandle,
    session_log_path: &std::path::Path,
    diag_dir: &std::sync::Arc<std::sync::Mutex<Option<std::path::PathBuf>>>,
    reward_screen_active: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    active_since: &mut Option<std::time::Instant>,
    last_dismiss_at: &mut Option<std::time::Instant>,
    rewards_emitted_ms: &std::sync::Arc<std::sync::atomic::AtomicU64>,
) {
    let Some(since) = *active_since else { return };
    if since.elapsed().as_secs() < 20 {
        return;
    }
    close_reward_overlay(
        app,
        session_log_path,
        diag_dir,
        reward_screen_active,
        active_since,
        last_dismiss_at,
        rewards_emitted_ms,
        "AUTO-DISMISS (20s timeout)",
    );
}

/// Hide the reward overlay now and clear its state. Used by the 20 s timeout and
/// when the relic-pick grid opens (the reward screen has closed by then, even if
/// its "shut down" line hasn't reached EE.log yet). Skips the 5 s minimum display.
pub(crate) fn close_reward_overlay(
    app: &tauri::AppHandle,
    session_log_path: &std::path::Path,
    diag_dir: &std::sync::Arc<std::sync::Mutex<Option<std::path::PathBuf>>>,
    reward_screen_active: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    active_since: &mut Option<std::time::Instant>,
    last_dismiss_at: &mut Option<std::time::Instant>,
    rewards_emitted_ms: &std::sync::Arc<std::sync::atomic::AtomicU64>,
    reason: &str,
) {
    let open_for = active_since.map(|since| since.elapsed().as_secs_f64());
    append_to_diag(
        session_log_path,
        &format!(
            "[STEP 4] {}\n├─ Time     : {}\n└─ Open for : {}\n\n",
            reason,
            chrono::Local::now().format("%H:%M:%S%.3f"),
            open_for.map(|seconds| format!("{seconds:.1}s")).unwrap_or_else(|| "(unknown)".to_string()),
        ),
    );
    if let Ok(mut guard) = diag_dir.lock() {
        if let Some(folder) = guard.take() {
            let _ = std::fs::copy(session_log_path, folder.join("ocr_session_log.txt"));
        }
    }
    reward_screen_active.store(false, Ordering::SeqCst);
    *active_since = None;
    *last_dismiss_at = Some(std::time::Instant::now());
    rewards_emitted_ms.store(0, Ordering::SeqCst);
    if let Some(window) = app.get_webview_window("relic-overlay") {
        let _ = window.set_position(tauri::Position::Physical(
            tauri::PhysicalPosition { x: 0, y: -3000 },
        ));
    }
    if let Ok(mut rewards) = app.state::<AppState>().pending_relic_rewards.lock() {
        *rewards = None;
    }
    let _ = app.emit(events::RELIC_REWARDS, serde_json::Value::Null);
}

/// Schedule the final cleanup when the normal EE.log dismissal never arrives.
pub(crate) fn schedule_reward_safety_cleanup(
    app: tauri::AppHandle,
    session_log_path: std::path::PathBuf,
    diag_dir: std::sync::Arc<std::sync::Mutex<Option<std::path::PathBuf>>>,
    also_write_diagnostics: bool,
) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        if let Ok(mut rewards) = app.state::<AppState>().pending_relic_rewards.lock() {
            *rewards = None;
        }
        let _ = app.emit(events::RELIC_REWARDS, serde_json::Value::Null);
        if let Some(window) = app.get_webview_window("relic-overlay") {
            let _ = window.set_position(tauri::Position::Physical(
                tauri::PhysicalPosition { x: 0, y: -3000 },
            ));
        }
        if also_write_diagnostics {
            append_to_diag(&session_log_path, "[STEP 4] AUTO-DISMISS (20s safety fallback)\n\n");
        } else {
            let _ = append_to_file(&session_log_path, "[STEP 4] AUTO-DISMISS (20s safety fallback)\n\n");
        }
        if let Ok(mut guard) = diag_dir.lock() {
            if let Some(folder) = guard.take() {
                let _ = std::fs::copy(&session_log_path, folder.join("ocr_session_log.txt"));
            }
        }
    });
}

/// Store the payload for a late overlay mount, then notify the frontend.
pub(crate) fn publish_relic_rewards(
    app: &tauri::AppHandle,
    payload: Option<&serde_json::Value>,
) {
    if payload.is_some_and(|payload| !payload.is_null())
        && !app.state::<AppState>().overlays_enabled.load(Ordering::SeqCst) {
        return;
    }
    if let Some(payload) = payload.filter(|payload| !payload.is_null()) {
        if let Ok(mut pending) = app.state::<AppState>().pending_relic_rewards.lock() {
            *pending = Some(payload.clone());
        }
    }
    let _ = app.emit(events::RELIC_REWARDS, payload);
}

/// Emit the best partial result after OCR timed out, hide the overlay, and
/// persist the session log to the diagnostic folder.
///
/// `session_valid` is false when a newer trigger has already taken over the
/// shared `reward_screen_active` flag by the time this stale task's 45 s
/// deadline fires (e.g. a fast dismiss-then-retrigger). In that case every
/// action below is skipped — touching the active flag, overlay position or
/// pending payload here would stomp on the newer session instead of just
/// cleaning up this one.
pub(crate) fn finalize_reward_ocr_timeout(
    app: &tauri::AppHandle,
    best_payload: Option<serde_json::Value>,
    active: &std::sync::atomic::AtomicBool,
    session_log_path: &std::path::Path,
    diag_dir: &std::sync::Arc<std::sync::Mutex<Option<std::path::PathBuf>>>,
    session_valid: bool,
) {
    if !session_valid {
        let _ = append_to_file(
            session_log_path,
            "[STEP 2] OCR TIMEOUT — stale session, dropping result (newer trigger took over)\n\n",
        );
        return;
    }
    let emit_val = if active.load(Ordering::SeqCst) {
        best_payload.unwrap_or(serde_json::Value::Null)
    } else {
        serde_json::Value::Null
    };
    publish_relic_rewards(app, Some(&emit_val));
    let _ = append_to_file(
        session_log_path,
        "[STEP 2] OCR TIMEOUT — 45 seconds elapsed, emitting best result\n\n",
    );
    if let Some(win) = app.get_webview_window("relic-overlay") {
        let _ = win.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: 0,
            y: -3000,
        }));
    }
    active.store(false, Ordering::SeqCst);
    if let Ok(mut g) = diag_dir.lock() {
        if let Some(folder) = g.take() {
            let _ = std::fs::copy(session_log_path, folder.join("ocr_session_log.txt"));
        }
    }
}
