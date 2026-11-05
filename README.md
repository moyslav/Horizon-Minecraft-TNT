# Horizon Minecraft TNT

Experimental TNT implementation for the Horizon Minecraft client, written in Rust/WASM.

## Русский

Это незавершённый экспериментальный перенос многоуровневого TNT-мода на API Horizon.

Подробная документация на русском и английском находится здесь:

- [Полная документация RU/EN](mods/official/insane-tnt/README.md)

В проекте есть:

- 42 экспериментальных типа TNT;
- Rust/WASM mod package;
- generated textures, item definitions, models, recipes and translations;
- TNT-template block integration;
- creative Search/Tools & Utilities integration;
- vanilla TNT explosion power set to `4000.0` instead of `4.0`.

Это не готовый релиз. Перед использованием делайте резервную копию мира: большие заряды могут вызвать сильные лаги, уничтожить большую область или оставить мир в тяжёлом состоянии.

Сборка мода из корня исходного Horizon checkout:

```powershell
cd mods/official/insane-tnt
cargo run -p cargo-horizon-mod -- build
cargo run -p cargo-horizon-mod -- test
```

## English

This repository contains an unfinished experimental multi-tier TNT implementation for the Horizon Minecraft client, written in Rust/WASM.

Full documentation in both Russian and English:

- [Full RU/EN documentation](mods/official/insane-tnt/README.md)

Included work:

- 42 experimental TNT types;
- a Rust/WASM mod package;
- generated textures, item definitions, models, recipes and translations;
- TNT-template block integration;
- creative Search/Tools & Utilities integration;
- vanilla TNT explosion power changed to `4000.0` instead of `4.0`.

This is not a release-ready project. Back up worlds before testing: large charges can cause severe lag, destroy large areas, or leave a world in a heavy state.

Build from the root of the original Horizon checkout:

```powershell
cd mods/official/insane-tnt
cargo run -p cargo-horizon-mod -- build
cargo run -p cargo-horizon-mod -- test
```

## Status

Community contributions are welcome. Known limitations, broken paths, test coverage and the remaining implementation plan are documented in [`mods/official/insane-tnt/README.md`](mods/official/insane-tnt/README.md).
