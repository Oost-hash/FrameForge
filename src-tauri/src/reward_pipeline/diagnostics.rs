//! JSONL attempt logging for the OCR retry loop: one event line per run, the
//! moment it happens — no prose, no tree glyphs, no summaries. The retry
//! backoff values (400/500/700 ms) and the 3-attempt full-catalog expansion
//! stay in the retry loop; this module records their outcomes.

use tauri::{Emitter, Manager};

use crate::app_state::AppState;
use crate::diagnostics::write_bmp;

use super::events::{self, AttemptEvent};
use super::policy::RewardPaths;

/// Append one attempt event to the session log (JSONL) and mirror the same
/// line to the last-result file, then surface the failure kind to the UI.
pub(crate) fn log_attempt(
    app: &tauri::AppHandle,
    paths: RewardPaths<'_>,
    event: &AttemptEvent,
    diag_dir: &std::sync::Arc<std::sync::Mutex<Option<std::path::PathBuf>>>,
) {
    let diagnostics_enabled = crate::diagnostics::ocr_pipeline_diagnostics_enabled();
    events::log(paths.session_log_path, event);
    if diagnostics_enabled {
        if let Ok(line) = serde_json::to_string(event) {
            let _ = std::fs::write(paths.last_path, format!("{line}\n"));
        }
    }

    let status = match event.result {
        "dark_frame" => Some("⬛ Dark frame (PrintWindow) — retrying".to_string()),
        "ocr_empty" => Some("⬜ OCR found no text — retrying".to_string()),
        "ocr_error" => Some("⚠️ OCR engine error — retrying".to_string()),
        "capture_failed" => Some("⚠️ Capture failed".to_string()),
        "no_match" => Some("❌ No catalog match, retrying...".to_string()),
        _ => None,
    };
    if let Some(status) = status {
        let _ = app.emit(crate::events::FF_STATUS, status);
    }

    // Keep the first failed OCR frame so its filename states exactly why it was
    // recorded; later desktop captures cannot overwrite it.
    if diagnostics_enabled && event.n == 1 {
        let Some(file_name) = (match event.result {
            "no_match" => Some("ocr_no_match_attempt_1.bmp"),
            "dark_frame" => Some("ocr_dark_frame_attempt_1.bmp"),
            "ocr_empty" => Some("ocr_empty_attempt_1.bmp"),
            "ocr_error" => Some("ocr_error_attempt_1.bmp"),
            _ => None,
        }) else {
            return;
        };
        let frame = app.state::<AppState>().last_ocr_frame.lock()
            .ok().and_then(|g| g.clone());
        let diag_snap = diag_dir.lock().ok().and_then(|g| g.clone());
        if let (Some((px, w, h)), Some(folder)) = (frame, diag_snap) {
            let _ = write_bmp(&folder.join(file_name), &px, w, h);
        }
    }
}

/// Advance the no-match streak; on the 3rd consecutive strike the catalog
/// expands to the full item list. Returns true when this tick expanded it, so
/// the caller can record `catalog_expanded` on the attempt event.
pub(crate) fn no_match_tick(
    streak: &mut u32,
    cat: &mut std::sync::Arc<Vec<(String, String)>>,
    fallback_cat: &std::sync::Arc<Vec<(String, String)>>,
) -> bool {
    *streak += 1;
    if *streak == 3 && cat.len() < fallback_cat.len() {
        *cat = std::sync::Arc::clone(fallback_cat);
        true
    } else {
        false
    }
}
