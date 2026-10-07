//! Relic-reward OCR pipeline: EE.log trigger → capture → OCR recognition →
//! catalog matching → overlay publish. Split into one module per stage; see
//! each submodule's doc comment for its slice of the pipeline.

mod capture;
mod diagnostics;
mod matching;
mod policy;
mod publish;
mod recognize;
mod trigger;

pub(crate) use capture::{capture_reward_items, schedule_reward_diagnostic_capture};
pub(crate) use diagnostics::{
    log_reward_best_result, log_reward_capture_failed, log_reward_confirm_no_improvement,
    log_reward_dark_frame, log_reward_no_match, log_reward_ocr_empty, log_reward_ocr_stopped,
};
pub(crate) use policy::{RewardAttempt, RewardPaths};
pub(crate) use publish::{
    auto_dismiss_relic_rewards, close_reward_overlay, dismiss_relic_rewards,
    finalize_reward_ocr_timeout, publish_relic_rewards, schedule_reward_safety_cleanup,
    DismissState,
};
pub(crate) use recognize::extract_reward_items_twophase;
pub(crate) use trigger::{
    build_fallback_reward_catalog, collect_session_relics, collect_void_projection_state,
    filter_relic_reward_catalog, prepare_reward_session, prepare_reward_trigger,
    RewardTrigger, VoidProjectionState,
};

pub struct OcrParams<'a> {
    pixels: &'a [u8],
    pix_w: u32,
    pix_h: u32,
    game_h: u32,
    catalog: &'a [(String, String)],
    capture_info: &'a str,
    hint_squad_size: Option<usize>,
    player_names: &'a [String],
    /// Run the text-recognition pass on a grayscale+contrast-stretched copy of
    /// `pixels` instead of the raw frame. Only the OCR text engine's input is
    /// affected — rarity-bar and icon-colour classification always read the
    /// original `pixels`, since those rely on hue (orange/teal/gold), which
    /// preprocessing destroys. Used for the reuse-last-frame low-confidence
    /// fallback in `reward_pipeline::capture_reward_items`.
    preprocess_text_pass: bool,
}
