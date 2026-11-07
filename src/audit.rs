//! Static analysis rules A1–A3 (SPEC §9).
//!
//! A1 (`error`): one combo bound to different commands. A2 (`warn`): binds
//! invisible in the stock hotkey overlay. A3 (`info`): common niri actions
//! with no bind at all. Findings never claim which duplicate "wins" — niri
//! rejects the whole config (see the change notes).

use std::collections::BTreeMap;

use serde::Serialize;

use crate::l10n::{self, Lang, Msg};
use crate::model::{Action, Bind};

/// Severity of a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warn,
    Info,
}

impl Level {
    /// Localized level name for the human-readable report (the JSON form
    /// stays lowercase English via serde).
    pub fn as_str(&self, lang: Lang) -> &'static str {
        match self {
            Level::Error => l10n::tr(lang, Msg::LevelError),
            Level::Warn => l10n::tr(lang, Msg::LevelWarn),
            Level::Info => l10n::tr(lang, Msg::LevelInfo),
        }
    }
}

/// Reference to one of the binds involved in a finding.
#[derive(Debug, Clone, Serialize)]
pub struct BindRef {
    pub keys_raw: String,
    pub file: String,
    pub line: usize,
    /// What exactly this variant does (actions or title).
    pub detail: String,
}

/// One audit finding.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// Rule id: `A1` | `A2` | `A3`.
    pub rule: String,
    pub level: Level,
    pub message: String,
    pub binds: Vec<BindRef>,
}

fn bind_detail(bind: &Bind) -> String {
    if bind.actions.is_empty() {
        bind.title.clone()
    } else {
        bind.actions
            .iter()
            .map(Action::describe)
            .collect::<Vec<_>>()
            .join("; ")
    }
}

fn bind_ref(bind: &Bind) -> BindRef {
    BindRef {
        keys_raw: bind.keys_raw.clone(),
        file: bind.file.clone(),
        line: bind.line,
        detail: bind_detail(bind),
    }
}

/// Rule A1: same canonical combo bound to different commands.
/// Same action on different keys is explicitly not a finding.
fn a1_duplicates(binds: &[Bind], lang: Lang) -> Vec<Finding> {
    let mut by_combo: BTreeMap<&str, Vec<&Bind>> = BTreeMap::new();
    for bind in binds {
        by_combo.entry(bind.combo.as_str()).or_default().push(bind);
    }
    let mut findings = Vec::new();
    for (combo, group) in by_combo {
        if group.len() < 2 {
            continue;
        }
        let distinct: std::collections::BTreeSet<String> =
            group.iter().map(|b| bind_detail(b)).collect();
        if distinct.len() < 2 {
            continue; // same actions duplicated — not a conflict (SPEC §9)
        }
        let message = l10n::a1_message(lang, combo, group.len());
        findings.push(Finding {
            rule: "A1".into(),
            level: Level::Error,
            message,
            binds: group.iter().map(|b| bind_ref(b)).collect(),
        });
    }
    findings
}

/// Rule A2: spawn/spawn-sh binds (or unknown builtins) without
/// `hotkey-overlay-title` — invisible in the stock PNG overlay. Builtins
/// covered by the dictionary get niri's default title and are not findings.
fn a2_overlay_invisible(binds: &[Bind], lang: Lang) -> Vec<Finding> {
    let mut findings = Vec::new();
    for bind in binds {
        if bind.overlay_title_present {
            continue;
        }
        let Some(action) = bind.actions.first() else {
            continue;
        };
        let is_custom = match action.kind.as_str() {
            "spawn" | "spawn-sh" => true,
            _ => action
                .name
                .as_deref()
                .is_none_or(|name| !crate::groups::is_dictionary_action(name)),
        };
        if !is_custom {
            continue;
        }
        findings.push(Finding {
            rule: "A2".into(),
            level: Level::Warn,
            message: l10n::a2_message(lang, &bind.keys_raw, &bind_detail(bind)),
            binds: vec![bind_ref(bind)],
        });
    }
    findings
}

/// Common niri actions worth having a bind for (SPEC §9, A3). The list is
/// static; `clean.kdl` binds every entry so the negative audit test yields
/// zero findings.
pub const COMMON_ACTIONS: &[&str] = &[
    "screenshot",
    "screenshot-screen",
    "screenshot-window",
    "toggle-window-floating",
    "consume-or-expel-window-left",
    "consume-or-expel-window-right",
    "consume-window-into-column",
    "expel-window-from-column",
    "toggle-column-tabbed-display",
    "maximize-column",
    "toggle-overview",
    "switch-focus-between-floating-and-tiling",
    "focus-window-previous",
    "move-column-to-first",
    "move-column-to-last",
    "move-workspace-up",
    "move-workspace-down",
    "expand-column-to-available-width",
    "center-column",
    "center-visible-columns",
    "switch-preset-column-width",
    "switch-preset-window-height",
    "set-window-height",
    "set-column-width",
    "toggle-keyboard-shortcuts-inhibit",
    "power-off-monitors",
    "quit",
    "show-hotkey-overlay",
    "switch-layout",
    "reset-window-height",
];

/// Rule A3: common niri actions with no bind at all.
fn a3_missing_actions(binds: &[Bind], lang: Lang) -> Vec<Finding> {
    let mut bound: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for bind in binds {
        for action in &bind.actions {
            if let Some(name) = &action.name {
                bound.insert(name.as_str());
            }
        }
    }
    COMMON_ACTIONS
        .iter()
        .filter(|name| !bound.contains(**name))
        .map(|name| Finding {
            rule: "A3".into(),
            level: Level::Info,
            message: l10n::a3_message(lang, name),
            binds: vec![],
        })
        .collect()
}

/// Run all rules, deterministic order: A1 (config order), A2 (config order),
/// A3 (static list order). Finding texts follow `lang`.
pub fn audit(binds: &[Bind], lang: Lang) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(a1_duplicates(binds, lang));
    findings.extend(a2_overlay_invisible(binds, lang));
    findings.extend(a3_missing_actions(binds, lang));
    findings
}

/// Human-readable report in the report language (one finding per block).
pub fn render_text(findings: &[Finding], lang: Lang) -> String {
    if findings.is_empty() {
        return format!("{}\n", l10n::tr(lang, Msg::NoFindings));
    }
    let mut out = String::new();
    for finding in findings {
        out.push_str(&format!(
            "{} [{}] {}\n",
            finding.rule,
            finding.level.as_str(lang),
            finding.message
        ));
        for bind in &finding.binds {
            out.push_str(&format!(
                "  {} ({}:{}) — {}\n",
                bind.keys_raw, bind.file, bind.line, bind.detail
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_config;
    use std::path::Path;

    fn fixture_binds(name: &str) -> Vec<Bind> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        parse_config(&path).expect("fixture parses")
    }

    #[test]
    fn a1_finds_conflict_with_both_variants() {
        let binds = fixture_binds("conflict.kdl");
        let findings = audit(&binds, Lang::Ru);
        let a1: Vec<&Finding> = findings.iter().filter(|f| f.rule == "A1").collect();
        assert_eq!(a1.len(), 1);
        assert_eq!(a1[0].level, Level::Error);
        assert_eq!(a1[0].binds.len(), 2, "both variants listed");
        let lines: Vec<usize> = a1[0].binds.iter().map(|b| b.line).collect();
        assert_eq!(lines, [3, 4]);
        let message = &a1[0].message;
        assert!(!message.to_lowercase().contains("побеждает"));
    }

    #[test]
    fn a1_ignores_same_action_on_different_keys() {
        let binds = fixture_binds("edge.kdl");
        let findings = audit(&binds, Lang::Ru);
        // Mod+Left и Mod+H оба focus-column-left — не находка; но дубли
        // фокуса на кириллических клавишах с теми же экшенами — тоже нет.
        let a1: Vec<&Finding> = findings.iter().filter(|f| f.rule == "A1").collect();
        assert_eq!(a1.len(), 1, "only the Mod+D conflict");
        let combos: Vec<&str> = a1[0].binds.iter().map(|b| b.keys_raw.as_str()).collect();
        assert!(combos.contains(&"Mod+D"));
    }

    #[test]
    fn a2_warns_on_spawn_without_overlay_title_only() {
        let binds = fixture_binds("edge.kdl");
        let findings = audit(&binds, Lang::Ru);
        let a2: Vec<&Finding> = findings.iter().filter(|f| f.rule == "A2").collect();
        // spawn-бинды без overlay-title: Mod+T (kitty), Mod+N (noctalia),
        // Mod+K (spawn-sh), Mod+D ×2, XF86AudioMute. Dictionary builtins — нет.
        assert_eq!(a2.len(), 6, "spawn/spawn-sh without overlay title");
        for finding in &a2 {
            assert_eq!(finding.level, Level::Warn);
            assert_eq!(finding.binds.len(), 1);
            let detail = &finding.binds[0].detail;
            assert!(
                detail.starts_with("spawn"),
                "warn targets spawn binds, got: {detail}"
            );
        }
    }

    #[test]
    fn clean_fixture_has_zero_findings() {
        let binds = fixture_binds("clean.kdl");
        let findings = audit(&binds, Lang::Ru);
        assert_eq!(findings.len(), 0, "clean.kdl must be finding-free");
    }

    #[test]
    fn a3_reports_missing_common_actions_on_edge() {
        let binds = fixture_binds("edge.kdl");
        let findings = audit(&binds, Lang::Ru);
        let a3: Vec<&Finding> = findings.iter().filter(|f| f.rule == "A3").collect();
        let reported: std::collections::BTreeSet<&str> = a3
            .iter()
            .filter_map(|f| f.message.rsplit(' ').next())
            .collect();
        // edge.kdl биндит только часть списка: quit, power-off-monitors и др.
        // отсутствуют, а toggle-window-floating/toggle-overview/
        // show-hotkey-overlay/close-window — присутствуют.
        assert!(reported.contains("quit"));
        assert!(reported.contains("screenshot"));
        assert!(!reported.contains("toggle-window-floating"));
        assert!(!reported.contains("toggle-overview"));
        assert!(a3.iter().all(|f| f.level == Level::Info));
    }

    #[test]
    fn render_text_is_russian_and_numbered() {
        let binds = fixture_binds("conflict.kdl");
        let findings = audit(&binds, Lang::Ru);
        let text = render_text(&findings, Lang::Ru);
        assert!(text.contains("A1 [error]"));
        assert!(text.contains("app-one"));
        assert!(text.contains("conflict.kdl:3"));

        let empty = render_text(&[], Lang::Ru);
        assert!(empty.contains("Находок нет"));
    }
}
