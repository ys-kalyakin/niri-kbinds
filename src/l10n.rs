//! Locale selection and message catalogs (capability `localization`).
//!
//! The process language is fixed at start from the environment
//! (`LC_ALL` → `LC_MESSAGES` → `LANG`); empty, `C`/`POSIX` and unknown
//! locales fall back to English. Simple messages live in `Msg` + `tr` —
//! both translations share one match arm, so a missing translation fails
//! to compile. Parameterized messages are typed functions (dynamic
//! `format!` arguments need literal format strings, so templates would
//! not work). Core functions that build user-visible strings take `Lang`
//! explicitly (deterministic tests without env races); UI and CLI paths
//! read the process global set by [`init`].

use std::sync::atomic::{AtomicU8, Ordering};

/// UI language of a process run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Lang {
    En = 0,
    Ru = 1,
}

const EN: u8 = Lang::En as u8;
const RU: u8 = Lang::Ru as u8;

/// Process language, English until [`init`] runs (tests never call it and
/// stay deterministic without touching the environment).
static LANG: AtomicU8 = AtomicU8::new(EN);

/// Fix the process language from the environment. Called once at the very
/// start of every binary's `main`.
pub fn init() {
    LANG.store(detect() as u8, Ordering::Relaxed);
}

/// The process language (`En` before `init`).
pub fn current() -> Lang {
    match LANG.load(Ordering::Relaxed) {
        RU => Lang::Ru,
        _ => Lang::En,
    }
}

/// First non-empty of `LC_ALL`, `LC_MESSAGES`, `LANG`; otherwise English.
fn detect() -> Lang {
    for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = std::env::var(key)
            && !value.trim().is_empty()
        {
            return parse_lang(&value);
        }
    }
    Lang::En
}

/// Language of one locale value: the subtag before `_`, `-` or `.`;
/// `C`/`POSIX`/empty and unknown subtags are English (`ru_RU.UTF-8` → Ru).
fn parse_lang(value: &str) -> Lang {
    let value = value.trim();
    if matches!(value, "" | "C" | "POSIX") {
        return Lang::En;
    }
    let language = value
        .split('.')
        .next()
        .unwrap_or("")
        .split(['_', '-'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match language.as_str() {
        "ru" => Lang::Ru,
        _ => Lang::En,
    }
}

/// Simple (parameter-free) user-facing messages; both translations in one
/// arm — a missing translation does not compile.
#[derive(Debug, Clone, Copy)]
pub enum Msg {
    // Overlay UI (SPEC center-overlay, «Карточка „Центр“»).
    SearchPlaceholder,
    FooterHints,
    CheatsheetHeader,
    // Overlay boot diagnostics.
    WaylandSessionNotFound,
    FocusedOutputFail,
    NiriMsgLaunchFail,
    NiriMsgBadFormat,
    // Exec diagnostics.
    EmptyCommand,
    FallbackNotPossible,
    DirectFallbackUnixOnly,
    // CLI diagnostics.
    HomeNotSet,
    EmptyConfig,
    ErrorPrefix,
    // Audit report.
    NoFindings,
    LevelError,
    LevelWarn,
    LevelInfo,
    // CLI help: `kbinds` root and subcommands.
    CliAbout,
    CmdList,
    CmdExec,
    CmdDmenu,
    CmdAudit,
    CmdOverlay,
    // CLI help: arguments.
    FlagConfig,
    FlagJson,
    FlagDryRun,
    FlagStrict,
    FlagLauncher,
    FlagMode,
    FlagFontScale,
    ArgId,
    // CLI help: the `kbinds-overlay` binary.
    OverlayBinAbout,
}

/// Translation of a simple message.
pub fn tr(lang: Lang, msg: Msg) -> &'static str {
    match (msg, lang) {
        (Msg::SearchPlaceholder, Lang::Ru) => "Поиск…",
        (Msg::SearchPlaceholder, Lang::En) => "Search…",
        (Msg::FooterHints, Lang::Ru) => "↑↓ выбор · Enter выполнить · Esc закрыть",
        (Msg::FooterHints, Lang::En) => "↑↓ select · Enter execute · Esc close",
        (Msg::CheatsheetHeader, Lang::Ru) => "Важные клавиши",
        (Msg::CheatsheetHeader, Lang::En) => "Important keys",
        (Msg::WaylandSessionNotFound, Lang::Ru) => "kbinds-overlay: не найдена Wayland-сессия",
        (Msg::WaylandSessionNotFound, Lang::En) => "kbinds-overlay: no Wayland session found",
        (Msg::FocusedOutputFail, Lang::Ru) => {
            "kbinds-overlay: не удалось определить фокусный монитор"
        }
        (Msg::FocusedOutputFail, Lang::En) => {
            "kbinds-overlay: failed to determine the focused monitor"
        }
        (Msg::NiriMsgLaunchFail, Lang::Ru) => "запуск `niri msg` не удался",
        (Msg::NiriMsgLaunchFail, Lang::En) => "failed to run `niri msg`",
        (Msg::NiriMsgBadFormat, Lang::Ru) => {
            "`niri msg --json focused-output` вернул неожиданный формат"
        }
        (Msg::NiriMsgBadFormat, Lang::En) => {
            "`niri msg --json focused-output` returned an unexpected format"
        }
        (Msg::EmptyCommand, Lang::Ru) => "пустая команда",
        (Msg::EmptyCommand, Lang::En) => "empty command",
        (Msg::FallbackNotPossible, Lang::Ru) => "fallback для builtin-экшена невозможен без niri",
        (Msg::FallbackNotPossible, Lang::En) => {
            "fallback for builtin actions is not possible without niri"
        }
        (Msg::DirectFallbackUnixOnly, Lang::Ru) => "прямой fallback реализован только на unix",
        (Msg::DirectFallbackUnixOnly, Lang::En) => "direct fallback is only implemented on unix",
        (Msg::HomeNotSet, Lang::Ru) => "HOME не задан; укажите --config PATH",
        (Msg::HomeNotSet, Lang::En) => "HOME is not set; pass --config PATH",
        (Msg::EmptyConfig, Lang::Ru) => "конфиг не содержит биндов",
        (Msg::EmptyConfig, Lang::En) => "the config contains no binds",
        (Msg::ErrorPrefix, Lang::Ru) => "ошибка",
        (Msg::ErrorPrefix, Lang::En) => "error",
        (Msg::NoFindings, Lang::Ru) => "Находок нет.",
        (Msg::NoFindings, Lang::En) => "No findings.",
        (Msg::LevelError, Lang::Ru) => "error",
        (Msg::LevelError, Lang::En) => "error",
        (Msg::LevelWarn, Lang::Ru) => "предупреждение",
        (Msg::LevelWarn, Lang::En) => "warning",
        (Msg::LevelInfo, Lang::Ru) => "инфо",
        (Msg::LevelInfo, Lang::En) => "info",
        (Msg::CliAbout, Lang::Ru) => "Интерактивная шпаргалка биндов niri",
        (Msg::CliAbout, Lang::En) => "Interactive cheatsheet for niri key binds",
        (Msg::CmdList, Lang::Ru) => "Список биндов в человекочитаемом виде",
        (Msg::CmdList, Lang::En) => "List binds in human-readable form",
        (Msg::CmdExec, Lang::Ru) => "Выполнить бинд по id",
        (Msg::CmdExec, Lang::En) => "Execute a bind by id",
        (Msg::CmdDmenu, Lang::Ru) => {
            "Интерактивный поиск по биндам через dmenu-совместимый лаунчер"
        }
        (Msg::CmdDmenu, Lang::En) => {
            "Interactive search over binds via a dmenu-compatible launcher"
        }
        (Msg::CmdAudit, Lang::Ru) => {
            "Статический анализ: конфликты комбо и бинды, невидимые в штатном оверлее"
        }
        (Msg::CmdAudit, Lang::En) => {
            "Static analysis: combo conflicts and binds invisible in the hotkey overlay"
        }
        (Msg::CmdOverlay, Lang::Ru) => "GUI-оверлей «Центр» поверх биндов",
        (Msg::CmdOverlay, Lang::En) => "GUI overlay «Центр» over the binds",
        (Msg::FlagConfig, Lang::Ru) => {
            "Путь к конфигу niri (по умолчанию: ~/.config/niri/config.kdl)"
        }
        (Msg::FlagConfig, Lang::En) => {
            "Path to the niri config (default: ~/.config/niri/config.kdl)"
        }
        (Msg::FlagJson, Lang::Ru) => "Вывести машиночитаемый JSON",
        (Msg::FlagJson, Lang::En) => "Output machine-readable JSON",
        (Msg::FlagDryRun, Lang::Ru) => "Печатать команду(ы) без выполнения",
        (Msg::FlagDryRun, Lang::En) => "Print the command(s) without executing anything",
        (Msg::FlagStrict, Lang::Ru) => "Код выхода 1 при наличии находок уровня error",
        (Msg::FlagStrict, Lang::En) => "Exit with code 1 when error-level findings are present",
        (Msg::FlagLauncher, Lang::Ru) => "Используемый бинарник лаунчера",
        (Msg::FlagLauncher, Lang::En) => "Launcher binary to use",
        (Msg::FlagMode, Lang::Ru) => "Режим оверлея (реализован только `center`)",
        (Msg::FlagMode, Lang::En) => "Overlay mode (only `center` is implemented)",
        (Msg::FlagFontScale, Lang::Ru) => {
            "Масштаб шрифта оверлея (перекрывает токен `font_scale` из кэша темы; 0.5–3.0)"
        }
        (Msg::FlagFontScale, Lang::En) => {
            "Font scale of the overlay text (overrides the theme-cache `font_scale` token; clamped to 0.5–3.0)"
        }
        (Msg::ArgId, Lang::Ru) => "Id бинда, как показано в `kbinds list --json`",
        (Msg::ArgId, Lang::En) => "Bind id, as shown by `kbinds list --json`",
        (Msg::OverlayBinAbout, Lang::Ru) => "GUI-шпаргалка биндов niri",
        (Msg::OverlayBinAbout, Lang::En) => "GUI cheatsheet overlay for niri key binds",
    }
}

/// Overlay list counter: «N из M биндов» / «N of M binds».
pub fn counter(lang: Lang, visible: usize, total: usize) -> String {
    match lang {
        Lang::Ru => format!("{visible} из {total} биндов"),
        Lang::En => format!("{visible} of {total} binds"),
    }
}

/// Config parse context with the file path.
pub fn config_parse(lang: Lang, path: &std::path::Path) -> String {
    match lang {
        Lang::Ru => format!("разбор конфига {}", path.display()),
        Lang::En => format!("parsing config {}", path.display()),
    }
}

/// `niri msg` exited with a non-zero code.
pub fn niri_msg_exit_code(lang: Lang, code: i32) -> String {
    match lang {
        Lang::Ru => format!("`niri msg` завершился с кодом {code}"),
        Lang::En => format!("`niri msg` exited with code {code}"),
    }
}

/// No bind with the requested id.
pub fn bind_not_found(lang: Lang, id: &str) -> String {
    match lang {
        Lang::Ru => format!("бинд с id `{id}` не найден (см. kbinds list --json)"),
        Lang::En => format!("bind with id `{id}` not found (see kbinds list --json)"),
    }
}

/// Only fuzzel is supported as the launcher.
pub fn only_fuzzel(lang: Lang, launcher: &str) -> String {
    match lang {
        Lang::Ru => format!("поддерживается только fuzzel (получено: {launcher})"),
        Lang::En => format!("only fuzzel is supported (got: {launcher})"),
    }
}

/// Launcher process failed to start.
pub fn launcher_launch_failed(lang: Lang, launcher: &str) -> String {
    match lang {
        Lang::Ru => format!("запуск {launcher} не удался"),
        Lang::En => format!("failed to launch {launcher}"),
    }
}

/// The launcher's selection does not map back to a bind.
pub fn choice_not_matched(lang: Lang, choice: &str) -> String {
    match lang {
        Lang::Ru => format!("выбор не найден в списке биндов: {choice}"),
        Lang::En => format!("selection not found among binds: {choice}"),
    }
}

/// `niri msg` could not be launched for an action.
pub fn exec_launch_failed(lang: Lang, cmd: &str) -> String {
    match lang {
        Lang::Ru => format!("запуск {cmd} не удался (niri установлен и сессия активна?)"),
        Lang::En => format!("failed to run {cmd} (is niri installed and the session active?)"),
    }
}

/// An action command exited non-zero.
pub fn exec_exit_code(lang: Lang, code: i32, cmd: &str) -> String {
    match lang {
        Lang::Ru => format!("команда завершилась с кодом {code}: {cmd}"),
        Lang::En => format!("command exited with code {code}: {cmd}"),
    }
}

/// niri is unavailable and a builtin action has no direct fallback.
pub fn niri_unavailable_builtin(lang: Lang, cmd: &str) -> String {
    match lang {
        Lang::Ru => format!("niri недоступен, builtin-экшен выполнить нечем: {cmd}"),
        Lang::En => format!("niri is unavailable, a builtin action cannot be run: {cmd}"),
    }
}

/// niri is unavailable; running the command directly instead.
pub fn niri_unavailable_direct(lang: Lang, cause: &str) -> String {
    match lang {
        Lang::Ru => format!("niri недоступен ({cause}), выполняю напрямую"),
        Lang::En => format!("niri unavailable ({cause}), running directly"),
    }
}

/// Direct (detached) spawn of a program failed.
pub fn direct_spawn_failed(lang: Lang, program: &str) -> String {
    match lang {
        Lang::Ru => format!("прямой запуск {program} не удался"),
        Lang::En => format!("direct spawn of {program} failed"),
    }
}

/// Audit A1: one combo assigned to different commands.
pub fn a1_message(lang: Lang, combo: &str, count: usize) -> String {
    match lang {
        Lang::Ru => format!("Комбо {combo} назначено {count} раз с разными командами:"),
        Lang::En => format!("Combo {combo} is assigned {count} times with different commands:"),
    }
}

/// Audit A2: a bind invisible in the stock hotkey overlay.
pub fn a2_message(lang: Lang, keys: &str, detail: &str) -> String {
    match lang {
        Lang::Ru => {
            format!("Бинд {keys} не виден в штатном оверлее (нет hotkey-overlay-title): {detail}")
        }
        Lang::En => format!(
            "Bind {keys} is invisible in the stock overlay (no hotkey-overlay-title): {detail}"
        ),
    }
}

/// Audit A3: a common niri action with no bind.
pub fn a3_message(lang: Lang, name: &str) -> String {
    match lang {
        Lang::Ru => format!("Распространённый экшен niri без бинда: {name}"),
        Lang::En => format!("Common niri action with no bind: {name}"),
    }
}

/// Subcommand texts: (name, about, [(arg id, arg help)]).
type SubTexts = [(&'static str, Msg, &'static [(&'static str, Msg)])];

/// Localized help texts for the `kbinds` CLI: applied to the derived
/// command before parsing (design D5 — derive keeps the structure, l10n
/// owns the display strings).
pub fn apply_cli_texts(cmd: &mut clap::Command, lang: Lang) {
    let mut owned = std::mem::take(cmd);
    owned = owned.about(tr(lang, Msg::CliAbout));
    owned = owned.mut_arg("config", |arg| arg.help(tr(lang, Msg::FlagConfig)));
    let texts: &SubTexts = &[
        ("list", Msg::CmdList, &[("json", Msg::FlagJson)]),
        (
            "exec",
            Msg::CmdExec,
            &[("dry_run", Msg::FlagDryRun), ("id", Msg::ArgId)],
        ),
        ("dmenu", Msg::CmdDmenu, &[("launcher", Msg::FlagLauncher)]),
        (
            "audit",
            Msg::CmdAudit,
            &[("json", Msg::FlagJson), ("strict", Msg::FlagStrict)],
        ),
        (
            "overlay",
            Msg::CmdOverlay,
            &[
                ("dry_run", Msg::FlagDryRun),
                ("mode", Msg::FlagMode),
                ("font_scale", Msg::FlagFontScale),
            ],
        ),
    ];
    // Mutate subcommands in place: `about`/`mut_arg` are consuming builder
    // methods, so take each slot, rebuild and write it back through the
    // same `&mut` (a lookup by name after `mem::take` would miss the slot —
    // its name is empty until written back).
    for sub in owned.get_subcommands_mut() {
        let Some((_, about, args)) = texts.iter().find(|(name, _, _)| *name == sub.get_name())
        else {
            continue;
        };
        let mut taken = std::mem::take(sub);
        taken = taken.about(tr(lang, *about));
        for (arg_id, msg) in *args {
            let help = tr(lang, *msg);
            taken = taken.mut_arg(arg_id, |arg| arg.help(help));
        }
        *sub = taken;
    }
    *cmd = owned;
}

/// Localized help texts for the `kbinds-overlay` binary.
pub fn apply_overlay_cli_texts(cmd: &mut clap::Command, lang: Lang) {
    let mut owned = std::mem::take(cmd);
    owned = owned.about(tr(lang, Msg::OverlayBinAbout));
    for (arg_id, msg) in [
        ("config", Msg::FlagConfig),
        ("dry_run", Msg::FlagDryRun),
        ("mode", Msg::FlagMode),
        ("font_scale", Msg::FlagFontScale),
    ] {
        let help = tr(lang, msg);
        owned = owned.mut_arg(arg_id, |arg| arg.help(help));
    }
    *cmd = owned;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_lang_cases() {
        assert_eq!(parse_lang("ru_RU.UTF-8"), Lang::Ru);
        assert_eq!(parse_lang("ru"), Lang::Ru);
        assert_eq!(parse_lang("ru-RU"), Lang::Ru);
        assert_eq!(parse_lang("en_US.UTF-8"), Lang::En);
        assert_eq!(parse_lang("en"), Lang::En);
        assert_eq!(parse_lang("de_DE.UTF-8"), Lang::En, "unknown → fallback en");
        assert_eq!(parse_lang("C"), Lang::En);
        assert_eq!(parse_lang("C.UTF-8"), Lang::En);
        assert_eq!(parse_lang("POSIX"), Lang::En);
        assert_eq!(parse_lang(""), Lang::En);
    }

    #[test]
    fn default_language_is_english_without_init() {
        // Tests never touch the environment: the global stays En.
        assert_eq!(current(), Lang::En);
    }

    #[test]
    fn parameterized_messages_carry_their_values() {
        assert_eq!(counter(Lang::Ru, 3, 10), "3 из 10 биндов");
        assert_eq!(counter(Lang::En, 3, 10), "3 of 10 binds");
        assert_eq!(
            bind_not_found(Lang::Ru, "mod-t"),
            "бинд с id `mod-t` не найден (см. kbinds list --json)"
        );
        assert_eq!(
            exec_exit_code(Lang::En, 2, "niri msg action quit"),
            "command exited with code 2: niri msg action quit"
        );
    }
}
