# Tasks

## 1. Ядро l10n

- [x] 1.1 Создать `src/l10n.rs`: `enum Lang { En, Ru }`, `detect()` (`LC_ALL` → `LC_MESSAGES` → `LANG`, подтег до `_`/`-`/`.`, `C`/`POSIX`/пусто/неизвестно → `En`), глобальный `OnceLock<Lang>` + `init()`; юнит-тесты определения на таблице значений (`ru_RU.UTF-8`→Ru, `en_US`→En, `de_DE`→En, `C`→En, пусто→En, приоритет `LC_ALL` над `LANG`) — `cargo test l10n`
- [x] 1.2 Завести каталог: `enum Msg` и `tr(Lang, Msg) -> &'static str` для простых строк, типизированные функции для подстановок (счётчик, диагностика с `{path}`/`{id}`/`{code}`/`{launcher}` и т.п.); наполнить переводами ru/en для всех поверхностей из proposal (оверлей, CLI help, диагностика `lib.rs`/`exec.rs`/`fuzzel.rs`/`ui/app.rs`, находки аудита); проверить, что при новом варианте `Msg` без перевода сборка падает

## 2. Словарь групп и заголовков

- [x] 2.1 Перевести `groups.rs` на схему D4: `enum Group`, таблица `ACTION_TITLES: &[(&str, Group, &str en, &str ru)]`, `group_by_prefix`/`spawn_group` возвращают `Group`, `group_name(Group, Lang)`; резолюция title/group принимает `Lang`; юнит-тест: `close-window` при `En` → «Close window»/группа окон, при `Ru` → «Закрыть окно»/«Окна» — `cargo test groups`
- [x] 2.2 Починить зависимые тесты (`items.rs`, `parser.rs`, интеграционные) явной фиксацией `Lang` в вызовах, не трогая env; `cargo test` зелёный

## 3. Пользовательские поверхности

- [x] 3.1 Оверлей `ui/app.rs`: плейсхолдер поиска, счётчик, футер, заголовок шпаргалки и error-контексты через `l10n` (язык из глобального `init()`); запуск `kbinds overlay --dry-run` при `LANG=ru_RU.UTF-8` показывает русский UI, при `LC_ALL=en_US.UTF-8` — английский
- [x] 3.2 Диагностика `lib.rs`/`exec.rs`/`fuzzel.rs`: `bail!`/`.context`/`eprintln!` через `l10n`; тест: `exec несуществующий-id` при `en` даёт английское сообщение в stderr, код 2
- [x] 3.3 Находки `audit.rs`: сообщения правил A1–A3 и сводки через `l10n` с явным `Lang`; тест: находка A2 на фикстуре при `En` и `Ru` даёт одинаковый id правила и уровень, разные языки текста — `cargo test audit`
- [x] 3.4 CLI help в `main.rs`: `init()` в начале `main`, `l10n::apply_cli_texts(&mut cmd, lang)` перед `get_matches` (about корня, подкоманд, help флагов); doc-comments привести к внутренним описаниям; прогон `kbinds --help` при `ru_RU.UTF-8` и `en_US.UTF-8` показывает соответствующие тексты

## 4. Интеграционная приёмка

- [x] 4.1 Прогон всего CLI при двух локалях: `list`, `list --json` (схема полей и `id` идентичны, `title`/`group` дефолтов различаются), `audit`, `audit --strict` (код 1 не зависит от языка), `dmenu` с fuzzel (промпт `binds> `, формат пунктов не изменились), `overlay` — чек-лист по сценариям спек зафиксирован в итоге задачи
- [x] 4.2 `cargo test` и `cargo clippy` без регрессий; полный набор сценариев дельт-спек (`localization`, `center-overlay`, `binds-audit`) проходит вручную либо автотестами

## Workflow follow-up

- Архивация изменения и синк дельт в основные спеки — отдельным запросом через `/opsx:archive`.
