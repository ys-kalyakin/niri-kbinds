//! Deterministic action grouping and localized titles (SPEC §6, design D5).
//!
//! The dictionary is seeded from the actions actually present in the user's
//! live config; unknown actions fall back to prefix rules and finally to the
//! `Other` group with the raw action name as the title — grouping never
//! fails and is stable between runs. Titles and group names follow the
//! `Lang` passed by the caller (capability `localization`); dictionary
//! membership itself is locale-independent.

use crate::l10n::Lang;
use crate::model::{ActionKind, Bind, TitleSource};

/// Display groups of the bind list. The set is fixed; localized names live
/// in [`Group::name`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    /// Window management actions.
    Windows,
    /// Workspace actions.
    Workspaces,
    /// Focus and navigation actions.
    Navigation,
    /// Screenshot actions.
    Screenshots,
    /// Shell/launcher layer actions.
    System,
    /// Spawn binds of regular applications.
    Apps,
    /// Everything the rules above do not classify.
    Other,
}

impl Group {
    /// Localized display name of the group.
    pub fn name(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Group::Windows, Lang::En) => "Windows",
            (Group::Windows, Lang::Ru) => "Окна",
            (Group::Workspaces, Lang::En) => "Workspaces",
            (Group::Workspaces, Lang::Ru) => "Воркспейсы",
            (Group::Navigation, Lang::En) => "Navigation",
            (Group::Navigation, Lang::Ru) => "Навигация",
            (Group::Screenshots, Lang::En) => "Screenshots",
            (Group::Screenshots, Lang::Ru) => "Скриншоты",
            (Group::System, Lang::En) => "System",
            (Group::System, Lang::Ru) => "Система",
            (Group::Apps, Lang::En) => "Apps",
            (Group::Apps, Lang::Ru) => "Программы",
            (Group::Other, Lang::En) => "Other",
            (Group::Other, Lang::Ru) => "Прочее",
        }
    }
}

/// Exact action name → (group, English title, Russian title). One entry
/// carries both translations, so a missing translation fails to compile.
const ACTION_TITLES: &[(&str, Group, &str, &str)] = &[
    // Окна / Windows
    (
        "close-window",
        Group::Windows,
        "Close window",
        "Закрыть окно",
    ),
    (
        "fullscreen-window",
        Group::Windows,
        "Toggle window fullscreen",
        "Полноэкранный режим окна",
    ),
    (
        "toggle-window-floating",
        Group::Windows,
        "Toggle window floating",
        "Плавающее окно: вкл/выкл",
    ),
    (
        "toggle-column-tabbed-display",
        Group::Windows,
        "Toggle column tabbed display",
        "Вкладки колонки: вкл/выкл",
    ),
    (
        "maximize-column",
        Group::Windows,
        "Maximize column",
        "Развернуть колонку",
    ),
    (
        "set-window-height",
        Group::Windows,
        "Set window height",
        "Высота окна",
    ),
    (
        "set-column-width",
        Group::Windows,
        "Set column width",
        "Ширина колонки",
    ),
    (
        "reset-window-height",
        Group::Windows,
        "Reset window height",
        "Сбросить высоту окна",
    ),
    (
        "reset-window-height-full",
        Group::Windows,
        "Reset window height to full",
        "Высота окна на весь экран",
    ),
    (
        "switch-preset-window-height",
        Group::Windows,
        "Switch preset window height",
        "Пресет высоты окна",
    ),
    (
        "switch-preset-column-width",
        Group::Windows,
        "Switch preset column width",
        "Пресет ширины колонки",
    ),
    (
        "expand-column-to-available-width",
        Group::Windows,
        "Expand column to available width",
        "Колонка на всю ширину",
    ),
    (
        "center-column",
        Group::Windows,
        "Center column",
        "Колонка в центр",
    ),
    (
        "center-visible-columns",
        Group::Windows,
        "Center visible columns",
        "Видимые колонки в центр",
    ),
    (
        "consume-window-into-column",
        Group::Windows,
        "Consume window into column",
        "Влить окно в колонку",
    ),
    (
        "expel-window-from-column",
        Group::Windows,
        "Expel window from column",
        "Выдавить окно из колонки",
    ),
    (
        "consume-or-expel-window-left",
        Group::Windows,
        "Consume or expel window left",
        "Влить/выдавить окно влево",
    ),
    (
        "consume-or-expel-window-right",
        Group::Windows,
        "Consume or expel window right",
        "Влить/выдавить окно вправо",
    ),
    (
        "move-column-left",
        Group::Windows,
        "Move column left",
        "Переместить колонку влево",
    ),
    (
        "move-column-right",
        Group::Windows,
        "Move column right",
        "Переместить колонку вправо",
    ),
    (
        "move-column-up",
        Group::Windows,
        "Move column up",
        "Переместить колонку вверх",
    ),
    (
        "move-column-down",
        Group::Windows,
        "Move column down",
        "Переместить колонку вниз",
    ),
    (
        "move-window-up",
        Group::Windows,
        "Move window up",
        "Переместить окно вверх",
    ),
    (
        "move-window-down",
        Group::Windows,
        "Move window down",
        "Переместить окно вниз",
    ),
    (
        "move-column-to-first",
        Group::Windows,
        "Move column to first",
        "Колонку в начало",
    ),
    (
        "move-column-to-last",
        Group::Windows,
        "Move column to last",
        "Колонку в конец",
    ),
    (
        "move-column-to-index",
        Group::Windows,
        "Move column to index",
        "Колонку на позицию",
    ),
    (
        "move-column-to-monitor-up",
        Group::Windows,
        "Move column to monitor up",
        "Колонку на монитор вверх",
    ),
    (
        "move-column-to-monitor-down",
        Group::Windows,
        "Move column to monitor down",
        "Колонку на монитор вниз",
    ),
    (
        "move-column-to-monitor-left",
        Group::Windows,
        "Move column to monitor left",
        "Колонку на монитор влево",
    ),
    (
        "move-column-to-monitor-right",
        Group::Windows,
        "Move column to monitor right",
        "Колонку на монитор вправо",
    ),
    (
        "swap-window-in-column-up",
        Group::Windows,
        "Swap window in column up",
        "Обменять окно в колонке вверх",
    ),
    (
        "swap-window-in-column-down",
        Group::Windows,
        "Swap window in column down",
        "Обменять окно в колонке вниз",
    ),
    // Воркспейсы / Workspaces
    (
        "move-window-to-workspace",
        Group::Workspaces,
        "Move window to workspace",
        "Окно на воркспейс",
    ),
    (
        "move-window-to-workspace-up",
        Group::Workspaces,
        "Move window to workspace above",
        "Окно на воркспейс выше",
    ),
    (
        "move-window-to-workspace-down",
        Group::Workspaces,
        "Move window to workspace below",
        "Окно на воркспейс ниже",
    ),
    (
        "focus-workspace",
        Group::Workspaces,
        "Go to workspace",
        "На воркспейс",
    ),
    (
        "focus-workspace-up",
        Group::Workspaces,
        "Workspace above",
        "Воркспейс выше",
    ),
    (
        "focus-workspace-down",
        Group::Workspaces,
        "Workspace below",
        "Воркспейс ниже",
    ),
    (
        "focus-workspace-previous",
        Group::Workspaces,
        "Previous workspace",
        "Предыдущий воркспейс",
    ),
    (
        "move-column-to-workspace",
        Group::Workspaces,
        "Move column to workspace",
        "Колонку на воркспейс",
    ),
    (
        "move-column-to-workspace-up",
        Group::Workspaces,
        "Move column to workspace above",
        "Колонку на воркспейс выше",
    ),
    (
        "move-column-to-workspace-down",
        Group::Workspaces,
        "Move column to workspace below",
        "Колонку на воркспейс ниже",
    ),
    (
        "move-workspace-up",
        Group::Workspaces,
        "Move workspace up",
        "Переместить воркспейс выше",
    ),
    (
        "move-workspace-down",
        Group::Workspaces,
        "Move workspace down",
        "Переместить воркспейс ниже",
    ),
    (
        "toggle-preempt",
        Group::Workspaces,
        "Toggle workspace focus preemption",
        "Перехват фокуса воркспейса",
    ),
    // Навигация / Navigation
    (
        "focus-column-left",
        Group::Navigation,
        "Column left",
        "Колонка влево",
    ),
    (
        "focus-column-right",
        Group::Navigation,
        "Column right",
        "Колонка вправо",
    ),
    (
        "focus-column-up",
        Group::Navigation,
        "Column up",
        "Колонка выше",
    ),
    (
        "focus-column-down",
        Group::Navigation,
        "Column down",
        "Колонка ниже",
    ),
    (
        "focus-column-first",
        Group::Navigation,
        "First column",
        "Первая колонка",
    ),
    (
        "focus-column-last",
        Group::Navigation,
        "Last column",
        "Последняя колонка",
    ),
    (
        "focus-column-left-or-top",
        Group::Navigation,
        "Column left or top",
        "Колонка влево или в начало",
    ),
    (
        "focus-column-right-or-bottom",
        Group::Navigation,
        "Column right or bottom",
        "Колонка вправо или в конец",
    ),
    (
        "focus-window-up",
        Group::Navigation,
        "Window above",
        "Окно выше",
    ),
    (
        "focus-window-down",
        Group::Navigation,
        "Window below",
        "Окно ниже",
    ),
    (
        "focus-window-previous",
        Group::Navigation,
        "Previous window",
        "Предыдущее окно",
    ),
    (
        "focus-monitor-up",
        Group::Navigation,
        "Monitor above",
        "Монитор выше",
    ),
    (
        "focus-monitor-down",
        Group::Navigation,
        "Monitor below",
        "Монитор ниже",
    ),
    (
        "focus-monitor-left",
        Group::Navigation,
        "Monitor left",
        "Монитор влево",
    ),
    (
        "focus-monitor-right",
        Group::Navigation,
        "Monitor right",
        "Монитор вправо",
    ),
    (
        "focus-monitor-previous",
        Group::Navigation,
        "Previous monitor",
        "Предыдущий монитор",
    ),
    (
        "switch-focus-between-floating-and-tiling",
        Group::Navigation,
        "Focus floating/tiling",
        "Фокус: плавающее/тайлинг",
    ),
    (
        "focus-window-or-workspace-left",
        Group::Navigation,
        "Window/workspace left",
        "Окно/воркспейс влево",
    ),
    (
        "focus-window-or-workspace-right",
        Group::Navigation,
        "Window/workspace right",
        "Окно/воркспейс вправо",
    ),
    (
        "focus-window-or-workspace-up",
        Group::Navigation,
        "Window/workspace up",
        "Окно/воркспейс выше",
    ),
    (
        "focus-window-or-workspace-down",
        Group::Navigation,
        "Window/workspace down",
        "Окно/воркспейс ниже",
    ),
    ("toggle-overview", Group::Navigation, "Overview", "Обзор"),
    // Скриншоты / Screenshots
    ("screenshot", Group::Screenshots, "Screenshot", "Скриншот"),
    (
        "screenshot-screen",
        Group::Screenshots,
        "Screenshot screen",
        "Скриншот экрана",
    ),
    (
        "screenshot-window",
        Group::Screenshots,
        "Screenshot window",
        "Скриншот окна",
    ),
    // Система / System
    (
        "show-hotkey-overlay",
        Group::System,
        "Hotkey overlay",
        "Шпаргалка хоткеев",
    ),
    (
        "switch-layout",
        Group::System,
        "Switch layout",
        "Переключить раскладку",
    ),
    ("quit", Group::System, "Quit niri", "Выйти из niri"),
    (
        "quit-skip-confirmation",
        Group::System,
        "Quit niri without confirmation",
        "Выйти из niri без подтверждения",
    ),
    (
        "power-off-monitors",
        Group::System,
        "Power off monitors",
        "Погасить мониторы",
    ),
    (
        "toggle-keyboard-shortcuts-inhibit",
        Group::System,
        "Inhibit system shortcuts",
        "Запрет системных хоткеев",
    ),
    (
        "toggle-debug-tint",
        Group::System,
        "Toggle debug tint",
        "Отладочная подсветка",
    ),
    (
        "disable-passthrough-keybinds",
        Group::System,
        "Disable passthrough binds",
        "Отключить passthrough-бинды",
    ),
];

/// Whether the builtin action has a dictionary title (niri shows its
/// default title in the stock overlay). Locale-independent — the A2 audit
/// rule relies on membership, not on the wording.
pub fn is_dictionary_action(name: &str) -> bool {
    dictionary_entry(name).is_some()
}

fn dictionary_entry(name: &str) -> Option<&(&'static str, Group, &'static str, &'static str)> {
    ACTION_TITLES.iter().find(|(n, _, _, _)| *n == name)
}

/// Group for a builtin action by prefix when the exact name is unknown.
fn group_by_prefix(name: &str) -> Group {
    if name.starts_with("focus-workspace")
        || name.starts_with("move-column-to-workspace")
        || name.starts_with("move-window-to-workspace")
        || name.starts_with("move-workspace")
        || name.starts_with("workspace-")
    {
        return Group::Workspaces;
    }
    if name.starts_with("screenshot") {
        return Group::Screenshots;
    }
    if name.starts_with("focus-")
        || name.starts_with("switch-focus")
        || name.starts_with("overview")
    {
        return Group::Navigation;
    }
    if name.starts_with("move-")
        || name.starts_with("consume-")
        || name.starts_with("expel-")
        || name.starts_with("swap-")
        || name.starts_with("close-")
        || name.starts_with("fullscreen-")
        || name.starts_with("maximize-")
        || name.starts_with("set-window")
        || name.starts_with("set-column")
        || name.starts_with("reset-")
        || name.starts_with("switch-preset-")
        || name.starts_with("expand-column")
        || name.starts_with("center-")
        || name.starts_with("toggle-window")
        || name.starts_with("toggle-column")
    {
        return Group::Windows;
    }
    Group::Other
}

/// Spawn programs that belong to the shell/launcher layer (`System`).
const SYSTEM_SPAWN_PROGRAMS: &[&str] = &["noctalia", "vicinae", "fuzzel", "niri"];

/// Group for a spawn bind by its program name.
fn spawn_group(cmd: &[String]) -> Group {
    let first = cmd.first().map(String::as_str).unwrap_or_default();
    let program = first.split_whitespace().next().unwrap_or(first);
    if SYSTEM_SPAWN_PROGRAMS.contains(&program) {
        Group::System
    } else {
        Group::Apps
    }
}

/// Title/group/source for one bind, following the priority of SPEC §6:
/// `hotkey-overlay-title` → dictionary action name → raw action name →
/// spawn command line.
fn title_and_group(bind: &Bind, lang: Lang) -> (String, TitleSource, String) {
    if let Some(overlay) = &bind.overlay_title {
        return (
            overlay.clone(),
            TitleSource::OverlayTitle,
            group_for_actions(bind, lang),
        );
    }
    group_and_title_for_actions(bind, lang)
}

fn group_for_actions(bind: &Bind, lang: Lang) -> String {
    let Some(action) = bind.actions.first() else {
        return Group::Other.name(lang).to_string();
    };
    match action.kind {
        ActionKind::Spawn | ActionKind::SpawnSh => {
            let cmd = action.cmd.clone().unwrap_or_default();
            spawn_group(&cmd).name(lang).to_string()
        }
        ActionKind::Builtin => {
            let name = action.name.as_deref().unwrap_or_default();
            match dictionary_entry(name) {
                Some((_, group, _, _)) => group.name(lang).to_string(),
                None => group_by_prefix(name).name(lang).to_string(),
            }
        }
    }
}

fn group_and_title_for_actions(bind: &Bind, lang: Lang) -> (String, TitleSource, String) {
    let Some(action) = bind.actions.first() else {
        return (
            bind.keys_raw.clone(),
            TitleSource::RawAction,
            Group::Other.name(lang).to_string(),
        );
    };
    match action.kind {
        ActionKind::Spawn | ActionKind::SpawnSh => {
            let cmd_text = action.cmd.clone().unwrap_or_default().join(" ");
            (
                cmd_text,
                TitleSource::Command,
                group_for_actions(bind, lang),
            )
        }
        ActionKind::Builtin => {
            let name = action.name.clone().unwrap_or_default();
            match dictionary_entry(&name) {
                Some((_, group, title_en, title_ru)) => {
                    let title = match lang {
                        Lang::En => *title_en,
                        Lang::Ru => *title_ru,
                    };
                    (
                        title.to_string(),
                        TitleSource::ActionName,
                        group.name(lang).to_string(),
                    )
                }
                None => (
                    name.clone(),
                    TitleSource::RawAction,
                    group_by_prefix(&name).name(lang).to_string(),
                ),
            }
        }
    }
}

/// Lowercase concatenation of title, keys, actions and group (SPEC §6).
pub fn search_text(bind: &Bind) -> String {
    let mut parts: Vec<String> = vec![bind.title.clone(), bind.keys_raw.clone()];
    parts.push(bind.keys_display.join(" "));
    for action in &bind.actions {
        parts.push(action.describe());
    }
    parts.push(bind.group.clone());
    parts.join(" ").to_lowercase()
}

/// Fill title, title_source, group and search for a parsed bind.
pub fn finalize(bind: &mut Bind, lang: Lang) {
    let (title, title_source, group) = title_and_group(bind, lang);
    bind.title = title;
    bind.title_source = title_source;
    bind.group = group;
    bind.search = search_text(bind);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Action;
    use crate::testutil::{builtin, sample_bind, spawn};

    fn bind_with(actions: Vec<Action>) -> Bind {
        Bind {
            actions,
            // The sample's overlay title would hijack the title priority.
            overlay_title_present: false,
            overlay_title: None,
            ..sample_bind()
        }
    }

    #[test]
    fn overlay_title_has_top_priority() {
        let mut bind = bind_with(vec![builtin("close-window")]);
        bind.overlay_title_present = true;
        bind.overlay_title = Some("Случайные обои".into());
        finalize(&mut bind, Lang::Ru);
        assert_eq!(bind.title, "Случайные обои");
        assert_eq!(bind.title_source, TitleSource::OverlayTitle);
    }

    #[test]
    fn dictionary_action_title_and_group_in_both_locales() {
        let mut bind = bind_with(vec![builtin("close-window")]);
        finalize(&mut bind, Lang::Ru);
        assert_eq!(bind.title, "Закрыть окно");
        assert_eq!(bind.title_source, TitleSource::ActionName);
        assert_eq!(bind.group, "Окна");

        let mut bind = bind_with(vec![builtin("close-window")]);
        finalize(&mut bind, Lang::En);
        assert_eq!(bind.title, "Close window");
        assert_eq!(bind.group, "Windows");
    }

    #[test]
    fn unknown_builtin_falls_back_to_raw_name() {
        let mut bind = bind_with(vec![builtin("some-custom-action")]);
        finalize(&mut bind, Lang::Ru);
        assert_eq!(bind.title, "some-custom-action");
        assert_eq!(bind.title_source, TitleSource::RawAction);
        assert_eq!(bind.group, "Прочее");
        finalize(&mut bind, Lang::En);
        assert_eq!(bind.group, "Other");
    }

    #[test]
    fn prefix_rules_apply_when_exact_name_is_unknown() {
        let mut bind = bind_with(vec![builtin("focus-column-foo")]);
        finalize(&mut bind, Lang::Ru);
        assert_eq!(bind.title, "focus-column-foo");
        assert_eq!(bind.group, "Навигация");

        let mut bind = bind_with(vec![builtin("move-column-foo")]);
        finalize(&mut bind, Lang::Ru);
        assert_eq!(bind.group, "Окна");
    }

    #[test]
    fn workspace_prefix_beats_generic_focus_prefix() {
        let mut bind = bind_with(vec![builtin("focus-workspace-in-area")]);
        finalize(&mut bind, Lang::Ru);
        assert_eq!(bind.group, "Воркспейсы");
    }

    #[test]
    fn spawn_title_is_command_line() {
        let mut bind = bind_with(vec![spawn(&["kitty"])]);
        finalize(&mut bind, Lang::Ru);
        assert_eq!(bind.title, "kitty");
        assert_eq!(bind.title_source, TitleSource::Command);
        assert_eq!(bind.group, "Программы");

        let mut bind = bind_with(vec![spawn(&["noctalia", "msg", "wallpaper-random"])]);
        finalize(&mut bind, Lang::En);
        assert_eq!(bind.group, "System");
    }

    #[test]
    fn grouping_is_deterministic() {
        let mut first = bind_with(vec![builtin("focus-workspace-up")]);
        let mut second = bind_with(vec![builtin("focus-workspace-up")]);
        finalize(&mut first, Lang::Ru);
        finalize(&mut second, Lang::Ru);
        assert_eq!(first.group, second.group);
        assert_eq!(first.search, second.search);
    }

    #[test]
    fn search_includes_title_keys_actions_and_group() {
        let mut bind = bind_with(vec![spawn(&["noctalia", "msg", "wallpaper-random"])]);
        bind.overlay_title_present = true;
        bind.overlay_title = Some("Случайные обои".into());
        finalize(&mut bind, Lang::Ru);
        assert_eq!(
            bind.search,
            "случайные обои mod+shift+w mod shift w spawn noctalia msg wallpaper-random система"
        );
    }

    #[test]
    fn dictionary_membership_is_locale_independent() {
        assert!(is_dictionary_action("close-window"));
        assert!(!is_dictionary_action("some-custom-action"));
    }
}
