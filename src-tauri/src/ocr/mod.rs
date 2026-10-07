//! Screen capture + OCR for Warframe relic reward detection.
//!
//! This module separates platform-specific capture/OCR code (`windows.rs`, `linux.rs`)
//! from platform-agnostic domain logic (preprocessing, BMP encoding, word matching,
//! catalog matching, and reward item extraction).

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

// ─── Re-exports from platform modules ────────────────────────────────────────

#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "linux")]
pub use linux::*;

// ─── Domain logic (platform-agnostic) ────────────────────────────────────────

/// Grayscale + contrast stretch on BGRA pixels.
/// Converting to grayscale is the key step: element icons (Cold, Heat, Toxin)
/// are colored glyphs — in the original BGRA image WinRT OCR rejects these lines as
/// graphics. After grayscale they become neutral-brightness shapes, so OCR reads the
/// white text on either side of the icon instead of dropping the whole line.
pub fn preprocess_for_ocr(pixels: &[u8], width: u32, height: u32) -> (Vec<u8>, u32, u32) {
    let mut out = pixels.to_vec();
    for px in out.chunks_mut(4) {
        // Standard luminance: 0.299 R + 0.587 G + 0.114 B (BGRA order)
        let gray = ((px[2] as u32 * 299 + px[1] as u32 * 587 + px[0] as u32 * 114) / 1000)
            .min(255) as u8;
        // Mild contrast stretch [20, 235] → [0, 255]
        let v = ((gray as i32 - 20) * 255 / 215).clamp(0, 255) as u8;
        px[0] = v;
        px[1] = v;
        px[2] = v;
    }
    (out, width, height)
}

/// Encode BGRA pixels as a 24-bit BGR BMP (no alpha — BitmapDecoder handles it fine).
pub fn to_bmp(pixels_bgra: &[u8], width: u32, height: u32) -> Vec<u8> {
    let row_bytes = width * 3;
    let padding   = (4 - row_bytes % 4) % 4;
    let row_stride = row_bytes + padding;
    let image_size = row_stride * height;
    let file_size  = 54 + image_size;

    let mut bmp = Vec::with_capacity(file_size as usize);
    // File header
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&file_size.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&54u32.to_le_bytes());
    // Info header
    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&(width as i32).to_le_bytes());
    bmp.extend_from_slice(&(-(height as i32)).to_le_bytes()); // top-down
    bmp.extend_from_slice(&1u16.to_le_bytes());
    bmp.extend_from_slice(&24u16.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    bmp.extend_from_slice(&image_size.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    // Pixel rows (BGRA → BGR + padding)
    for row in 0..height {
        for col in 0..width {
            let i = ((row * width + col) * 4) as usize;
            bmp.push(pixels_bgra[i]);
            bmp.push(pixels_bgra[i + 1]);
            bmp.push(pixels_bgra[i + 2]);
        }
        bmp.extend(std::iter::repeat_n(0, padding as usize));
    }
    bmp
}

// Rarity-bar detection, icon classification, catalog matching, and the
// reward-extraction entry point now live under `reward_pipeline::recognize`
// and `reward_pipeline::matching` — this module stays pure capture/BMP code.
