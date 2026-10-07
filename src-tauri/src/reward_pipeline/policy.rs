//! Reward-pipeline parameter groups, grouped so the reward-session helpers
//! stay within clippy's argument limit.

/// One OCR attempt's identity: attempt counter, timestamp, card reads, diagnostics.
pub(crate) struct RewardAttempt<'a> {
    pub(crate) attempt: u32,
    pub(crate) ts: &'a str,
    pub(crate) items: &'a [String],
    pub(crate) dbg: &'a str,
}

/// The session-log and last-result paths shared by the reward log helpers.
pub(crate) struct RewardPaths<'a> {
    pub(crate) session_log_path: &'a std::path::Path,
    pub(crate) last_path: &'a std::path::Path,
}
