//! EE.log trigger handling: session relic collection, the VoidProjections
//! reward handshake, and catalog prefiltering for a new reward session.

use tauri::Manager;
use tracing::warn;

use crate::app_state::AppState;
use crate::append_to_file;

/// The trigger line that opened a reward session plus its catalog prefilter summary.
pub(crate) struct RewardTrigger<'a> {
    pub(crate) timestamp: &'a str,
    pub(crate) trigger_line: &'a str,
    pub(crate) prefilter_log: &'a str,
    pub(crate) catalog_len: usize,
}

pub(crate) fn collect_session_relics(text: &str, session_relics: &mut Vec<String>) {
    for line in text.lines() {
        if line.contains("Resource load completed")
            && line.contains("/Lotus/Types/Game/Projections/")
        {
            if let Some(paren) = line.find("(/Lotus/Types/Game/Projections/") {
                let path = line[paren + 1..].split(')').next().unwrap_or("").trim();
                if !path.is_empty() && !session_relics.iter().any(|relic| relic == path) {
                    session_relics.push(path.to_string());
                }
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct VoidProjectionState {
    in_sequence: bool,
    sequence_completed: bool,
    other_ids: std::collections::HashSet<String>,
    own_item: String,
}

impl VoidProjectionState {
    pub(crate) fn consume_sequence_completed(&mut self) -> bool {
        std::mem::take(&mut self.sequence_completed)
    }

    pub(crate) fn take_own_item(&mut self) -> Option<String> {
        if self.own_item.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.own_item))
        }
    }
}

/// Update the VoidProjections reward-handshake state from newly appended EE.log lines.
pub(crate) fn collect_void_projection_state(
    text: &str,
    state: &mut VoidProjectionState,
    squad_size: &std::sync::Arc<std::sync::Mutex<Option<usize>>>,
    session_log_path: &std::path::Path,
) {
    for line in text.lines() {
        let lower = line.to_lowercase();
        if lower.contains("voidprojections: getvoidprojectionreward") {
            state.in_sequence = true;
            state.other_ids.clear();
            state.own_item.clear();
            if let Ok(mut size) = squad_size.lock() {
                *size = None;
            }
        }
        if lower.contains("gets reward /lotus/") {
            if let Some(index) = line.find("/Lotus/") {
                state.own_item = line[index..].trim().to_string();
            }
        }
        if state.in_sequence {
            if lower.contains("still waiting on response from") {
                if let Some(id) = lower.split_whitespace().last() {
                    state.other_ids.insert(id.to_string());
                }
            } else if lower.contains("has reward info for all players now") {
                let squad = (1 + state.other_ids.len()).clamp(1, 4);
                if let Ok(mut size) = squad_size.lock() {
                    *size = Some(squad);
                }
                state.in_sequence = false;
                state.sequence_completed = true;
                let _ = append_to_file(
                    session_log_path,
                    &format!(
                        "[EE.log] VoidProjections squad\n\\
                         ├─ Local item : {}\n\\
                         ├─ Other players (unique IDs) : {}\n\\
                         └─ Squad size : {} total\n\n",
                        if state.own_item.is_empty() { "(not found)" } else { &state.own_item },
                        state.other_ids.len(),
                        squad,
                    ),
                );
            }
        }
    }
}

/// Narrow the OCR catalog to rewards from relics seen in the current session.
pub(crate) fn filter_relic_reward_catalog(
    app: &tauri::AppHandle,
    session_relics: &[String],
    full_catalog: &std::sync::Arc<Vec<(String, String)>>,
) -> (std::sync::Arc<Vec<(String, String)>>, String) {
    if session_relics.is_empty() {
        return (
            std::sync::Arc::clone(full_catalog),
            "  No relics collected — using full catalog (FrameForge started mid-mission?)".to_string(),
        );
    }
    let mut rewards: Vec<(String, String)> = {
        let state = app.state::<AppState>();
        let reward_map = state.relic_rewards.lock().unwrap_or_else(|e| e.into_inner());
        session_relics
            .iter()
            .filter_map(|path| reward_map.get(path.as_str()))
            .flat_map(|rewards| rewards.iter().map(|reward| (reward.unique_name.clone(), reward.name.clone())))
            .filter(|(_, name)| !name.is_empty())
            .collect()
    };
    if rewards.is_empty() {
        return (
            std::sync::Arc::clone(full_catalog),
            format!(
                "  {} relic path(s) found but none matched relic_rewards — using full catalog\n  Paths: {:?}",
                session_relics.len(), session_relics
            ),
        );
    }
    rewards.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    rewards.dedup_by(|a, b| !a.0.is_empty() && a.0 == b.0);
    let names: Vec<&str> = rewards.iter().map(|(_, name)| name.as_str()).collect();
    let sample = &names[..names.len().min(8)];
    let log = format!(
        "  {} relic(s) → {} rewards (direct from Relics.json)\n  Relics: {:?}\n  Rewards: {:?}",
        session_relics.len(), rewards.len(), session_relics, sample
    );
    (std::sync::Arc::new(rewards), log)
}

/// Build a fallback OCR catalog when the initial cache was empty at monitor startup.
pub(crate) fn build_fallback_reward_catalog(
    app: &tauri::AppHandle,
) -> Option<std::sync::Arc<Vec<(String, String)>>> {
    let state = app.state::<AppState>();
    let items = state.wfcd_items.lock().unwrap_or_else(|e| e.into_inner());
    if items.is_empty() {
        return None;
    }
    let blueprints = state.blueprint_to_result.lock().unwrap_or_else(|e| e.into_inner());
    let excluded = [
        "Warframes", "Primary", "Secondary", "Melee", "Companion", "Sentinels", "Archwing",
        "Arch-Gun", "Arch-Melee", "Pets", "Robotic",
    ];
    let mut catalog: Vec<(String, String)> = items
        .iter()
        .filter(|item| {
            let name = item.name.to_lowercase();
            !excluded.contains(&item.category.as_str())
                && !name.ends_with("intact")
                && !name.ends_with("exceptional")
                && !name.ends_with("flawless")
                && !name.ends_with("radiant")
                && (name.contains("prime") || name.starts_with("forma"))
        })
        .map(|item| (item.unique_name.clone(), item.name.clone()))
        .collect();
    for (path, (name, _)) in blueprints.iter() {
        let lower = name.to_lowercase();
        if lower.contains("prime") || lower.starts_with("forma") {
            catalog.push((path.clone(), name.clone()));
        }
    }
    catalog.sort_by(|a, b| a.0.cmp(&b.0));
    catalog.dedup_by(|a, b| a.0 == b.0);
    (!catalog.is_empty()).then(|| std::sync::Arc::new(catalog))
}

/// Start the diagnostic files for a relic-reward OCR session.
pub(crate) fn prepare_reward_session(
    session_log_path: &std::path::Path,
    squad_names: &std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    trigger: RewardTrigger<'_>,
    auto_capture_dir: &std::path::Path,
    diag_dir: &std::sync::Arc<std::sync::Mutex<Option<std::path::PathBuf>>>,
    last_found_path: &std::path::Path,
) {
    let RewardTrigger {
        timestamp,
        trigger_line,
        prefilter_log,
        catalog_len,
    } = trigger;
    let names = squad_names.lock().map(|names| names.clone()).unwrap_or_default();
    let known_names = if names.is_empty() {
        "  (none — names not yet seen in EE.log)".to_string()
    } else {
        names.iter().map(|name| format!("  • {name}")).collect::<Vec<_>>().join("\n")
    };
    if let Err(error) = std::fs::write(
        session_log_path,
        format!(
            "══════════════════════════════════════════════\n\\
             RELIC OVERLAY SESSION — {}\n\\
             ═════════════════════════════════════════════\n\\
             Log path  : {}\n\n\\
             [KNOWN PLAYERS — OCR username filter]\n\\
             {}\n\n\\
             [STEP 1] EE.log TRIGGER\n\\
             ├─ Time     : {}\n\\
             ├─ Line     : \"{}\"\n\\
             ├─ Prefilter: {}\n\\
             └─ Catalog  : {} items\n\n",
            timestamp,
            session_log_path.display(),
            known_names,
            timestamp,
            trigger_line,
            prefilter_log,
            catalog_len,
        ),
    ) {
        warn!(error = %error, "session log write failed");
    }
    let run_dir = auto_capture_dir.join(chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string());
    let _ = std::fs::create_dir_all(&run_dir);
    if let Ok(mut guard) = diag_dir.lock() {
        *guard = Some(run_dir);
    }
    let _ = std::fs::write(
        last_found_path,
        format!("=== {} ===\nEE.log trigger fired\n{}\n", timestamp, trigger_line),
    );
}

/// Prepare OCR hints immediately when a relic reward screen is triggered.
pub(crate) fn prepare_reward_trigger(
    app: &tauri::AppHandle,
    squad_names: &std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    squad_size: &std::sync::Arc<std::sync::Mutex<Option<usize>>>,
    session_relics: &[String],
) {
    if let Ok(local_player) = app.state::<AppState>().local_player_name.lock() {
        if let Some(name) = local_player.as_ref() {
            if let Ok(mut names) = squad_names.lock() {
                if !names.iter().any(|existing| existing == name) {
                    names.push(name.clone());
                }
            }
        }
    }
    let relic_hint = session_relics.len().min(4);
    if relic_hint >= 1 {
        if let Ok(mut hint) = squad_size.lock() {
            if hint.is_none() {
                *hint = Some(relic_hint);
            }
        }
    }
}
