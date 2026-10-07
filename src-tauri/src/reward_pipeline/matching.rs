//! Pure word/fuzzy-matching helpers used by [`super::recognize`] to score OCR
//! text against the item catalog. No capture, no catalog I/O — just string math.

pub(super) fn lev_dist(a: &str, b: &str) -> usize {
    let a = a.as_bytes();
    let b = b.as_bytes();
    let (m, n) = (a.len(), b.len());
    if m.abs_diff(n) > 3 { return 99; }
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr = vec![0usize; n + 1];
    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            curr[j] = if a[i-1] == b[j-1] { prev[j-1] }
                      else { 1 + prev[j].min(curr[j-1]).min(prev[j-1]) };
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[n]
}

/// Check whether `catalog_word` appears in `ocr_words` via:
///   1. Exact match
///   2. Prefix match: OCR truncated ("prime"→"pri", "voruna"→"vor")
///   3. Suffix substring: "neuroptics" → OCR gives "rüroptics"/"tearoptics" which
///      both contain "optics" — the distinctive suffix is preserved even when the
///      prefix is garbled. Check last 5+ chars as a substring in any OCR word.
///   4. Levenshtein ≤ 1 (or ≤ 2 for ≥8-char words) for single-char typos
///   5. Sliding-window inside longer merged tokens ("Sevagotfirime")
pub(super) fn word_found_in_set(
    catalog_word: &str,
    ocr_words: &std::collections::HashSet<String>,
) -> bool {
    if ocr_words.contains(catalog_word) { return true; }
    if catalog_word.len() < 4 { return false; }

    // Prefix: OCR word is the leading portion of the catalog word
    for ocr_w in ocr_words {
        if ocr_w.len() >= 3 && catalog_word.starts_with(ocr_w.as_str()) { return true; }
    }

    // Suffix substring: check if last N chars of catalog word appear inside any OCR word
    if catalog_word.len() >= 6 {
        let suffix_len = (catalog_word.len() / 2).max(5);
        let suffix = &catalog_word[catalog_word.len() - suffix_len..];
        if ocr_words.iter().any(|w| w.find(suffix).is_some_and(|p| p != 1)) { return true; }
    }

    // Edit budget by word length
    let max_dist = if catalog_word.len() >= 8 {
        2
    } else if catalog_word.len() >= 5 {
        1
    } else {
        0
    };
    let wb = catalog_word.as_bytes();
    for ocr_w in ocr_words {
        if ocr_w.len() >= 4 {
            let dist = lev_dist(catalog_word, ocr_w);
            let len_diff = (catalog_word.len() as isize - ocr_w.len() as isize).unsigned_abs();
            if dist <= max_dist && !(len_diff == dist && len_diff >= 2) { return true; }
        }
        let ob = ocr_w.as_bytes();
        if ob.len() >= wb.len() + 4 {
            for (win_start, win) in ob.windows(wb.len()).enumerate() {
                let errs = wb.iter().zip(win.iter()).filter(|(a, b)| a != b).count();
                if errs == 0 && win_start + wb.len() == ob.len() && win_start >= 3 { continue; }
                if errs <= max_dist { return true; }
            }
        }
    }
    false
}

/// Normalise OCR text for catalog matching.
pub(super) fn normalise(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii() { return c.to_ascii_lowercase(); }
            match c {
                'À'|'Á'|'Â'|'Ã'|'Ä'|'Å'|'à'|'á'|'â'|'ã'|'ä'|'å' => 'a',
                'È'|'É'|'Ê'|'Ë'|'è'|'é'|'ê'|'ë' => 'e',
                'Ì'|'Í'|'Î'|'Ï'|'ì'|'í'|'î'|'ï' => 'i',
                'Ò'|'Ó'|'Ô'|'Õ'|'Ö'|'ò'|'ó'|'ô'|'õ'|'ö' => 'o',
                'Ù'|'Ú'|'Û'|'Ü'|'ù'|'ú'|'û'|'ü' => 'u',
                'Ñ'|'ñ' => 'n',
                'Ç'|'ç' => 'c',
                'Ý'|'ý'|'ÿ' => 'y',
                _ => ' ',
            }
        })
        .collect()
}

pub(super) fn extract_item_name_words(words: &std::collections::HashSet<String>) -> Vec<String> {
    const SKIP: &[&str] = &[
        "prime", "blueprint", "owned", "crafted", "bl", "neuroptics", "systems",
        "chassis", "barrel", "stock", "receiver", "handle", "blade", "grip",
        "limb", "upper", "lower", "string", "link", "carapace", "cerebrum",
        "forma", "riven", "sliver", "ayatan",
    ];
    words.iter()
        .filter(|w| w.len() >= 3 && !SKIP.contains(&w.as_str()))
        .cloned()
        .collect()
}

pub(super) fn bar_centers_are_valid(centers: &[f32]) -> bool {
    let n = centers.len();
    if n == 0 { return false; }
    if centers[0] < 0.15 || centers[n - 1] > 0.90 { return false; }
    if n < 2 { return true; }
    for pair in centers.windows(2) {
        if pair[1] - pair[0] < 0.08 { return false; }
    }
    if n >= 3 {
        let gaps: Vec<f32> = centers.windows(2).map(|p| p[1] - p[0]).collect();
        let mean = gaps.iter().sum::<f32>() / gaps.len() as f32;
        if gaps.iter().any(|g| (g - mean).abs() > 0.04) { return false; }
    }
    let span = centers[n - 1] - centers[0];
    let expected = match n {
        2 => 0.34f32,
        3 => 0.46,
        _ => 0.52,
    };
    (span - expected).abs() < 0.10
}

pub(super) fn hardcoded_card_centers(n: usize) -> Vec<f32> {
    match n {
        1 => vec![0.50],
        2 => vec![0.435, 0.565],
        3 => vec![0.37, 0.50, 0.63],
        _ => vec![0.31, 0.44, 0.56, 0.69],
    }
}

/// Groups x-positions into clusters: a value joins the running cluster while it
/// stays within `gap` of that cluster's mean. Returns each cluster's mean, left to right.
pub(super) fn cluster_x_centers(mut xs: Vec<f32>, gap: f32) -> Vec<f32> {
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut clusters: Vec<(f32, usize)> = Vec::new(); // (sum, count)
    for x in xs {
        match clusters.last_mut() {
            Some((sum, n)) if x - *sum / *n as f32 <= gap => { *sum += x; *n += 1; }
            _ => clusters.push((x, 1)),
        }
    }
    clusters.into_iter().map(|(sum, n)| sum / n as f32).collect()
}

pub(super) fn build_word_set(texts: &[String]) -> std::collections::HashSet<String> {
    let corrected = texts.join(" ")
        .replace('@', "bl").replace(')', "d").replace('&', " p");
    normalise(&corrected).chars()
        .map(|c| if c.is_ascii_alphabetic() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .filter(|w| w.len() >= 3)
        .map(|s| s.to_string())
        .collect()
}

pub(super) fn score_item(display_name: &str, words: &std::collections::HashSet<String>) -> f32 {
    let norm = normalise(display_name);
    let mut seen = std::collections::HashSet::new();
    let item_words: Vec<&str> = norm.split_whitespace()
        .filter(|&w| seen.insert(w))
        .collect();
    if item_words.is_empty() { return 0.0; }
    let n_catalog = item_words.len() as f32;
    let n_ocr = words.len() as f32;
    let matched = item_words.iter()
        .filter(|&&w| word_found_in_set(w, words))
        .count();

    // A full catalog match scores 1.0. A partial match is capped at 0.9, so an
    // extra unmatched catalog word (e.g. "Chassis" on "Lavos Prime Chassis
    // Blueprint") can't beat the exact name via the length bonus below.
    let base = if matched == item_words.len() {
        1.0
    } else {
        (matched as f32 / n_catalog)
            .max(if n_ocr > 0.0 { matched as f32 / n_ocr } else { 0.0 })
            * 0.9
    };

    let len_bonus: f32 = item_words.iter()
        .filter(|&&w| !word_found_in_set(w, words))
        .map(|&cw| {
            words.iter()
                .map(|ow| {
                    let diff = (cw.len() as isize - ow.len() as isize).unsigned_abs();
                    if diff == 0 { 0.08_f32 } else if diff == 1 { 0.04 } else { 0.0 }
                })
                .fold(0.0_f32, f32::max)
        })
        .sum::<f32>() / n_catalog;

    base + len_bonus
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ocr_words(words: &[&str]) -> std::collections::HashSet<String> {
        words.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn four_character_words_must_be_read_exactly() {
        for (catalog_word, on_screen) in [
            ("limb", "limbo"),
            ("gara", "galatine"),
            ("khra", "khora"),
            ("star", "stars"),
        ] {
            assert!(
                !word_found_in_set(catalog_word, &ocr_words(&[on_screen])),
                "{catalog_word:?} must not match {on_screen:?}"
            );
        }
    }

    #[test]
    fn longer_words_keep_their_edit_tolerance() {
        assert!(word_found_in_set("blueprint", &ocr_words(&["bluepnnt"])));
        assert!(word_found_in_set("tenora", &ocr_words(&["tenova"])));
        assert!(word_found_in_set("limb", &ocr_words(&["lim"])));
    }

    #[test]
    fn bar_centers_must_be_evenly_spaced() {
        assert!(!bar_centers_are_valid(&[0.204, 0.316, 0.655]));
        assert!(bar_centers_are_valid(&[0.27, 0.50, 0.73]));
        assert!(bar_centers_are_valid(&[0.24, 0.41, 0.59, 0.76]));
    }
}
