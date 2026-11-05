# Horizon TNT Work-In-Progress

## Русская версия

Этот каталог содержит незавершённый перенос идеи многоуровневого TNT-мода на Rust/WASM API Horizon. Это не готовый релиз и не официальный порт исходного Java-мода. Код, текстуры и таблицы эффектов написаны заново для Horizon.

## Что сделано

### 1. Отдельный Rust-мод

`mods/official/insane-tnt/` собирается как WASM component через `cargo-horizon-mod`.

Мод добавляет 42 типа зарядов, среди них:

- Small TNT и High Explosive TNT;
- Mini Nuke, Nuke, Mega Nuke, Ultra Nuke, Titan Nuke;
- Chungus Nuke, Kraken Nuke;
- World Dissolver и Ultra World Dissolver;
- Volcano, Storm, Lightning, Meteor, Anvil, Arrow;
- Wither, Crystal, Sponge, Ice, Slime, Virus и другие.

Каждый тип имеет:

- собственный item id в namespace `insane_tnt`;
- client item definition;
- generated 16x16 PNG-текстуру;
- item model;
- локализацию `en_us` и `ru_ru`;
- shapeless recipe;
- описание эффектов в `src/kinds.rs`.

Источником генерации контента является таблица в `tools/make_assets.py`. После изменения таблицы нужно перегенерировать assets и `src/kinds.rs`.

```powershell
python tools/make_assets.py
cargo run -p cargo-horizon-mod -- build
```

### 2. Большие эффекты

Мод создаёт эффекты через команды Horizon. Базовые ядерные типы создают много примированных TNT, огонь, молнии и область очистки. Команды выдаются порциями по тикам, чтобы не отправлять весь взрыв одной пачкой и не блокировать основной игровой цикл.

Верхние уровни намеренно опасны для мира. Они могут:

- вызвать сильный TPS/FPS лаг;
- создать тысячи сущностей TNT;
- уничтожить большую область;
- оставить мир в тяжёлом состоянии после перезапуска;
- перегрузить dedicated/LAN сервер.

Проверять их следует только на копии мира.

### 3. Поддержка TNT-template блоков

`crates/horizon-client/src/game/tnt_kinds.rs` и `game/mods.rs` позволяют считать блок, объявленный content-модом с `template = "minecraft:tnt"`, TNT-блоком для vanilla-пути зажигания.

Добавлена registry-проверка всех состояний такого блока. Это нужно, потому что один block id может иметь несколько block states.

### 4. Creative inventory

`Creative::add_mod_items` добавляет предметы content-модов:

- в Search tab;
- в существующую вкладку namespace, если она уже экспортирована;
- иначе в `Tools & Utilities`.

Добавлены unit tests для обоих случаев.

Это не создаёт новую native creative tab из WASM-мода. Текущий API Horizon пока не даёт модам полноценный declarative API для новых creative tabs. Поэтому отдельная вкладка `Insane TNT` ещё не является гарантированной частью публичного интерфейса.

### 5. Vanilla TNT 1000x

В `crates/horizon-client/src/game/tnt.rs` стандартный примированный `minecraft:tnt` теперь вызывает:

```rust
g.server_explode(center, 4_000.0, interaction, false)
```

Vanilla power равен `4.0`, поэтому `4000.0` означает 1000x по параметру explosion power. Это изменение относится к обычному vanilla TNT и не требует WASM-мода.

## Управление текущим модом

Текущий экспериментальный WASM-мод первоначально регистрировал клавишу `G` и открывал mod-owned UI. На практике этот путь зависит от:

- включённого mods runtime;
- корректной записи key binding;
- отсутствия другой обработки `G`;
- того, что игра находится в `Playing`, а не в inventory/title screen;
- корректного UI event bridge.

Клавиша `G` не должна считаться стабильным интерфейсом текущей версии. Для диагностики используйте команду:

```text
/tnt menu
```

Если UI API не работает, используйте:

```text
/tnt list
/tnt give nuke
/tnt place nuke
/tnt here nuke
```

## Как устроена установка

1. Собрать мод:

```powershell
cd mods/official/insane-tnt
cargo run -p cargo-horizon-mod -- build
```

2. Скопировать `target/horizon-mod/insane_tnt.hmod` в каталог игры:

```text
<game directory>/horizon-mods/insane_tnt.hmod
```

3. Включить мод в файле рядом с `options.txt`:

```text
mod.insane_tnt:true
```

4. Перезапустить игру после изменения content-мода.

5. При подозрении на устаревший overlay удалить только сгенерированный каталог:

```text
<game directory>/.horizon-cache/mod-content
```

Не удаляйте сохранения мира.

## Известные проблемы

### Чёрный экран/долгий запуск

Content-мод заставляет Horizon строить overlay ресурсов и заново загружать assets. Это может занимать десятки секунд и потреблять много памяти. Если overlay или renderer-path зависает, окно может выглядеть чёрным, хотя процесс ещё работает.

Что проверить:

- `logs/latest.log`;
- строки `first frame`, `resources loaded`, `content from`;
- нет ли второго клиента, который держит world lock;
- актуален ли `.horizon-cache/mod-content`;
- запускается ли vanilla client без `horizon-mods.txt`.

### Creative tab

Предметы попадают в Search/Tools & Utilities, но отдельная native tab не гарантирована. Нужен новый публичный API для регистрации creative tab с icon, title и item provider.

### G

Клавиша зависит от WASM input bridge и текущего экрана. Сейчас мод не должен использовать G как единственный способ получить предмет.

### Огромные взрывы

Создавать тысячи TNT entity через команды неэффективно. Это демонстрационный fallback, а не производственная реализация. Для настоящего world-scale TNT нужен server-side explosion scheduler, который:

- не создаёт тысячи сущностей;
- обрабатывает блоки чанками;
- ограничивает работу за тик;
- сохраняет очередь при паузе/перезапуске;
- синхронизирует разрушения с LAN clients;
- учитывает mobGriefing, drops и explosion interaction.

### Зажигание

Надёжный vanilla-путь существует для `minecraft:tnt` и TNT-template blocks через flint and steel/fire charge. Семантика item charge и mod-owned placed charge пока не полностью совпадает с Java-модом. Redstone, цепное зажигание и все edge cases нужно покрыть parity tests.

### Контент и модели

Для каждого content block нужны согласованные:

- registry declaration;
- blockstate JSON;
- model JSON;
- texture;
- item definition/model, если у блока есть item.

Отсутствие blockstate binding может привести к ошибке `unbound gallery block state` во время загрузки render gallery.

## Что нужно доделать community

Приоритет 1:

- сделать отдельную native creative tab через публичный API;
- стабилизировать `G` и UI event bridge;
- заменить тысячи TNT entities на серверный scheduler;
- добавить dedicated-server implementation;
- добавить integration test: place -> ignite -> fuse -> explosion;
- добавить тест запуска с включённым content-модом.

Приоритет 2:

- parity tests против Java reference для fuse, random push, block drops и chain priming;
- LAN/remote packet support для больших explosions;
- корректные item stack consumption в survival/creative;
- sound, particles и camera shake для каждого tier;
- конфиг power/radius/entity budget;
- предупреждение перед world-scale explosion.

Приоритет 3:

- рецепты и баланс;
- tag integration;
- полноценные translations;
- animated primed-charge rendering;
- save/load активных mod charges;
- profiling и benchmarks;
- ограничение команд и защита от случайного уничтожения spawn/production worlds.

## Проверки

Рабочие проверки текущего дерева:

```powershell
cargo run -p cargo-horizon-mod -- build
cargo run -p cargo-horizon-mod -- test
cargo test -p horizon-client --lib creative::tests::mod_items_join_their_namespace_tab_and_the_search_tab
cargo check -p horizon-client --bin horizon-client --features horizon-client/mods
```

Это проверяет сборку и API-контракты, но не доказывает, что каждый огромный взрыв безопасен или производителен в реальном мире.

---

## English Version

This directory contains an unfinished Rust/WASM reimplementation of the idea of a multi-tier TNT mod for Horizon. It is not a release-ready mod and is not an official port of the original Java mod. The code, tables, and textures were written from scratch for Horizon.

## What is implemented

### 1. Standalone Rust mod

`mods/official/insane-tnt/` builds as a WASM component through `cargo-horizon-mod`.

The mod defines 42 charge types, including:

- Small TNT and High Explosive TNT;
- Mini Nuke, Nuke, Mega Nuke, Ultra Nuke, Titan Nuke;
- Chungus Nuke and Kraken Nuke;
- World Dissolver and Ultra World Dissolver;
- Volcano, Storm, Lightning, Meteor, Anvil, and Arrow;
- Wither, Crystal, Sponge, Ice, Slime, Virus, and others.

Each type has:

- an item id in the `insane_tnt` namespace;
- a client item definition;
- a generated 16x16 PNG texture;
- an item model;
- `en_us` and `ru_ru` localization;
- a shapeless recipe;
- an effect description in `src/kinds.rs`.

The source of truth is the table in `tools/make_assets.py`. After editing that table, regenerate the assets and Rust effect table.

```powershell
python tools/make_assets.py
cargo run -p cargo-horizon-mod -- build
```

### 2. Large effects

The mod builds effects from Horizon commands. Nuclear tiers create many primed TNT entities, fire, lightning, and clearing operations. Commands are emitted in per-tick batches so one detonation does not submit the entire command queue in a single game tick.

The highest tiers are intentionally dangerous. They can:

- cause severe TPS/FPS lag;
- create thousands of TNT entities;
- destroy a large area;
- leave a world in a heavy state after restart;
- overload a dedicated or LAN server.

Only test them on a backup world.

### 3. TNT-template blocks

`crates/horizon-client/src/game/tnt_kinds.rs` and `game/mods.rs` allow a content-mod block declared with `template = "minecraft:tnt"` to be recognized by the vanilla ignition path.

All states of such a block are registered. This matters because one block id can have multiple block states.

### 4. Creative inventory

`Creative::add_mod_items` adds content-mod items:

- to the Search tab;
- to an existing namespace tab when one is exported;
- otherwise to `Tools & Utilities`.

Unit tests cover both placement cases.

This does not create a new native creative tab from a WASM mod. The current Horizon API does not yet expose a complete declarative API for registering new creative tabs. Therefore a dedicated `Insane TNT` tab is not guaranteed by the current public interface.

### 5. Vanilla TNT at 1000x

In `crates/horizon-client/src/game/tnt.rs`, the standard primed `minecraft:tnt` now calls:

```rust
g.server_explode(center, 4_000.0, interaction, false)
```

Vanilla TNT uses power `4.0`, so `4000.0` is a 1000x multiplier of the explosion-power parameter. This change applies to normal vanilla TNT and does not require the WASM mod.

## Current mod controls

The experimental WASM mod originally registered `G` and opened a mod-owned UI. In practice, this path depends on:

- the mods runtime being enabled;
- the key binding being persisted correctly;
- no other handler consuming `G`;
- the game being in `Playing`, not inventory/title screen;
- the UI event bridge being available.

`G` must not be treated as a stable interface in the current version. Use the command surface for diagnostics:

```text
/tnt menu
```

If the UI API is unavailable, use:

```text
/tnt list
/tnt give nuke
/tnt place nuke
/tnt here nuke
```

## Installation

1. Build the mod:

```powershell
cd mods/official/insane-tnt
cargo run -p cargo-horizon-mod -- build
```

2. Copy `target/horizon-mod/insane_tnt.hmod` into the game directory:

```text
<game directory>/horizon-mods/insane_tnt.hmod
```

3. Enable it beside `options.txt`:

```text
mod.insane_tnt:true
```

4. Restart the game after changing content-mod files.

5. If the generated overlay is stale, remove only:

```text
<game directory>/.horizon-cache/mod-content
```

Do not delete world saves.

## Known problems

### Black screen or very long startup

A content mod makes Horizon build a resource overlay and reload assets. This can take tens of seconds and consume substantial memory. If the overlay or renderer path stalls, the window can look black even while the process is still alive.

Check:

- `logs/latest.log`;
- `first frame`, `resources loaded`, and `content from` lines;
- whether another client holds the world lock;
- whether `.horizon-cache/mod-content` is stale;
- whether the vanilla client starts with `horizon-mods.txt` disabled.

### Creative tab

Items are inserted into Search/Tools & Utilities, but a dedicated native tab is not guaranteed. A public API for registering a creative tab with an icon, title, and item provider is still needed.

### G key

The key depends on the WASM input bridge and the current screen. The mod must not rely on G as its only item-acquisition path.

### Huge explosions

Creating thousands of TNT entities through commands is inefficient. This is a demonstration fallback, not a production implementation. A real world-scale TNT feature needs a server-side explosion scheduler that:

- does not create thousands of entities;
- processes blocks in chunk-sized budgets;
- limits work per tick;
- persists its queue across pause/restart;
- synchronizes destruction with LAN clients;
- honors mobGriefing, drops, and explosion interaction rules.

### Ignition

The reliable vanilla path exists for `minecraft:tnt` and TNT-template blocks through flint and steel/fire charge. The semantics of mod item charges and mod-owned placed charges do not yet fully match the Java mod. Redstone, chain ignition, and edge cases still need parity coverage.

### Content and models

Every content block needs matching:

- registry declaration;
- blockstate JSON;
- model JSON;
- texture;
- item definition/model when the block has an item.

Missing blockstate bindings can produce `unbound gallery block state` during render-gallery loading.

## Community work list

Priority 1:

- add a native creative tab through a public API;
- stabilize G and the UI event bridge;
- replace thousands of TNT entities with a server-side scheduler;
- add dedicated-server support;
- add an integration test for place -> ignite -> fuse -> explosion;
- add a startup test with the content mod enabled.

Priority 2:

- parity tests against a Java reference for fuse, random push, drops, and chain priming;
- LAN/remote packet support for large explosions;
- correct item consumption in survival and creative;
- correct sounds, particles, and camera shake for each tier;
- configurable power/radius/entity budgets;
- a warning before world-scale explosions.

Priority 3:

- recipes and balance;
- tag integration;
- complete translations;
- animated primed-charge rendering;
- save/load for active mod charges;
- profiling and benchmarks;
- command limits and protection against accidental destruction of spawn or production worlds.

## Checks

Current-tree checks:

```powershell
cargo run -p cargo-horizon-mod -- build
cargo run -p cargo-horizon-mod -- test
cargo test -p horizon-client --lib creative::tests::mod_items_join_their_namespace_tab_and_the_search_tab
cargo check -p horizon-client --bin horizon-client --features horizon-client/mods
```

These checks validate the build and API contracts. They do not prove that every large explosion is safe or performant in a real world.
