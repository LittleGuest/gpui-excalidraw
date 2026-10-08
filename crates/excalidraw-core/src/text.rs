//! Soft wrapping for text that lives inside a container.
//!
//! Excalidraw wraps a bound label to its container's inner width with real
//! canvas metrics (`packages/element/src/textWrapping.ts`). `excalidraw-core`
//! has no font engine, so this module works from a per-character advance
//! estimate instead. Layout (`sync_container_text`, which sizes a sticky note
//! to its label) and the renderer both call through here, so they always agree
//! on where the breaks fall even though the estimate is approximate.

/// Approximate advance width of a single character at `font_size`.
///
/// CJK ideographs and fullwidth forms occupy a full em; everything else is
/// estimated at 0.6em, close enough for Latin text at the sizes Excalidraw
/// uses. A wrong estimate only shifts a break by a character or two.
pub fn char_advance(ch: char, font_size: f64) -> f64 {
    if is_wide(ch) {
        font_size
    } else {
        font_size * 0.6
    }
}

/// Whether a character is drawn at full width (CJK, Hangul, fullwidth forms).
fn is_wide(ch: char) -> bool {
    matches!(ch as u32,
        0x1100..=0x115F   // Hangul Jamo
        | 0x2E80..=0x303E // CJK radicals, Kangxi, symbols
        | 0x3041..=0x33FF // Hiragana, Katakana, CJK compatibility
        | 0x3400..=0x4DBF // CJK extension A
        | 0x4E00..=0x9FFF // CJK unified ideographs
        | 0xA000..=0xA4CF // Yi
        | 0xAC00..=0xD7A3 // Hangul syllables
        | 0xF900..=0xFAFF // CJK compatibility ideographs
        | 0xFE30..=0xFE4F // CJK compatibility forms
        | 0xFF00..=0xFF60 // Fullwidth forms
        | 0xFFE0..=0xFFE6
        | 0x20000..=0x3FFFD // CJK extension B and beyond
    )
}

fn measure(text: &str, font_size: f64) -> f64 {
    text.chars().map(|c| char_advance(c, font_size)).sum()
}

/// Wrap `text` to `max_width`, returning it with soft line breaks inserted.
///
/// Hard `\n` in the input are kept: wrapping is applied to each hard line
/// separately, exactly as upstream does.
pub fn wrap_text(text: &str, max_width: f64, font_size: f64) -> String {
    if !max_width.is_finite() || max_width <= 0.0 || font_size <= 0.0 {
        return text.to_string();
    }
    let mut lines: Vec<String> = Vec::new();
    for hard_line in text.split('\n') {
        lines.extend(wrap_line(hard_line, max_width, font_size));
    }
    lines.join("\n")
}

/// Greedy wrap of one hard line.
///
/// A break is taken before the character that would overflow, preferring the
/// last space on the line so Latin words are not split; text without spaces
/// (CJK, or a single over-long word) breaks between characters, which is what
/// upstream's `wrapWord` does for both.
fn wrap_line(line: &str, max_width: f64, font_size: f64) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current: Vec<char> = Vec::new();
    let mut current_width = 0.0;
    let mut last_space: Option<usize> = None;

    for ch in line.chars() {
        let advance = char_advance(ch, font_size);
        // Whitespace is never the character that wraps: like upstream, it is
        // appended past the limit and trimmed off the previous line when the
        // following token breaks.
        if !ch.is_whitespace() && !current.is_empty() && current_width + advance > max_width {
            if let Some(space) = last_space.filter(|space| *space > 0) {
                let head: String = current[..space].iter().collect();
                let tail: String = current[space + 1..].iter().collect();
                lines.push(head.trim_end().to_string());
                current = tail.chars().collect();
                current_width = measure(&tail, font_size);
                last_space = None;
            } else {
                lines.push(current.iter().collect());
                current.clear();
                current_width = 0.0;
            }
        }
        if ch == ' ' {
            last_space = Some(current.len());
        }
        current.push(ch);
        current_width += advance;
    }

    // Trailing whitespace is not a line of its own; a genuinely blank hard
    // line still has to yield one empty line so the count stays right.
    let tail: String = current.iter().collect();
    let tail = tail.trim_end();
    if !tail.is_empty() || lines.is_empty() {
        lines.push(tail.to_string());
    }
    lines
}
