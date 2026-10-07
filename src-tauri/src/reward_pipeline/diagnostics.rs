//! Session-log diagnostics for the OCR retry loop: dark-frame/empty-OCR/no-match
//! logging and the per-attempt best-result summary, preserving the 100/300/500/700ms
//! retry backoff values and the 3-attempt full-catalog expansion.

use tauri::{Emitter, Manager};

use crate::app_state::AppState;
use crate::diagnostics::write_bmp;
use crate::events;
use crate::append_to_file;

use super::policy::{RewardAttempt, RewardPaths};

/// Log that the OCR loop was stopped by an external dismiss signal.
pub(crate) fn log_reward_ocr_stopped(session_log_path: &std::path::Path) {
    let _ = append_to_file(
        session_log_path,
        "[STEP 2] OCR STOPPED — dismiss signal received\n\n",
    );
}

pub(crate) fn log_reward_dark_frame(
    app: &tauri::AppHandle,
    attempt: u32,
    ts: &str,
    dbg: &str,
    session_log_path: &std::path::Path,
    last_path: &std::path::Path,
) -> u64 {
    let entry = format!(
        "[STEP 2] OCR ATTEMPT #{}\n\
         ├─ Time     : {}\n\
         └─ RESULT   : {} → PrintWindow returned dark image\n\
            Check %TEMP%\\frameforge_capture_debug.bmp\n\
            Fix: switch Warframe to Borderless Windowed mode\n\
            Retrying in 100ms…\n\n",
        attempt, ts, dbg
    );
    let _ = append_to_file(session_log_path, &entry);
    let _ = std::fs::write(last_path, format!("=== {} ===\n{} — retrying\n", ts, dbg));
    let _ = app.emit(events::FF_STATUS, format!("⬛ {}", dbg));
    100
}

pub(crate) fn log_reward_ocr_empty(
    app: &tauri::AppHandle,
    attempt: u32,
    ts: &str,
    dbg: &str,
    session_log_path: &std::path::Path,
    last_path: &std::path::Path,
) -> u64 {
    let entry = format!(
        "[STEP 2] OCR ATTEMPT #{}\n\
         ├─ Time     : {}\n\
         └─ RESULT   : {} → image has content but OCR found no text\n\
            Check %TEMP%\\frameforge_capture_debug.bmp\n\
            Retrying in 300ms…\n\n",
        attempt, ts, dbg
    );
    let _ = append_to_file(session_log_path, &entry);
    let _ = std::fs::write(last_path, format!("=== {} ===\n{} — retrying\n", ts, dbg));
    let _ = app.emit(events::FF_STATUS, format!("⬜ {}", dbg));
    300
}

pub(crate) fn log_reward_capture_failed(
    app: &tauri::AppHandle,
    attempt: u32,
    ts: &str,
    session_log_path: &std::path::Path,
    last_path: &std::path::Path,
) -> u64 {
    let entry = format!(
        "[STEP 2] OCR ATTEMPT #{}\n\
         ├─ Time     : {}\n\
         └─ RESULT   : capture failed — Warframe window not found\n\
            Retrying in 500ms…\n\n",
        attempt, ts
    );
    let _ = append_to_file(session_log_path, &entry);
    let _ = std::fs::write(last_path, format!("=== {} ===\nCapture failed (window not found?)\n", ts));
    let _ = app.emit(events::FF_STATUS, "⚠️ Capture failed");
    500
}

pub(crate) fn log_reward_no_match(
    app: &tauri::AppHandle,
    attempt_info: RewardAttempt<'_>,
    no_match_streak: &mut u32,
    cat: &mut std::sync::Arc<Vec<(String, String)>>,
    fallback_cat: &std::sync::Arc<Vec<(String, String)>>,
    paths: RewardPaths<'_>,
    diag_dir: &std::sync::Arc<std::sync::Mutex<Option<std::path::PathBuf>>>,
) -> u64 {
    let RewardAttempt { attempt, ts, items, dbg } = attempt_info;
    let RewardPaths { session_log_path, last_path } = paths;
    *no_match_streak += 1;
    let expanded = if *no_match_streak == 3 && cat.len() < fallback_cat.len() {
        *cat = std::sync::Arc::clone(fallback_cat);
        true
    } else {
        false
    };
    let cur_cat_len = cat.len();
    let expand_note = if expanded {
        format!(" [expanded to full catalog: {}]", cur_cat_len)
    } else {
        String::new()
    };
    let entry = format!(
        "[STEP 2] OCR ATTEMPT #{}\n\
         ├─ Time     : {}\n\
         {}\n\
         └─ RESULT   : no catalog match (catalog={}){}\u{2192} retrying in 700ms\n\n",
        attempt, ts, dbg, cur_cat_len, expand_note
    );
    let _ = append_to_file(session_log_path, &entry);
    let _ = std::fs::write(
        last_path,
        format!("=== {} ===\nno match (catalog={}): {:?}\n{}\n", ts, cur_cat_len, items, dbg),
    );
    let _ = app.emit(events::FF_STATUS, "❌ No catalog match, retrying...");
    if attempt == 1 {
        let frame = app.state::<AppState>().last_ocr_frame.lock()
            .ok().and_then(|g| g.clone());
        let diag_snap = diag_dir.lock().ok().and_then(|g| g.clone());
        if let (Some((px, w, h)), Some(folder)) = (frame, diag_snap) {
            let _ = write_bmp(&folder.join("screenshot.bmp"), &px, w, h);
        }
    }
    700
}

pub(crate) fn log_reward_best_result(
    attempt_info: RewardAttempt<'_>,
    complete: bool,
    confirm_ready: bool,
    paths: RewardPaths<'_>,
) {
    let RewardAttempt { attempt, ts, items, dbg } = attempt_info;
    let RewardPaths { session_log_path, last_path } = paths;
    let label = if complete && confirm_ready { "✅" } else { "⚡" };
    let status_label = if complete && confirm_ready {
        "locked"
    } else if complete {
        "soft-complete, waiting for EE hint"
    } else {
        "waiting"
    };
    let _ = crate::append_to_file(
        session_log_path,
        &format!("{} {} items ({})", label, items.len(), status_label),
    );
    let result_label = if complete && confirm_ready {
        "LOCKED & emitting"
    } else if complete {
        "soft-complete, retrying (waiting for EE hint)"
    } else {
        "saved, retrying"
    };
    let session_entry = format!(
        "[STEP 2] OCR ATTEMPT #{}\n\
         ├─ Time     : {}\n\
         {}\n\
         └─ RESULT   : {} items found \u{2192} {}\n\
         \u{2514}\u{2500} Items    : {:?}\n\n",
        attempt, ts, dbg, items.len(), result_label, items,
    );
    let _ = append_to_file(session_log_path, &session_entry);
    let _ = std::fs::write(last_path, format!("=== {} ===\nItems: {:?}\n{}\n", ts, items, dbg));
}

pub(crate) fn log_reward_confirm_no_improvement(
    attempt: u32,
    ts: &str,
    items: &[String],
    session_log_path: &std::path::Path,
) {
    let _ = crate::append_to_file(
        session_log_path,
        &format!(
            "[STEP 2] OCR ATTEMPT #{} (confirm)\n\
             \u{251c}\u{2500} Time     : {}\n\
             \u{2514}\u{2500} {} items \u{2014} same as before, confirmed\n\n",
            attempt, ts, items.len()
        ),
    );
}
