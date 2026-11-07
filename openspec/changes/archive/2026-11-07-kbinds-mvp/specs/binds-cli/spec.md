# Spec Delta: binds-cli

## Purpose

CLI-интерфейс `kbinds`: команда `list` (включая машиночитаемый JSON) и команда `exec` (выполнение действий бинда через `niri msg action`, с `--dry-run`), единые коды выхода и обработка ошибок.

## ADDED Requirements

### Requirement: Команда list

`kbinds list` SHALL выводить список биндов из конфига. С флагом `--json` вывод SHALL быть валидным JSON-массивом объектов с полями: `id`, `title`, `title_source`, `actions` (с `kind`, `name`, `cmd`), `keys_raw`, `keys_display`, `combo`, `group`, `repeat`, `overlay_title_present`, `file`, `line`, `search`. Без `--json` вывод SHALL быть человекочитаемым (id, клавиши, заголовок, группа).

#### Scenario: Валидный JSON

- **WHEN** выполняется `kbinds list --json` на непустом конфиге
- **THEN** stdout парсится как JSON-массив; у каждого объекта заполнены `combo`, `keys_display`, `group`, `search`; запись отфильтровывается стандартным `jq`

#### Scenario: Сценарий фильтрации

- **WHEN** выполняется `kbinds list --json | jq '.[] | select(.group=="Воркспейсы")'`
- **THEN** вывод содержит только бинды группы «Воркспейсы»

### Requirement: Идентификатор бинда

Каждый бинд SHALL иметь стабильный `id` — kebab-case форму канонического комбо (например, `mod+shift+w` → `mod-shift-w`), уникальный среди всех биндов модели (дубли комбо получают различимый суффикс).

#### Scenario: Стабильность id между запусками

- **WHEN** `list --json` выполняется дважды на одном конфиге
- **THEN** id каждого бинда одинаковы; `exec` по id из вывода `list` находит тот же бинд

### Requirement: Команда exec

`kbinds exec <id>` SHALL выполнять все действия бинда по порядку: `builtin` — через `niri msg action <имя> <аргументы…>`; `spawn`/`spawn-sh` — через `niri msg action spawn -- <argv…>` (синтаксис проверен: `niri msg action spawn -- <COMMAND>...`). `--dry-run` SHALL печатать команды в stdout и не выполнять ничего. Запуск exec не SHALL требовать сеть.

#### Scenario: Builtin через niri msg

- **WHEN** выполняется `kbinds exec <id>` для бинда с `builtin`-действием
- **THEN** вызывается `niri msg action <имя> <аргументы…>`

#### Scenario: Spawn через niri msg spawn

- **WHEN** выполняется `kbinds exec <id>` для бинда с `spawn`-действием
- **THEN** вызывается `niri msg action spawn -- <argv…>` в точности как записано в конфиге

#### Scenario: Несколько действий по порядку

- **WHEN** бинд содержит несколько действий и выполняется `kbinds exec <id>`
- **THEN** действия выполняются в порядке следования в конфиге

#### Scenario: Dry-run ничего не выполняет

- **WHEN** выполняется `kbinds exec mod-t --dry-run`
- **THEN** в stdout печатается команда(ы) в точном виде запуска, побочных эффектов нет, код выхода 0

### Requirement: Переопределение пути конфига

Все команды SHALL принимать `--config PATH` (по умолчанию `~/.config/niri/config.kdl`).

#### Scenario: Разбор фикстуры

- **WHEN** выполняется команда с `--config tests/fixtures/edge.kdl`
- **THEN** разбор идёт по указанному файлу, `file` в модели указывает на него

### Requirement: Коды выхода и диагностика

Коды выхода: `0` — успех; `1` — находки audit при `--strict`; `2` — ошибка (IO, парсинг, неверное использование, неизвестный id в `exec`). Диагностика SHALL выводиться в stderr; stdout SHALL содержать только полезный вывод команды.

#### Scenario: Ошибка парсинга

- **WHEN** конфиг не существует или не парсится
- **THEN** код выхода 2, сообщение об ошибке в stderr, stdout пуст

#### Scenario: Неизвестный id

- **WHEN** выполняется `kbinds exec no-such-id`
- **THEN** код выхода 2, сообщение в stderr перечисляет, что бинд не найден
