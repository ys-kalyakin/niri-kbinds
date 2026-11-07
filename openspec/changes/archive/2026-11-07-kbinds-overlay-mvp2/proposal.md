# Proposal: kbinds-overlay-mvp2

## Why

MVP1 даёт данные и исполнение биндов (CLI, fuzzel-dmenu), но шпаргалка остаётся текстовой: пользователь должен читать список, а не видеть его. Утверждённый макет `mockup.html` («Центр») задаёт графический оверлей — layer-shell поверхность с поиском, группами и keycaps, закрывающая сценарий «посмотрел → нашёл → выполнил». GTK-стек недоступен (нет devel-пакетов, sudo закрыт), поэтому стек зафиксирован как `iced` + `iced_layershell` 0.19.1 (чистый Rust, только crates.io).

## What Changes

- Новый бинарь `kbinds-overlay` (второй `[[bin]]` в крейте `niri-kbinds`): `kbinds overlay [--config PATH] [--dry-run] [--mode center]`.
- Новая surface-часть UI: layer-shell слой `overlay`, keyboard interactivity `exclusive`, полноэкранное затемнение на фокусном мониторе (`niri msg focused-output --json` при старте).
- Карточка «Центр» по макету: поиск с автофокусом, сгруппированный список (группы и порядок — как MVP1 `groups`), keycaps из `keys_display`, счётчик «N из M биндов», футер-подсказка.
- Навигация: `↑/↓` циклично, `Enter` — выполнить и закрыть, `Esc` / клик вне карточки — закрыть, клик по строке — выполнить, прокрутка колесом.
- `--dry-run`: команда бинда печатается в **stderr** (в отличие от `kbinds exec --dry-run`, который печатает в stdout), ничего не выполняется, оверлей закрывается.
- Новые зависимости: `iced`, `iced_layershell` 0.19.1 (+ их дерево); системных devel-пакетов не требуется.
- README: установка, бинд `Mod+Shift+/` в niri, скриншот оверлея, отличия от `kbinds dmenu`.
- Fuzzel-адаптер и CLI-команды MVP1 не меняются.

## Capabilities

### New Capabilities

- `center-overlay`: графический оверлей «Центр» — layer-shell поверхность поверх рабочего стола с затемнением, карточкой поиска, сгруппированным списком биндов, клавиатурной и мышиной навигацией, выполнением выбранного бинда через MVP1 `exec` и режимом `--dry-run`.

### Modified Capabilities

(нет — поведение MVP1 (`binds-parsing`, `binds-cli`, `binds-audit`, `fuzzel-dmenu`) не меняется; `kbinds-overlay` переиспользует библиотечные модули как есть)

## Impact

- **Cargo.toml**: +2 зависимости (`iced`, `iced_layershell` 0.19.1), +1 `[[bin]]` `kbinds-overlay`.
- **Новый код**: `src/bin/overlay.rs` (аргументы, запуск UI), `src/ui/{app,items,theme}.rs` (состояние iced, pure-функции фильтрации, палитра).
- **Переиспользуется без изменений**: `parser`, `model`, `groups`, `exec` (`action_argv` + `display_command` — публичны; dry-run оверлея печатает в stderr, поэтому `exec_bind` напрямую не вызывается в dry-run-режиме), `keys_display`.
- **README.md**: раздел про оверлей + скриншот.
- **Конфиг niri пользователя**: бинд `Mod+Shift+/` (+ дубль `Mod+Shift+backslash` для ru-раскладки) → `spawn "kbinds" "overlay"`; по решению пользователя штатный `show-hotkey-overlay` переносится на свободный `Mod+Shift+U` (дубль комбо niri отверг бы целиком) (M5).
- **Риски**: API `iced_layershell` 0.19 (слой/якоря/exclusive-клавиатура/прозрачность фона) — арбитр этап M1; если крейт блокирует, вернуться с фактами и вариантами, не меняя стек молча.
