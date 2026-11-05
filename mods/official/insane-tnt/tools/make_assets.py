"""Writes the content of the Insane TNT mod: its item declarations, recipes,
item models, 16x16 textures, the two languages, and `src/kinds.rs`.

One table here is the single source: a type's id, what it is called, its
colours, its crafting ingredient and what it does. No file of the original
Java mod is used; the textures are drawn here.

    python tools/make_assets.py
"""
import json
import os
import struct
import zlib

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
NS = "insane_tnt"

# path, English name, Russian name, ingredient, effects, hue, saturation
# Effects use the same names as `Effect` in src/lib.rs.
TYPES = [
    ("small_tnt", "Small TNT", "Малый динамит", "minecraft:gunpowder",
     [("blast", 1, 0.0)], 0.0, 0.95),
    ("high_explosive_tnt", "High Explosive TNT", "Фугасный динамит", "minecraft:blaze_powder",
     [("blast", 4, 1.2), ("sound",)], 0.02, 1.0),
    ("mini_nuke", "Mini Nuke", "Мини-бомба", "minecraft:fire_charge",
     [("blast", 10, 2.5), ("fire", 8, 3.0)], 0.05, 1.0),
    ("nuke", "Nuke", "Ядерная бомба", "minecraft:nether_star",
      [("blast", 800, 28.0), ("fire", 120, 24.0), ("dissolve", 24), ("sound",)], 0.07, 1.0),
    ("mega_nuke", "Mega Nuke", "Мега-бомба", "minecraft:netherite_ingot",
      [("blast", 1400, 42.0), ("fire", 180, 36.0), ("dissolve", 30), ("bolt", 12, 32.0, False)], 0.08, 1.0),
    ("ultra_nuke", "Ultra Nuke", "Ультра-бомба", "minecraft:dragon_breath",
      [("blast", 2200, 60.0), ("fire", 240, 48.0), ("dissolve", 40), ("bolt", 18, 48.0, False)], 0.09, 1.0),
    ("titan_nuke", "Titan Nuke", "Титан-бомба", "minecraft:heart_of_the_sea",
      [("blast", 3000, 80.0), ("fire", 300, 64.0), ("dissolve", 48), ("bolt", 24, 64.0, False)], 0.10, 1.0),
    ("kraken_nuke", "Kraken Nuke", "Кракен-бомба", "minecraft:prismarine_shard",
     [("blast", 64, 8.0), ("bolt", 12, 10.0, True), ("freeze", 6)], 0.52, 0.95),
    ("chungus_nuke", "Chungus Nuke", "Чунгус-бомба", "minecraft:enchanted_golden_apple",
      [("blast", 4000, 100.0), ("fire", 400, 80.0), ("dissolve", 56), ("bolt", 32, 80.0, False)], 0.12, 1.0),
    ("big_chungus_tnt", "Big Chungus TNT", "Большой чунгус", "minecraft:golden_apple",
     [("blast", 24, 4.0), ("fire", 12, 5.0)], 0.13, 1.0),
    ("world_dissolver", "World Dissolver", "Растворитель мира", "minecraft:echo_shard",
     [("dissolve", 8), ("blast", 40, 10.0)], 0.72, 0.75),
    ("ultra_world_dissolver", "Ultra World Dissolver", "Ультра-растворитель мира",
     "minecraft:recovery_compass",
     [("dissolve", 15), ("blast", 64, 16.0), ("bolt", 12, 18.0, False)], 0.75, 0.85),
    ("volcano_tnt", "Volcano TNT", "Вулкан", "minecraft:magma_block",
     [("blast", 4, 1.5), ("lava", 1), ("fire", 12, 4.0)], 0.03, 1.0),
    ("super_volcano_tnt", "Super Volcano TNT", "Супер-вулкан", "minecraft:magma_cream",
     [("blast", 10, 3.0), ("lava", 2), ("fire", 24, 8.0), ("bolt", 4, 8.0, False)], 0.04, 1.0),
    ("storm_tnt", "Storm TNT", "Шторм", "minecraft:phantom_membrane",
     [("bolt", 6, 8.0, True), ("blast", 2, 1.0)], 0.55, 0.45),
    ("super_storm_tnt", "Super Storm TNT", "Супер-шторм", "minecraft:lightning_rod",
     [("bolt", 14, 12.0, True), ("blast", 4, 2.0)], 0.58, 0.5),
    ("sunstorm_tnt", "Sunstorm TNT", "Солнечный шторм", "minecraft:sunflower",
     [("noon",), ("bolt", 8, 6.0, False), ("fire", 16, 6.0)], 0.14, 1.0),
    ("super_sunstorm_tnt", "Super Sunstorm TNT", "Супер-солнечный шторм", "minecraft:glowstone",
     [("noon",), ("bolt", 18, 10.0, False), ("fire", 30, 10.0)], 0.15, 1.0),
    ("lightning_tnt", "Lightning TNT", "Молния", "minecraft:copper_ingot",
     [("bolt", 4, 3.0, False), ("blast", 1, 0.0)], 0.16, 0.9),
    ("focused_lightning_tnt", "Focused Lightning TNT", "Сфокусированная молния",
     "minecraft:amethyst_shard",
     [("bolt", 8, 0.5, False), ("blast", 2, 0.5)], 0.78, 0.8),
    ("meteor_tnt", "Meteor TNT", "Метеорит", "minecraft:flint_and_steel",
     [("meteor", 6, "minecraft:magma_block"), ("blast", 4, 2.0)], 0.06, 0.9),
    ("anvil_tnt", "Anvil TNT", "Наковальня", "minecraft:anvil",
     [("anvil", 12), ("blast", 1, 0.0)], 0.6, 0.1),
    ("arrow_tnt", "Arrow TNT", "Стрелы", "minecraft:arrow",
     [("arrow", 24), ("blast", 1, 0.0)], 0.1, 0.35),
    ("wither_tnt", "Wither TNT", "Иссушитель", "minecraft:wither_skeleton_skull",
     [("mobs", ["minecraft:wither_skeleton"], 3), ("blast", 2, 1.0)], 0.0, 0.0),
    ("crystal_tnt", "Crystal TNT", "Кристалл", "minecraft:end_crystal",
     [("crystal", 6), ("blast", 1, 0.0)], 0.85, 0.75),
    ("sponge_tnt", "Sponge TNT", "Губка", "minecraft:sponge",
     [("sponge", 6), ("blast", 2, 1.0)], 0.14, 0.6),
    ("ice_tnt", "Ice TNT", "Лёд", "minecraft:blue_ice",
     [("freeze", 5), ("blast", 2, 1.0)], 0.55, 0.7),
    ("subzero_tnt", "Subzero TNT", "Субзеро", "minecraft:packed_ice",
     [("freeze", 8), ("blast", 3, 1.5), ("bolt", 3, 5.0, True)], 0.56, 0.9),
    ("slime_tnt", "Slime TNT", "Слизь", "minecraft:slime_ball",
     [("slime", 4), ("blast", 2, 1.0)], 0.33, 0.65),
    ("magic_slime_tnt", "Magic Slime TNT", "Магическая слизь", "minecraft:slime_block",
     [("slime", 8), ("blast", 2, 1.0), ("bolt", 3, 4.0, False)], 0.4, 0.8),
    ("night_tnt", "Night TNT", "Ночь", "minecraft:coal_block",
     [("night",), ("blast", 3, 1.5)], 0.68, 0.45),
    ("glitch_tnt", "Glitch TNT", "Глитч", "minecraft:sculk_sensor",
     [("wool", 4), ("blast", 3, 1.5)], 0.83, 0.9),
    ("virus_tnt", "Virus TNT", "Вирус", "minecraft:fermented_spider_eye",
     [("mobs", ["minecraft:zombie"], 4), ("blast", 3, 2.0)], 0.28, 0.9),
    ("spread_tnt", "Spread TNT", "Разброс", "minecraft:redstone",
     [("ring", 12, 4.0)], 0.99, 0.9),
    ("super_spread_tnt", "Super Spread TNT", "Супер-разброс", "minecraft:redstone_block",
     [("ring", 24, 8.0)], 0.97, 0.85),
    ("respawn_tnt", "Respawn TNT", "Возрождение", "minecraft:compass",
     [("spawn",), ("blast", 2, 1.0)], 0.45, 0.4),
    ("buff_tnt", "Buff TNT", "Бафф", "minecraft:glistering_melon_slice",
     [("buff",)], 0.35, 0.9),
    ("bnt", "BeeNT", "Пчелиный", "minecraft:honeycomb",
     [("mobs", ["minecraft:bee"], 12), ("blast", 1, 0.0)], 0.13, 1.0),
    ("gay_tnt", "Gay TNT", "Радужный", "minecraft:white_wool",
     [("wool", 4), ("blast", 1, 0.0)], 0.5, 0.6),
    ("cosmic_tnt", "Cosmic TNT", "Космический", "minecraft:amethyst_cluster",
     [("night",), ("meteor", 6, "minecraft:glowstone"), ("blast", 4, 2.0)], 0.7, 0.8),
    ("y_tnt", "Y TNT", "Y-динамит", "minecraft:gunpowder",
     [("column", 16)], 0.2, 0.85),
    ("transformation_tnt", "Transformation TNT", "Трансформация", "minecraft:gold_ingot",
     [("fill", 4, "minecraft:gold_block", "minecraft:stone"), ("blast", 2, 1.0)], 0.11, 1.0),
]

RARITY = {"small": "common", "high": "uncommon", "mini": "uncommon", "nuke": "rare",
          "mega": "rare", "ultra": "epic", "titan": "epic", "chungus": "epic",
          "world": "epic", "kraken": "rare", "big": "uncommon"}


def rarity_for(path):
    for key, value in RARITY.items():
        if path.startswith(key):
            return value
    return "common"


def effect_rust(effect):
    name = effect[0]
    args = effect[1:]
    if name == "blast":
        return "Effect::Blast { count: %d, spread: %.1f }" % args
    if name == "fire":
        return "Effect::Fire { count: %d, spread: %.1f }" % args
    if name == "bolt":
        count, spread, thunder = args
        return "Effect::Bolt { count: %d, spread: %.1f, thunder: %s }" % (
            count, spread, "true" if thunder else "false")
    if name == "meteor":
        count, block = args
        return "Effect::Meteor { count: %d, block: \"%s\" }" % (count, block)
    if name == "mobs":
        kinds, count = args
        return "Effect::Mobs { kinds: &[%s], count: %d }" % (
            ", ".join('"%s"' % k for k in kinds), count)
    if name == "ring":
        count, radius = args
        return "Effect::Ring { count: %d, radius: %.1f }" % (count, radius)
    if name == "fill":
        radius, block, replace = args
        return "Effect::Fill { radius: %d, block: \"%s\", replace: \"%s\" }" % (
            radius, block, replace)
    if name == "anvil":
        return "Effect::Anvil { count: %d }" % (args[0],)
    if name == "arrow":
        return "Effect::Arrow { count: %d }" % (args[0],)
    if name == "crystal":
        return "Effect::Crystal { count: %d }" % (args[0],)
    if name == "sponge":
        return "Effect::Sponge { radius: %d }" % (args[0],)
    if name == "freeze":
        return "Effect::Freeze { radius: %d }" % (args[0],)
    if name == "slime":
        return "Effect::Slime { count: %d }" % (args[0],)
    if name == "wool":
        return "Effect::Wool { radius: %d }" % (args[0],)
    if name == "dissolve":
        return "Effect::Dissolve { radius: %d }" % (args[0],)
    if name == "lava":
        return "Effect::Lava { radius: %d }" % (args[0],)
    if name == "column":
        return "Effect::Column { count: %d }" % (args[0],)
    plain = {"night": "Effect::Night", "noon": "Effect::Noon", "spawn": "Effect::SpawnPoint",
             "buff": "Effect::Buff", "sound": "Effect::Boom"}
    return plain[name]


def write(path, text):
    full = os.path.join(ROOT, path)
    os.makedirs(os.path.dirname(full), exist_ok=True)
    with open(full, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)


def write_json(path, value):
    write(path, json.dumps(value, indent=2, ensure_ascii=False) + "\n")


def hsv(h, s, v):
    h = (h % 1.0) * 6.0
    i = int(h)
    f = h - i
    p = v * (1 - s)
    q = v * (1 - s * f)
    t = v * (1 - s * (1 - f))
    rgb = [(v, t, p), (q, v, p), (p, v, t), (p, q, v), (t, p, v), (v, p, q)][i % 6]
    return tuple(int(round(c * 255)) for c in rgb)


# The 16x16 face: a TNT crate, `accent` on the bands, near-white in the middle
# with the letters in the accent's dark tone.
LETTERS = [
    "..###.###..#.#.###..",
    "...#..##.##.#.#..#..",
    "...#..##.#.##.#..#..",
    "...#..#.#..#..#..#..",
]


def texture(path, hue, saturation):
    accent = hsv(hue, saturation, 0.86)
    dark = hsv(hue, saturation, 0.38)
    light = hsv(hue, saturation * 0.25, 0.97)
    pixels = [[accent for _ in range(16)] for _ in range(16)]
    for x in range(16):
        for y in range(16):
            if 3 <= y <= 12:
                pixels[y][x] = light
            if y in (3, 12) or x in (0, 15):
                pixels[y][x] = dark
    # The stripe with the letters, three rows high, centred.
    for row in range(3):
        y = 5 + row * 2
        for x in range(16):
            if 1 <= x <= 14:
                pixels[y][x] = (255, 255, 255) if (x + y) % 7 else (242, 242, 242)
    glyph = "TNT"
    for letter, index in zip(glyph, range(3)):
        for dy, row in enumerate(LETTERS):
            for dx, cell in enumerate(row[:5]):
                if cell == "#":
                    x = 2 + index * 5 + dx
                    y = 5 + dy
                    if 0 <= x < 16 and 0 <= y < 16:
                        pixels[y][x] = dark
    raw = b"".join(b"\x00" + bytes(c for px in row for c in px) for row in pixels)
    def chunk(kind, data):
        return (struct.pack(">I", len(data)) + kind + data
                + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF))
    png = (b"\x89PNG\r\n\x1a\n"
           + chunk(b"IHDR", struct.pack(">IIBBBBB", 16, 16, 8, 6, 0, 0, 0))
           + chunk(b"IDAT", zlib.compress(raw, 9))
           + chunk(b"IEND", b""))
    full = os.path.join(ROOT, "assets", NS, "textures", "item", path + ".png")
    os.makedirs(os.path.dirname(full), exist_ok=True)
    with open(full, "wb") as f:
        f.write(png)


def main():
    # The charge block copies TNT behaviour, while remaining breakable.
    write_json("data/%s/horizon/block/lit_charge.json" % NS, {
        "template": "minecraft:tnt",
        "properties": {"hardness": 0.0},
        "item": True,
    })
    en, ru = {}, {}
    kinds = []
    for path, name, name_ru, ingredient, effects, hue, saturation in TYPES:
        write_json("data/%s/horizon/item/%s.json" % (NS, path), {
            "template": "minecraft:tnt",
            "rarity": rarity_for(path),
        })
        write_json("data/%s/recipe/%s.json" % (NS, path), {
            "type": "minecraft:crafting_shapeless",
            "category": "redstone",
            "ingredients": ["minecraft:tnt", ingredient, "minecraft:gunpowder"],
            "result": {"id": "%s:%s" % (NS, path)},
        })
        write_json("assets/%s/items/%s.json" % (NS, path), {
            "model": {"type": "minecraft:model", "model": "%s:item/%s" % (NS, path)},
        })
        write_json("assets/%s/models/item/%s.json" % (NS, path), {
            "parent": "minecraft:item/generated",
            "textures": {"layer0": "%s:item/%s" % (NS, path)},
        })
        texture(path, hue, saturation)
        en["item.%s.%s" % (NS, path)] = name
        ru["item.%s.%s" % (NS, path)] = name_ru
        kinds.append((path, name, effects))
    write_json("assets/%s/items/lit_charge.json" % NS, {
        "model": {"type": "minecraft:model", "model": "minecraft:item/tnt"},
    })
    write_json("assets/%s/models/block/lit_charge.json" % NS, {
        "parent": "minecraft:block/cube_all",
        "textures": {"all": "minecraft:block/tnt_side"},
    })
    write_json("assets/%s/blockstates/lit_charge.json" % NS, {
        "variants": {"": {"model": "%s:block/lit_charge" % NS}},
    })
    write_json("assets/%s/items/lit_charge.json" % NS, {
        "model": {"type": "minecraft:model", "model": "minecraft:item/tnt"},
    })
    write_json("assets/%s/lang/en_us.json" % NS, en)
    write_json("assets/%s/lang/ru_ru.json" % NS, ru)

    lines = [
        "//! The TNT types, generated by `tools/make_assets.py` from its one table:",
        "//! ids, names, recipes and colours there, effects here.",
        "",
        "/// What a type does when it goes off at a point.",
        "#[derive(Clone, Copy)]",
        "pub enum Effect {",
        "    /// Primed TNT scattered in a sphere: the explosions add up.",
        "    Blast { count: u32, spread: f64 },",
        "    /// Fire blocks scattered around.",
        "    Fire { count: u32, spread: f64 },",
        "    /// Lightning bolts, and a thunderstorm when `thunder`.",
        "    Bolt { count: u32, spread: f64, thunder: bool },",
        "    /// Falling blocks from the sky.",
        "    Meteor { count: u32, block: &'static str },",
        "    /// Falling anvils from the sky.",
        "    Anvil { count: u32 },",
        "    /// Arrows raining down.",
        "    Arrow { count: u32 },",
        "    /// Mobs let loose.",
        "    Mobs { kinds: &'static [&'static str], count: u32 },",
        "    /// End crystals, which blow up when the blast reaches them.",
        "    Crystal { count: u32 },",
        "    /// Drains water in a sphere.",
        "    Sponge { radius: i32 },",
        "    /// Freezes water into ice in a sphere.",
        "    Freeze { radius: i32 },",
        "    /// Slimes let loose.",
        "    Slime { count: u32 },",
        "    /// A cube of blocks, `replace`d.",
        "    Fill { radius: i32, block: &'static str, replace: &'static str },",
        "    /// A ring of TNT with staggered fuses.",
        "    Ring { count: u32, radius: f64 },",
        "    /// A column of TNT straight up.",
        "    Column { count: u32 },",
        "    /// A pool of lava.",
        "    Lava { radius: i32 },",
        "    /// Air in a sphere: nothing is left.",
        "    Dissolve { radius: i32 },",
        "    /// A patchwork of every wool colour.",
        "    Wool { radius: i32 },",
        "    /// Midnight.",
        "    Night,",
        "    /// Noon.",
        "    Noon,",
        "    /// Your spawn point here.",
        "    SpawnPoint,",
        "    /// Good effects on you.",
        "    Buff,",
        "    /// The blast's sound.",
        "    Boom,",
        "}",
        "",
        "pub struct Kind {",
        "    pub path: &'static str,",
        "    pub name: &'static str,",
        "    pub effects: &'static [Effect],",
        "}",
        "",
        "pub const KINDS: &[Kind] = &[",
    ]
    for path, name, effects in kinds:
        lines.append("    Kind {")
        lines.append('        path: "%s",' % path)
        lines.append('        name: "%s",' % name)
        lines.append("        effects: &[")
        for effect in effects:
            lines.append("            %s," % effect_rust(effect))
        lines.append("        ],")
        lines.append("    },")
    lines.append("];")
    lines.append("")
    lines.append("pub fn find(path: &str) -> Option<&'static Kind> {")
    lines.append("    KINDS.iter().find(|k| k.path == path)")
    lines.append("}")
    lines.append("")
    write("src/kinds.rs", "\n".join(lines))
    print("wrote %d types" % len(kinds))


if __name__ == "__main__":
    main()
