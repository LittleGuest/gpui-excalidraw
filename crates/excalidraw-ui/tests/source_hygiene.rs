//! Source-level invariants that no behavioural test can reach.
//!
//! Both of these guard against failure modes that are invisible in a
//! screenshot and only surface as "the app feels broken" much later.

use excalidraw_ui::i18n::{I18n, Language};

fn source_files() -> Vec<(String, String)> {
    fn walk(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let name = path.file_name().unwrap().to_string_lossy().into_owned();
                out.push((name, std::fs::read_to_string(&path).unwrap()));
            }
        }
    }
    let mut out = Vec::new();
    walk(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .as_path(),
        &mut out,
    );
    out
}

/// Collect every `t("some.key")` literal in the crate.
///
/// The leading-byte check matters: `v.get("appState")` also contains the
/// substring `t("`, so a naive scan reports JSON keys as missing translations.
/// A real call is either `…t("` at a word boundary or `….t("` as a method.
fn referenced_keys(src: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for (idx, _) in src.match_indices("t(\"") {
        if src
            .as_bytes()
            .get(idx.wrapping_sub(1))
            .is_some_and(|prev| prev.is_ascii_alphanumeric() || *prev == b'_')
        {
            continue;
        }
        let rest = &src[idx + 3..];
        if let Some(end) = rest.find('"') {
            let key = &rest[..end];
            if !key.is_empty() {
                keys.push(key.to_string());
            }
        }
    }
    keys
}

/// Nothing the UI asks for may ever come back as its own key name.
///
/// That is exactly what had happened: `menu.new`, `menu.save`, `action.snap`
/// and twenty-one others only ever existed in the Simplified Chinese table, so
/// every other language printed the raw key into the hamburger menu. A missing
/// translation falls back to English, so the only way to see a raw key is for
/// it to be missing from *every* table — which this catches.
#[test]
fn every_key_the_ui_requests_is_translated_in_every_language() {
    let mut missing = Vec::new();
    for (file, src) in source_files() {
        for key in referenced_keys(&src) {
            for language in Language::all() {
                let i18n = I18n::new(language);
                if i18n.t(&key) == key {
                    missing.push(format!(
                        "{file}: t(\"{key}\") is untranslated in {language:?}"
                    ));
                }
            }
        }
    }
    missing.sort();
    missing.dedup();
    assert!(
        missing.is_empty(),
        "raw translation keys would reach the user:\n{}",
        missing.join("\n")
    );
}

/// English is the fallback for all ten other locales, so anything they define
/// but English does not is unreachable — and anything English lacks shows up as
/// a raw key everywhere.
#[test]
fn english_is_a_superset_of_every_other_locale() {
    let all = excalidraw_ui::i18n::translations();
    let en = all.get("en").expect("an English table");

    let mut orphans = Vec::new();
    for (code, table) in &all {
        if *code == "en" {
            continue;
        }
        for (key, value) in table.iter() {
            if !en.contains_key(key) {
                orphans.push(format!("{code}: \"{key}\" = {value:?}"));
            }
        }
    }
    orphans.sort();
    assert!(
        orphans.is_empty(),
        "these translations can never be displayed because English has no \
         matching key:\n{}",
        orphans.join("\n")
    );
}

/// Every state mutation must invalidate the view.
///
/// GPUI's `Context::listener` runs a handler but never calls `notify`, and this
/// view paints a hand-rolled canvas rather than a declarative tree — so a
/// mutation without a notify simply does not appear until something unrelated
/// dirties the window. `Editor::act` is the one place allowed to touch
/// `Context::listener`; it wraps the handler and notifies afterwards. This
/// keeps that funnel from being bypassed.
#[test]
fn all_listeners_go_through_the_notify_funnel() {
    let mut raw = Vec::new();
    for (file, src) in source_files() {
        for (line_no, line) in src.lines().enumerate() {
            if line.contains("cx.listener(") {
                raw.push(format!("{file}:{}: {}", line_no + 1, line.trim()));
            }
        }
    }
    assert_eq!(
        raw.len(),
        1,
        "only `Editor::act` may use Context::listener directly, so that every \
         handler notifies. Found:\n{}",
        raw.join("\n")
    );
    assert!(
        raw[0].starts_with("editor.rs"),
        "the single allowed use should be inside Editor::act, found {}",
        raw[0]
    );
}

/// `{{…}}` in a localized string marks a key name that has to be drawn as a
/// `<kbd>` chip. An unbalanced brace is not a compile error and not a missing
/// translation — it just quietly swallows the rest of the sentence, so every
/// locale is checked here instead of relying on someone noticing a screenshot.
#[test]
fn every_kbd_marker_is_balanced_in_every_locale() {
    let mut broken = Vec::new();
    for (code, table) in excalidraw_ui::i18n::translations() {
        for (key, value) in table.iter() {
            let opens = value.matches("{{").count();
            let closes = value.matches("}}").count();
            if opens != closes {
                broken.push(format!(
                    "{code}: \"{key}\" has {opens} `{{{{` but {closes} `}}}}`"
                ));
            }
            if value.contains("{{}}") {
                broken.push(format!("{code}: \"{key}\" has an empty `{{{{}}}}` chip"));
            }
        }
    }
    broken.sort();
    assert!(
        broken.is_empty(),
        "unbalanced kbd markers would eat hint text:\n{}",
        broken.join("\n")
    );
}

/// The canvas hint is the one string that uses the marker syntax today; if it
/// ever stops using it the chips silently disappear from the footer.
#[test]
fn the_canvas_hint_still_marks_its_key_names() {
    let parts = excalidraw_ui::split_hint(&I18n::new(Language::En).t("hint.moveCanvas"));
    let keys: Vec<&str> = parts
        .iter()
        .filter(|(_, is_key)| *is_key)
        .map(|(run, _)| run.as_str())
        .collect();
    assert_eq!(
        keys,
        vec!["Scroll wheel", "Space"],
        "the hint should chip exactly the two key names upstream chips"
    );
    let rendered: String = parts.iter().map(|(run, _)| run.as_str()).collect();
    assert_eq!(
        rendered, "To move canvas, hold Scroll wheel or Space while dragging, or use the hand tool",
        "stripping the markers must reproduce upstream's plain sentence"
    );
}

/// The command palette and the Help dialog resolve their labels from *data
/// tables* instead of `t("…")` literals, so the scanners above cannot see them.
/// A key that is missing from a locale would print English mid-list — or the
/// raw key if it is missing everywhere — which no screenshot would flag.
#[test]
fn table_driven_labels_are_explicit_in_every_locale() {
    let all = excalidraw_ui::i18n::translations();

    let mut keys: Vec<&'static str> = excalidraw_ui::Editor::palette_labels();
    keys.extend(excalidraw_ui::Editor::help_labels());
    keys.sort();
    keys.dedup();

    let mut missing = Vec::new();
    for key in keys {
        for (code, table) in &all {
            if !table.contains_key(key) {
                missing.push(format!("{code}: \"{key}\""));
            }
        }
    }
    missing.sort();
    missing.dedup();
    assert!(
        missing.is_empty(),
        "these palette/help labels would show English or a raw key:\n{}",
        missing.join("\n")
    );
}

/// English fallback hides gaps: a locale without an explicit entry silently
/// shows English text mid-sentence, which no raw-key check can see. Every key
/// the UI references must be *explicitly* present in every locale's table.
#[test]
fn ui_referenced_keys_are_explicit_in_every_locale() {
    let all = excalidraw_ui::i18n::translations();
    let en = all.get("en").expect("an English table");

    let mut missing = Vec::new();
    for (file, src) in source_files() {
        for key in referenced_keys(&src) {
            if !en.contains_key(key.as_str()) {
                continue; // the raw-key guard above owns that case
            }
            for (code, table) in &all {
                if *code == "en" || table.contains_key(key.as_str()) {
                    continue;
                }
                missing.push(format!("{code}: \"{key}\" (used in {file})"));
            }
        }
    }
    missing.sort();
    missing.dedup();
    assert!(
        missing.is_empty(),
        "these locales would silently show English text:\n{}",
        missing.join("\n")
    );
}
