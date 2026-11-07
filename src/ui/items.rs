//! Pure models and functions of the overlay list: grouping, filtering and
//! selection (SPEC-V2 §4) — unit-testable without a GUI.

use crate::model::Bind;

/// One visible line of the list: a group header or a bind row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    /// Group header with the group name.
    Header(String),
    /// Bind row; `index` points into the original binds vector.
    Bind { index: usize },
}

impl Row {
    /// Bind index when this row is selectable (a bind row).
    pub fn bind_index(&self) -> Option<usize> {
        match self {
            Row::Bind { index } => Some(*index),
            Row::Header(_) => None,
        }
    }
}

/// Group binds by their `group` field in order of first appearance
/// (the binds come from the parser in config order, so this is the
/// config order of groups, as in MVP1 `groups`).
pub fn build_groups(binds: &[Bind]) -> Vec<(String, Vec<usize>)> {
    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    for (index, bind) in binds.iter().enumerate() {
        match groups.iter_mut().find(|(name, _)| *name == bind.group) {
            Some((_, members)) => members.push(index),
            None => groups.push((bind.group.clone(), vec![index])),
        }
    }
    groups
}

/// Visible rows for a search query: header + bind rows per group, in group
/// order. An empty query shows everything; groups without matches are hidden.
/// Matching is a substring check over the lowercase `search` field of the
/// model (the query is lowercased the same way; no new normalizer).
pub fn filter_rows(binds: &[Bind], query: &str) -> Vec<Row> {
    let query = query.trim().to_lowercase();
    let matches =
        |index: usize| -> bool { query.is_empty() || binds[index].search.contains(&query) };
    let mut rows = Vec::new();
    for (name, members) in build_groups(binds) {
        let matched: Vec<usize> = members.into_iter().filter(|i| matches(*i)).collect();
        if matched.is_empty() {
            continue;
        }
        rows.push(Row::Header(name));
        rows.extend(matched.into_iter().map(|index| Row::Bind { index }));
    }
    rows
}

/// Index of the first selectable row (`None` for an empty list).
pub fn first_bind_row(rows: &[Row]) -> Option<usize> {
    rows.iter().position(|row| row.bind_index().is_some())
}

/// Move the selection cyclically over bind rows in both directions,
/// skipping group headers. A `selected` that points at a header (after a
/// filter reset) snaps to the first bind row. Returns `selected` unchanged
/// when there is nothing selectable.
pub fn move_selection(rows: &[Row], selected: usize, delta: i32) -> usize {
    let selectable: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.bind_index().is_some())
        .map(|(position, _)| position)
        .collect();
    let Some(&current) = selectable.iter().find(|&&position| position == selected) else {
        return selectable.first().copied().unwrap_or(selected);
    };
    let count = selectable.len() as i32;
    let position = selectable
        .iter()
        .position(|&p| p == current)
        .expect("current comes from selectable");
    let next = (position as i32 + delta).rem_euclid(count) as usize;
    selectable[next]
}

/// Keycap labels of a bind: the `keys_display` tokens as-is (arrows, Enter,
/// Cyrillic letters; no new key parsing).
pub fn keycaps(bind: &Bind) -> Vec<String> {
    bind.keys_display.clone()
}

/// Indexes of the cheat-sheet binds: every bind carrying a
/// `hotkey-overlay-title` — the same set niri's built-in PNG overlay used
/// to show — in config order. The overlay's right panel is read-only, so
/// this is computed once at boot and never filtered by the search query.
pub fn cheatsheet_binds(binds: &[Bind]) -> Vec<usize> {
    binds
        .iter()
        .enumerate()
        .filter(|(_, bind)| bind.overlay_title_present)
        .map(|(index, _)| index)
        .collect()
}

/// Card width without the cheat-sheet panel (the MVP2 «Центр» layout).
pub const CARD_WIDTH: f32 = 600.0;
/// Card width with the cheat-sheet panel: the 600px list column, a 1px
/// separator and the 300px panel (design D2).
pub const CARD_WIDTH_CHEATSHEET: f32 = 900.0;
/// Fixed width of the cheat-sheet panel on the right (design D2).
pub const CHEATSHEET_WIDTH: f32 = 300.0;

/// Card geometry of the «Центр» layout: centered, 600px wide — or 900px
/// when the cheat-sheet panel is shown (capped at 92% of the window),
/// 640px tall (capped the same way). Returns `(x, y, width, height)`.
/// The single source of the card rectangle for both the layout and the
/// outside-click hit test (design D2).
pub fn card_rect(window: (f32, f32), has_cheatsheet: bool) -> (f32, f32, f32, f32) {
    let base = if has_cheatsheet {
        CARD_WIDTH_CHEATSHEET
    } else {
        CARD_WIDTH
    };
    let width = base.min(window.0 * 0.92);
    let height = 640.0_f32.min(window.1 * 0.92);
    (
        (window.0 - width) / 2.0,
        (window.1 - height) / 2.0,
        width,
        height,
    )
}

/// Whether a cursor position is outside the card (a click there closes the
/// overlay, SPEC-V2 §5.7).
pub fn is_outside_card(cursor: (f32, f32), window: (f32, f32), has_cheatsheet: bool) -> bool {
    let (x, y, width, height) = card_rect(window, has_cheatsheet);
    cursor.0 < x || cursor.0 >= x + width || cursor.1 < y || cursor.1 >= y + height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Action, Bind};

    fn bind(id: &str, title: &str, keys: &str, group: &str) -> Bind {
        Bind {
            id: id.into(),
            title: title.into(),
            title_source: "overlay-title".into(),
            actions: vec![Action {
                kind: "builtin".into(),
                name: Some("close-window".into()),
                cmd: None,
            }],
            keys_raw: keys.into(),
            keys_display: keys.split('+').map(String::from).collect(),
            combo: keys.to_lowercase(),
            group: group.into(),
            repeat: true,
            overlay_title_present: true,
            overlay_title: Some(title.into()),
            file: "/tmp/c.kdl".into(),
            line: 1,
            search: format!("{title} {keys} spawn close-window {group}").to_lowercase(),
            raw: String::new(),
        }
    }

    fn sample() -> Vec<Bind> {
        vec![
            bind("a", "Закрыть окно", "Mod+Q", "Окна"),
            bind("b", "На воркспейс", "Mod+1", "Воркспейсы"),
            bind("c", "Воркспейс выше", "Mod+Ctrl+Up", "Воркспейсы"),
            bind("d", "Скриншот", "Print", "Скриншоты"),
        ]
    }

    #[test]
    fn groups_follow_first_appearance_order() {
        let groups = build_groups(&sample());
        let names: Vec<&str> = groups.iter().map(|(name, _)| name.as_str()).collect();
        assert_eq!(names, ["Окна", "Воркспейсы", "Скриншоты"]);
        assert_eq!(groups[1].1, vec![1, 2], "members keep config order");
    }

    #[test]
    fn empty_config_has_no_groups() {
        assert!(build_groups(&[]).is_empty());
        assert!(filter_rows(&[], "").is_empty());
    }

    #[test]
    fn empty_query_shows_everything() {
        let rows = filter_rows(&sample(), "");
        assert_eq!(rows.len(), 3 + 4, "3 headers + 4 binds");
        assert_eq!(rows[0], Row::Header("Окна".into()));
        assert_eq!(rows[1], Row::Bind { index: 0 });
    }

    #[test]
    fn russian_query_matches_search_field() {
        let rows = filter_rows(&sample(), "воркспейс");
        let binds: Vec<usize> = rows.iter().filter_map(|row| row.bind_index()).collect();
        assert_eq!(binds, vec![1, 2], "only workspace binds match");
        assert_eq!(rows[0], Row::Header("Воркспейсы".into()));
    }

    #[test]
    fn keys_and_mixed_case_query_match() {
        assert_eq!(
            filter_rows(&sample(), "mod+shift"),
            Vec::<Row>::new(),
            "sample has no mod+shift binds"
        );
        assert_eq!(filter_rows(&sample(), "MOD+Q").len(), 2, "header + bind");
        // Cyrillic query against the group name too.
        assert_eq!(filter_rows(&sample(), "скриншоты").len(), 2);
    }

    #[test]
    fn group_without_matches_is_hidden() {
        let rows = filter_rows(&sample(), "воркспейс");
        assert!(
            !rows
                .iter()
                .any(|row| matches!(row, Row::Header(name) if name == "Окна"))
        );
    }

    #[test]
    fn selection_moves_and_wraps() {
        let rows = filter_rows(&sample(), "");
        // Bind rows are at positions 1, 3, 4, 6 (Воркспейсы holds two binds).
        assert_eq!(first_bind_row(&rows), Some(1));
        assert_eq!(move_selection(&rows, 1, 1), 3);
        assert_eq!(
            move_selection(&rows, 3, -2),
            6,
            "two up from the second bind wraps to last"
        );
        assert_eq!(move_selection(&rows, 6, 1), 1, "wrap down to first");
        assert_eq!(move_selection(&rows, 1, -1), 6, "wrap up to last");
    }

    #[test]
    fn selection_snaps_after_filter_reset() {
        let rows = filter_rows(&sample(), "воркспейс");
        // Position 0 is a header — the reset case after a filter change;
        // either direction snaps to the first bind row (position 1).
        assert_eq!(move_selection(&rows, 0, 1), 1);
        assert_eq!(move_selection(&rows, 0, -1), 1, "snaps to first bind");
    }

    #[test]
    fn selection_is_safe_on_degenerate_lists() {
        assert_eq!(move_selection(&[], 0, 1), 0);
        let header_only = vec![Row::Header("Окна".into())];
        assert_eq!(move_selection(&header_only, 0, 1), 0);
    }

    #[test]
    fn keycaps_are_keys_display_tokens() {
        let binds = sample();
        assert_eq!(keycaps(&binds[0]), vec!["Mod", "Q"]);
        let cyrillic = bind("e", "Кириллица", "Mod+В", "Прочее");
        assert_eq!(keycaps(&cyrillic), vec!["Mod", "В"], "Cyrillic as symbols");
        let arrows = bind("f", "Стрелки", "Mod+Ctrl+Up", "Навигация");
        assert_eq!(keycaps(&arrows), vec!["Mod", "Ctrl", "Up"]);
    }

    #[test]
    fn card_geometry_and_hit_test() {
        // Single-column card (no titled binds): the MVP2 geometry.
        let (x, y, w, h) = card_rect((2560.0, 1440.0), false);
        assert_eq!((w, h), (600.0, 640.0));
        assert_eq!((x, y), (980.0, 400.0), "card is centered");
        assert!(!is_outside_card((1280.0, 720.0), (2560.0, 1440.0), false));
        assert!(is_outside_card((100.0, 720.0), (2560.0, 1440.0), false));
        assert!(is_outside_card((1280.0, 100.0), (2560.0, 1440.0), false));
        // Two-column card with the cheat-sheet panel (design D2).
        let (x, y, w, h) = card_rect((2560.0, 1440.0), true);
        assert_eq!((w, h), (900.0, 640.0));
        assert_eq!((x, y), (830.0, 400.0), "wider card stays centered");
        assert!(!is_outside_card((1280.0, 720.0), (2560.0, 1440.0), true));
        assert!(
            is_outside_card((820.0, 720.0), (2560.0, 1440.0), true),
            "left of the 900px card closes"
        );
        assert!(
            !is_outside_card((840.0, 720.0), (2560.0, 1440.0), true),
            "inside the 900px card does not close"
        );
    }

    #[test]
    fn card_width_is_capped_at_92_percent() {
        let (_, _, w, h) = card_rect((500.0, 600.0), false);
        assert_eq!(w, 460.0, "0.92 * 500");
        assert_eq!(h, 552.0, "0.92 * 600");
        let (_, _, w, _) = card_rect((500.0, 600.0), true);
        assert_eq!(w, 460.0, "the 900px card is capped the same way");
    }

    #[test]
    fn cheatsheet_selects_only_titled_binds_in_config_order() {
        let mut binds = sample();
        binds[1].overlay_title_present = false;
        binds[3].overlay_title_present = false;
        assert_eq!(cheatsheet_binds(&binds), vec![0, 2], "config order kept");
    }

    #[test]
    fn cheatsheet_empty_config_is_empty() {
        assert!(cheatsheet_binds(&[]).is_empty());
        let all_untitled: Vec<_> = sample()
            .into_iter()
            .map(|mut bind| {
                bind.overlay_title_present = false;
                bind
            })
            .collect();
        assert!(cheatsheet_binds(&all_untitled).is_empty());
    }

    #[test]
    fn cheatsheet_includes_every_titled_bind() {
        // `sample()` marks every bind with an overlay title.
        assert_eq!(cheatsheet_binds(&sample()), vec![0, 1, 2, 3]);
    }
}
