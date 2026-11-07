//! Integration tests over the fixtures and (ignored, manual) the live config.

use std::path::{Path, PathBuf};
use std::process::Command;

use niri_kbinds::model::ActionKind;
use niri_kbinds::parser::parse_config;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn edge_fixture_covers_all_spec_cases() {
    let binds = parse_config(&fixture("edge.kdl")).expect("edge.kdl must parse");
    // 24 кириллических + 14 из edge.kdl + 2 из include.
    assert_eq!(binds.len(), 40);

    // Бинды из include приписаны своему файлу и строкам.
    let inc = binds
        .iter()
        .find(|b| b.keys_raw == "Mod+I")
        .expect("include bind present");
    assert!(inc.file.ends_with("edge-inc.kdl"));
    assert_eq!(inc.line, 3);

    // Кириллица отображается символом, исходник сохранён.
    let ve = binds
        .iter()
        .find(|b| b.keys_raw == "Mod+Cyrillic_ve")
        .expect("cyrillic bind");
    assert_eq!(ve.keys_display, vec!["Mod".to_string(), "в".to_string()]);
    let slash = binds
        .iter()
        .find(|b| b.keys_raw == "Mod+Cyrillic_slash")
        .expect("slash-name bind");
    assert_eq!(
        slash.keys_display,
        vec!["Mod".to_string(), "Cyrillic_slash".to_string()]
    );

    // Дублированный combo получает различимые id в порядке конфига.
    let dupes: Vec<&str> = binds
        .iter()
        .filter(|b| b.combo == "mod+d")
        .map(|b| b.id.as_str())
        .collect();
    assert_eq!(dupes, ["mod-d", "mod-d-2"]);

    // Атрибуты, spawn-формы, XF86, комментарии.
    let q = binds.iter().find(|b| b.keys_raw == "Mod+Q").unwrap();
    assert!(!q.repeat);
    let space = binds.iter().find(|b| b.keys_raw == "Mod+Space").unwrap();
    assert_eq!(space.title, "Switch layout");
    let k = binds.iter().find(|b| b.keys_raw == "Mod+K").unwrap();
    assert_eq!(k.actions[0].kind, ActionKind::SpawnSh);
    let mute = binds
        .iter()
        .find(|b| b.keys_raw == "XF86AudioMute")
        .expect("XF86 tolerated");
    assert!(mute.raw.contains("allow-when-locked=true"));
    let m = binds.iter().find(|b| b.keys_raw == "Mod+M").unwrap();
    assert!(m.raw.contains("cooldown-ms=100"));
    let c = binds.iter().find(|b| b.keys_raw == "Mod+C").unwrap();
    assert_eq!(c.actions.len(), 1);
    assert!(c.raw.contains("condition"));
    // Строки внутри одного файла считаются корректно.
    let left = binds.iter().find(|b| b.keys_raw == "Mod+Left").unwrap();
    assert_eq!(left.line, 47);
}

#[test]
fn cyclic_include_fixture_is_an_error() {
    let err = parse_config(&fixture("cycle-a.kdl")).expect_err("cycle must fail");
    assert!(format!("{err:#}").contains("cyclic"));
}

/// A real (non-pipe) stdout write failure must surface as exit code 2 —
/// not be silently swallowed as "downstream closed the pipe" (binds-cli
/// exit codes). `/dev/full` accepts nothing and fails every write.
#[test]
#[cfg(target_os = "linux")]
fn list_stdout_write_failure_exits_2() {
    let dev_full = std::fs::File::create("/dev/full").expect("/dev/full exists");
    let status = Command::new(env!("CARGO_BIN_EXE_kbinds"))
        .args(["list", "--json"])
        .stdout(dev_full)
        .status()
        .expect("spawn kbinds");
    assert_eq!(
        status.code(),
        Some(2),
        "real stdout write failure must exit 2"
    );
}

/// `kbinds list --json | head -1`: head closes the pipe early; the broken
/// pipe stays a quiet success (exit 0, no panic, no error message).
#[test]
fn list_broken_pipe_is_quiet_success() {
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "{} list --json | head -1",
            env!("CARGO_BIN_EXE_kbinds")
        ))
        .status()
        .expect("run under sh");
    assert_eq!(status.code(), Some(0), "broken pipe must stay a success");
}

fn grep_count(path: &Path, pattern: &str) -> usize {
    let out = Command::new("grep")
        .args(["-cE", pattern])
        .arg(path)
        .output()
        .expect("grep available");
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .expect("grep count is a number")
}

/// Арбитр полноты ТЗ §7.3 на живом конфиге. Запуск вручную:
/// `cargo test -- --ignored`
///
/// В живом конфиге 159 биндов: grep ТЗ (`^\s*Mod` = 144) не видит 15 биндов
/// (10 XF86, Alt+Tab, Print ×3, Ctrl+Print, Alt+Print, Ctrl+Alt+Delete).
/// Поэтому арбитр двусторонний: полный grep по клавишным префиксам конфига
/// для общего числа и буквальный критерий ТЗ для Mod-биндов.
#[test]
#[ignore = "требует живой конфиг niri (~/.config/niri/config.kdl)"]
fn live_config_parse_matches_grep_arbiter() {
    let home = std::env::var("HOME").expect("HOME set");
    let config = Path::new(&home).join(".config/niri/config.kdl");
    assert!(
        config.exists(),
        "live config not found: {}",
        config.display()
    );

    let binds = parse_config(&config).expect("live config parses");
    let total_binds = binds.len();

    // Полный эталон: все bind-строки (префиксы клавиш, реально встречающиеся
    // в этом конфиге).
    let all_grep = grep_count(&config, r"^\s*(Mod|Ctrl|Alt|Shift|Super|Print|XF86)");
    assert_eq!(
        total_binds, all_grep,
        "parser must see every bind line; if the config gained other key \
         prefixes, extend the grep pattern"
    );

    // Буквальный критерий ТЗ §7.3/§15.1: Mod-бинды = grep '^\s*Mod' (144).
    let mod_grep = grep_count(&config, r"^\s*Mod");
    let mod_binds = binds
        .iter()
        .filter(|b| b.keys_raw.starts_with("Mod"))
        .count();
    assert_eq!(
        mod_binds, mod_grep,
        "Mod-prefixed binds must match the SPEC arbiter"
    );
}
