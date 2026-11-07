# Tasks

## 1. Pure-логика и геометрия (src/ui/items.rs)

- [x] 1.1 Добавить `cheatsheet_binds(&[Bind]) -> Vec<usize>` — индексы биндов с `overlay_title_present == true` в порядке конфига; юнит-тесты: выборка из смешанного набора, пустой конфиг, порядок совпадает с конфигом; `cargo test` зелёный
- [x] 1.2 Расширить `card_rect(window)` → `card_rect(window, has_cheatsheet: bool)` (900 при true, 600 при false; кап 92% и высота без изменений) и `is_outside_card` тем же параметром; обновить существующие тесты геометрии/хит-теста на оба значения флага; `cargo test` зелёный

## 2. Вёрстка и состояние (src/ui/app.rs)

- [x] 2.1 Поле `cheatsheet: Vec<usize>` в `Overlay`, заполняется в `boot` через `items::cheatsheet_binds`; `update()` панель не трогает (нет новых `Message`-вариантов); `cargo clippy -- -D warnings` и `cargo build` чисты
- [x] 2.2 Вёрстка по design D2/D3: карточка `row![левая колонка, разделитель 1px (outline_variant), панель]`, ширина 900 (кап 92%) или 600 без панели; панель — заголовок «Важные клавиши» (12px, вторичный цвет) + `scrollable` (id `kbinds-overlay-cheatsheet`) с компактными строками «title + keycaps» (4px вертикальные паддинги) из `items::keycaps`; `cargo build` чист
- [x] 2.3 Панель read-only: строки без `mouse_area` и подсветки выделения, клик по строке панели не выполняет действие и не закрывает оверлей (клик внутри карточки), `↑↓`/Enter двигают только левый список; проверить `--dry-run`-прогоном на живой сессии и скриншотом
- [x] 2.4 Прокрутка независима: колесо над панелью крутит панель, над списком — список, «клик вне карточки» с шириной 900 закрывается корректно (хит-тест из 1.2); живой прогон

## 3. Проверки и документы

- [x] 3.1 Полный гейт: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` — чисто; регресс MVP1-команд не задет (`kbinds list --json | wc -l` без изменений)
- [x] 3.2 Живые сценарии спеки на сессии: панель показывает ровно бинды с `hotkey-overlay-title` (16 на живом конфиге, кириллица символами), фильтр «воркспейс» панель не меняет, длинная панель прокручивается; скриншот для README
- [x] 3.3 README: раздел Overlay — абзац про правую панель (состав = `hotkey-overlay-title`, read-only) и новый скриншот; `kbinds-overlay`/`kbinds` переустановлены (`cargo install --path .`, обход rustup-прокси целевой машины через PATH toolchains)
- [x] 3.4 `openspec validate kbinds-overlay-cheatsheet --strict` — чисто (после архивации `kbinds-overlay-mvp2`, см. follow-up)

## Workflow follow-up

- Перед apply этого change заархивировать `kbinds-overlay-mvp2` (`/opsx:archive`) — дельта «MODIFIED» ссылается на capability `center-overlay`, которая попадает в main-спеки при архивации.
- После приёмки заархивировать этот change (`/opsx:archive`) и убедиться, что `center-overlay` в main-спеках содержит изменённые требования.
