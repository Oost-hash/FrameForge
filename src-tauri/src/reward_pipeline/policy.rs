//! Reward-pipeline parameter groups, grouped so the reward-session helpers
//! stay within clippy's argument limit.

/// The session-log and last-result paths shared by the reward log helpers.
pub(crate) struct RewardPaths<'a> {
    pub(crate) session_log_path: &'a std::path::Path,
    pub(crate) last_path: &'a std::path::Path,
}
