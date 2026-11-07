//! Combo canonicalization and human-friendly key display (SPEC §6, §7.2).
//!
//! The Cyrillic table is copied from the local reference
//! (`/usr/include/X11/keysymdef.h`, base Russian block 0x06c0–0x06df), never
//! from memory; every entry is pinned by a unit test. `Cyrillic_slash` does
//! not exist as a keysym (see the comment in the live config) and therefore
//! falls into the "unknown → raw" rule.

/// `Cyrillic_*` keysym name suffix → Unicode symbol (lowercase, as per the
/// keysymdef.h comments). Lookup is case-insensitive on the suffix.
pub const CYRILLIC: &[(&str, &str)] = &[
    ("yu", "ю"),
    ("a", "а"),
    ("be", "б"),
    ("tse", "ц"),
    ("de", "д"),
    ("ie", "е"),
    ("ef", "ф"),
    ("ghe", "г"),
    ("ha", "х"),
    ("i", "и"),
    ("shorti", "й"),
    ("ka", "к"),
    ("el", "л"),
    ("em", "м"),
    ("en", "н"),
    ("o", "о"),
    ("pe", "п"),
    ("ya", "я"),
    ("er", "р"),
    ("es", "с"),
    ("te", "т"),
    ("u", "у"),
    ("zhe", "ж"),
    ("ve", "в"),
    ("softsign", "ь"),
    ("yeru", "ы"),
    ("ze", "з"),
    ("sha", "ш"),
    ("e", "э"),
    ("shcha", "щ"),
    ("che", "ч"),
    ("hardsign", "ъ"),
];

/// Canonical order of modifiers in `combo` (SPEC §6).
const MODIFIER_ORDER: [&str; 4] = ["mod", "ctrl", "shift", "alt"];

/// Special key display mapping (SPEC §6).
const SPECIAL_DISPLAY: &[(&str, &str)] = &[
    ("return", "Enter"),
    ("left", "←"),
    ("right", "→"),
    ("up", "↑"),
    ("down", "↓"),
    ("slash", "/"),
];

fn known_modifier(token_lower: &str) -> Option<&'static str> {
    match token_lower {
        "mod" | "super" => Some("mod"),
        "ctrl" | "control" => Some("ctrl"),
        "shift" => Some("shift"),
        "alt" => Some("alt"),
        _ => None,
    }
}

/// Canonical audit key of a combo: modifiers in `mod, ctrl, shift, alt`
/// order, remaining modifier-like tokens in appearance order, key lowercase.
pub fn canonical_combo(keys_raw: &str) -> String {
    let tokens: Vec<&str> = keys_raw.split('+').map(str::trim).collect();
    let (mods, key) = match tokens.split_last() {
        Some((key, mods)) if !key.is_empty() => (mods, *key),
        // Degenerate input ("Mod+" or ""): canonicalize whatever is there.
        _ => return keys_raw.trim().to_lowercase(),
    };

    let mut canonical: Vec<&str> = vec![];
    let mut extra: Vec<String> = vec![];
    for token in mods {
        let lower = token.to_lowercase();
        let known = known_modifier(&lower);
        match known {
            Some(m) => {
                if !canonical.contains(&m) {
                    canonical.push(m);
                }
            }
            None => {
                if !extra.iter().any(|t| t == &lower) {
                    extra.push(lower);
                }
            }
        }
    }
    // Rebuild in canonical order: known modifiers sorted, then the rest.
    let mut ordered: Vec<String> = MODIFIER_ORDER
        .iter()
        .filter(|m| canonical.contains(*m))
        .map(|m| (*m).to_string())
        .collect();
    ordered.extend(extra);
    ordered.push(key.to_lowercase());
    ordered.join("+")
}

/// Display form of one key token; unknown names pass through unchanged.
pub fn display_key(key: &str) -> String {
    let lower = key.to_lowercase();
    if let Some((_, sym)) = SPECIAL_DISPLAY.iter().find(|(name, _)| *name == lower) {
        return (*sym).to_string();
    }
    if let Some(rest) = lower.strip_prefix("cyrillic_")
        && let Some((_, sym)) = CYRILLIC.iter().find(|(name, _)| *name == rest)
    {
        return (*sym).to_string();
    }
    key.to_string()
}

/// Display form of every key token of the combo.
pub fn keys_display(keys_raw: &str) -> Vec<String> {
    keys_raw
        .split('+')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(display_key)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every base-block pair taken from keysymdef.h, pinned one by one.
    const EXPECTED_PAIRS: &[(&str, &str)] = &[
        ("Cyrillic_yu", "ю"),       // 0x06c0 U+044E
        ("Cyrillic_a", "а"),        // 0x06c1 U+0430
        ("Cyrillic_be", "б"),       // 0x06c2 U+0431
        ("Cyrillic_tse", "ц"),      // 0x06c3 U+0446
        ("Cyrillic_de", "д"),       // 0x06c4 U+0434
        ("Cyrillic_ie", "е"),       // 0x06c5 U+0435
        ("Cyrillic_ef", "ф"),       // 0x06c6 U+0444
        ("Cyrillic_ghe", "г"),      // 0x06c7 U+0433
        ("Cyrillic_ha", "х"),       // 0x06c8 U+0445
        ("Cyrillic_i", "и"),        // 0x06c9 U+0438
        ("Cyrillic_shorti", "й"),   // 0x06ca U+0439
        ("Cyrillic_ka", "к"),       // 0x06cb U+043A
        ("Cyrillic_el", "л"),       // 0x06cc U+043B
        ("Cyrillic_em", "м"),       // 0x06cd U+043C
        ("Cyrillic_en", "н"),       // 0x06ce U+043D
        ("Cyrillic_o", "о"),        // 0x06cf U+043E
        ("Cyrillic_pe", "п"),       // 0x06d0 U+043F
        ("Cyrillic_ya", "я"),       // 0x06d1 U+044F
        ("Cyrillic_er", "р"),       // 0x06d2 U+0440
        ("Cyrillic_es", "с"),       // 0x06d3 U+0441
        ("Cyrillic_te", "т"),       // 0x06d4 U+0442
        ("Cyrillic_u", "у"),        // 0x06d5 U+0443
        ("Cyrillic_zhe", "ж"),      // 0x06d6 U+0436
        ("Cyrillic_ve", "в"),       // 0x06d7 U+0432
        ("Cyrillic_softsign", "ь"), // 0x06d8 U+044C
        ("Cyrillic_yeru", "ы"),     // 0x06d9 U+044B
        ("Cyrillic_ze", "з"),       // 0x06da U+0437
        ("Cyrillic_sha", "ш"),      // 0x06db U+0448
        ("Cyrillic_e", "э"),        // 0x06dc U+044D
        ("Cyrillic_shcha", "щ"),    // 0x06dd U+0449
        ("Cyrillic_che", "ч"),      // 0x06de U+0447
        ("Cyrillic_hardsign", "ъ"), // 0x06df U+044A
    ];

    #[test]
    fn cyrillic_table_matches_keysymdef_reference() {
        for (name, expected) in EXPECTED_PAIRS {
            assert_eq!(
                display_key(name),
                *expected,
                "table entry for {name} diverges from keysymdef.h"
            );
        }
    }

    /// Every §7.2 name required by the SPEC: the 23 real keysyms map to
    /// symbols, `Cyrillic_slash` (no such keysym) stays raw.
    #[test]
    fn spec_section_7_2_names() {
        for name in [
            "Cyrillic_a",
            "Cyrillic_be",
            "Cyrillic_de",
            "Cyrillic_el",
            "Cyrillic_em",
            "Cyrillic_er",
            "Cyrillic_es",
            "Cyrillic_ghe",
            "Cyrillic_ha",
            "Cyrillic_hardsign",
            "Cyrillic_i",
            "Cyrillic_ie",
            "Cyrillic_ka",
            "Cyrillic_o",
            "Cyrillic_sha",
            "Cyrillic_shcha",
            "Cyrillic_shorti",
            "Cyrillic_tse",
            "Cyrillic_u",
            "Cyrillic_ve",
            "Cyrillic_yeru",
            "Cyrillic_yu",
            "Cyrillic_ze",
        ] {
            let displayed = display_key(name);
            assert_ne!(displayed, name, "{name} must map to a symbol");
        }
        assert_eq!(display_key("Cyrillic_slash"), "Cyrillic_slash");
    }

    #[test]
    fn table_and_expected_pairs_are_in_sync() {
        for (name, _) in CYRILLIC {
            let expected = EXPECTED_PAIRS
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(&format!("Cyrillic_{name}")))
                .map(|(_, s)| *s)
                .unwrap_or_else(|| panic!("{name} has no pinned expectation"));
            assert_eq!(display_key(&format!("Cyrillic_{name}")), expected);
        }
        assert_eq!(CYRILLIC.len(), EXPECTED_PAIRS.len());
    }

    #[test]
    fn special_keys_display() {
        assert_eq!(display_key("Return"), "Enter");
        assert_eq!(display_key("Left"), "←");
        assert_eq!(display_key("Right"), "→");
        assert_eq!(display_key("Up"), "↑");
        assert_eq!(display_key("Down"), "↓");
        assert_eq!(display_key("Slash"), "/");
    }

    #[test]
    fn unknown_keys_pass_through() {
        assert_eq!(display_key("XF86AudioMute"), "XF86AudioMute");
        assert_eq!(display_key("Print"), "Print");
    }

    #[test]
    fn canonical_combo_orders_modifiers() {
        assert_eq!(canonical_combo("Mod+Shift+W"), "mod+shift+w");
        assert_eq!(canonical_combo("Shift+Mod+W"), "mod+shift+w");
        assert_eq!(canonical_combo("Ctrl+Alt+Left"), "ctrl+alt+left");
        assert_eq!(canonical_combo("Alt+Ctrl+Delete"), "ctrl+alt+delete");
        assert_eq!(canonical_combo("Mod+Cyrillic_ve"), "mod+cyrillic_ve");
        assert_eq!(canonical_combo("Mod"), "mod");
        assert_eq!(canonical_combo("Print"), "print");
    }

    #[test]
    fn keys_display_splits_tokens() {
        assert_eq!(
            keys_display("Mod+Shift+W"),
            vec!["Mod".to_string(), "Shift".to_string(), "W".to_string()]
        );
        assert_eq!(
            keys_display("Mod+Shift+Slash"),
            vec!["Mod".to_string(), "Shift".to_string(), "/".to_string()]
        );
        assert_eq!(
            keys_display("Mod+Cyrillic_ve"),
            vec!["Mod".to_string(), "в".to_string()]
        );
        assert_eq!(keys_display("Return"), vec!["Enter".to_string()]);
    }
}
