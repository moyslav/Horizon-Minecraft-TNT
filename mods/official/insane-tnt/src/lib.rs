//! Insane TNT for Horizon (`docs/mods.md`).
//!
//! Every TNT type is an item. Right-click places a charge; sneak + right-click
//! on the placed charge lights its fuse:
//! `/tnt place <type>` does the same.
//!
//! A placed charge is the block `insane_tnt:lit_charge`, declared with TNT's
//! template: the game treats it as TNT wherever a player does. Flint and
//! sneak + right-click or redstone lights it; the mod tracks the fuse and
//! blasts with its type where it lies.
//!
//! Lighting a charge the mod does not see its charge gone from (the blast of
//! another, a charge lit before the mod loaded) lights it once noticed: the
//! mod watches every charge for as long as it stands in a world.
//!
//! Every blast is built from vanilla commands (capability `command`), so a
//! world needs cheats, as the predator-missile example's do.
//!
//! Primed TNT in Horizon always explodes with power 4, so larger tiers chain
//! hundreds to thousands of explosions and clear a wide cube.
mod kinds;

use horizon_mod_sdk::{
    Mod, Registration, chat, command, game,
    input::{self, Event},
    recipes::{self, Stack},
    ui::{self, ItemWidget, Screen, Widget},
};
use kinds::{Effect, KINDS, Kind};
use std::collections::VecDeque;

/// How far the player can place or ignite a charge.
const REACH: f32 = 6.0;
/// Where the charge goes when nothing is hit: this far along the view.
const NO_HIT: f64 = 6.0;
/// Command lines handed to the game a tick: a batch, then the next tick's.
const PER_TICK: usize = 8;
/// Ticks to wait before the next charge, so a held button cannot spam.
const COOLDOWN: u32 = 8;
/// The placed charge's block: TNT's behaviour and states, no item.
const BLOCK: &str = "insane_tnt:lit_charge";
/// Ticks from lighting a placed charge to its blast: TNT's fuse.
const FUSE: u64 = 80;
/// How far charges are watched: to the far side of the render distance
/// either way.
const WATCH: f32 = 256.0;
/// Charges taken from the world once their column is out of `WATCH`.
const AWAY: f32 = 300.0;
/// How many charges are followed at once; the oldest forgotten first.
const CHARGES: usize = 128;
/// Ticks between two puffs of smoke over a lit charge.
const FLASH: u64 = 8;
/// Command lines one tick of watching may add up to.
const WATCH_LINES: usize = 32;

/// A charge lying in the world, as far as the mod has seen it.
struct Charge {
    /// The block's corner.
    cell: [i32; 3],
    /// Its centre.
    center: [f64; 3],
    /// What it is.
    kind: &'static Kind,
    /// Seen at its spot while it was last looked at: gone is lit.
    seen: bool,
    /// The tick it was first found gone, if it was.
    lit: Option<u64>,
}

struct InsaneTnt {
    /// The commands still to run, oldest first.
    queue: VecDeque<String>,
    /// Ticks until the use button works again.
    cooldown: u32,
    /// A counter to seed the scatter: the same type at the same spot still
    /// looks different every time.
    seed: u64,
    /// The charges placed in the world, newest last.
    charges: Vec<Charge>,
    /// A player in a world last tick: charges are dropped when they leave.
    in_world: bool,
}

impl Mod for InsaneTnt {
    fn init() -> Result<(Self, Registration), String> {
        let ids: Vec<String> = KINDS.iter().map(|k| format!("{}:{}", NS, k.path)).collect();
        input::claim_items(&ids);
        // "G" opens the picker; rebindable on the mod's Mods page.
        let _ = input::register_key(MENU_KEY, "Insane TNT charges", "key.keyboard.g");
        chat::show(&format!(
            "Insane TNT: {} types. Use /tnt menu; right-click places, sneak + right-click lights a charge.",
            KINDS.len()
        ));
        Ok((
            InsaneTnt {
                queue: VecDeque::new(),
                cooldown: 0,
                seed: 0x2545_F491_4F6C_DD1D,
                charges: Vec::new(),
                in_world: false,
            },
            Registration::new().command("tnt").on_tick().on_frame(),
        ))
    }

    fn tick(&mut self, tick: u64) {
        for event in input::poll() {
            match event {
                Event::UseStart(use_start) => {
                    let held = &use_start.item;
                    let Some(kind) = path_of(&held.id).and_then(kinds::find) else {
                        continue;
                    };
                    if self.cooldown > 0 {
                        continue;
                    }
                    self.cooldown = COOLDOWN;
                    let hit = game::raycast(REACH);
                    if input::mapping("key.sneak").is_some_and(|m| m.down) {
                        if let Some(hit) = hit.filter(|hit| hit.block == BLOCK) {
                            self.ignite([hit.block_x, hit.block_y, hit.block_z], tick);
                        }
                    } else if let Some(hit) = hit {
                        self.place(
                            kind,
                            [
                                f64::from(hit.block_x) + 0.5,
                                f64::from(hit.block_y) + 0.5,
                                f64::from(hit.block_z) + 0.5,
                            ],
                        );
                    } else if let Some(at) = aim_point() {
                        self.place(kind, at);
                    }
                }
                Event::KeyDown(name) if name == MENU_KEY => self.open_picker(),
                _ => {}
            }
        }
        if game::get_player().is_none() {
            self.charges.clear();
            self.in_world = false;
        } else {
            self.in_world = true;
        }
        self.watch(tick);
        self.cooldown = self.cooldown.saturating_sub(1);
        if !self.queue.is_empty() {
            let mut batch: Vec<String> = Vec::with_capacity(PER_TICK);
            while batch.len() < PER_TICK {
                match self.queue.pop_front() {
                    Some(line) => batch.push(line),
                    None => break,
                }
            }
            command::run_quiet(&batch);
        }
    }

    fn ui_select(&mut self, id: u32, _row: u32) {
        let Some(kind) = KINDS.get(id as usize) else {
            return;
        };
        match self.give(kind) {
            Ok(()) => chat::show(&format!("{} is yours: right-click to place", kind.name)),
            Err(e) => chat::show(&format!("{} cannot be handed over here: {e}", kind.name)),
        }
    }
    fn command(&mut self, _name: &str, args: &str) -> Result<Option<String>, String> {
        let args = args.trim();
        let (verb, rest) = match args.split_once(' ') {
            Some((verb, rest)) => (verb, rest.trim()),
            None => (args, ""),
        };
        match verb {
            "" | "list" => {
                let names: Vec<&str> = KINDS.iter().map(|k| k.name).collect();
                chat::show(&format!("{} types:", KINDS.len()));
                for line in names.chunks(4) {
                    chat::show(&line.join(", "));
                }
                Ok(Some(
                    "Right-click places a charge; sneak + right-click lights it. /tnt give <type> for one".into(),
                ))
            }
            "give" => {
                let kind =
                    kinds::find(rest).ok_or_else(|| format!("no TNT type {rest:?}: /tnt list"))?;
                match self.give(kind) {
                    Ok(()) => Ok(Some(format!("Gave you {}", kind.name))),
                    // No creative: the command, with the player's permissions.
                    Err(_) => {
                        self.run(format!("give @s {}:{}", NS, kind.path));
                        Ok(Some(format!("Gave you {}", kind.name)))
                    }
                }
            }
            "menu" => {
                self.open_picker();
                Ok(None)
            }
            "here" => {
                let kind =
                    kinds::find(rest).ok_or_else(|| format!("no TNT type {rest:?}: /tnt list"))?;
                let at = match game::get_player() {
                    Some(p) => [p.x, p.y, p.z],
                    None => return Err("Not in a world".into()),
                };
                self.detonate_at(kind, at);
                Ok(Some(format!("{} goes off under you", kind.name)))
            }
            "place" => {
                let kind =
                    kinds::find(rest).ok_or_else(|| format!("no TNT type {rest:?}: /tnt list"))?;
                let at = aim_point().ok_or("Nothing to place it on: look at a block")?;
                self.place(kind, at);
                Ok(Some(format!(
                    "{} is placed: light it with flint and steel",
                    kind.name
                )))
            }
            other => match kinds::find(other).filter(|_| rest.is_empty()) {
                Some(kind) => {
                    let at = aim_point().ok_or("Nothing to aim at: look at a block")?;
                    self.detonate_at(kind, at);
                    Ok(Some(format!("{} goes off where you look", kind.name)))
                }
                None => Err(format!("Unknown TNT type {other:?}: /tnt list")),
            },
        }
    }
}

/// The mod's namespace; every item is `<this>:<kind path>`.
const NS: &str = "insane_tnt";
/// The picker screen's list widget.
/// The key that opens the picker (input.register_key's name).
const MENU_KEY: &str = "insane_tnt_menu";

/// `insane_tnt:nuke` -> `nuke`.
fn path_of(id: &str) -> Option<&str> {
    id.strip_prefix(NS)?.strip_prefix(':')
}

/// The point along the view to blow up: the first block hit, else a few
/// blocks ahead of the player.
fn aim_point() -> Option<[f64; 3]> {
    if let Some(hit) = game::raycast(REACH) {
        // The block's own cell: a charge put down replaces what was hit.
        return Some([
            f64::from(hit.block_x) + 0.5,
            f64::from(hit.block_y) + 0.5,
            f64::from(hit.block_z) + 0.5,
        ]);
    }
    let p = game::get_player()?;
    let yaw = f64::from(p.yaw).to_radians();
    let pitch = f64::from(p.pitch).to_radians();
    let dir = [
        -yaw.sin() * pitch.cos(),
        -pitch.sin(),
        yaw.cos() * pitch.cos(),
    ];
    Some([
        p.x + dir[0] * NO_HIT,
        p.y + 1.62 + dir[1] * NO_HIT,
        p.z + dir[2] * NO_HIT,
    ])
}

/// The cell a corner is in.
fn cell_of(at: [f64; 3]) -> [i32; 3] {
    [
        at[0].floor() as i32,
        at[1].floor() as i32,
        at[2].floor() as i32,
    ]
}

/// Whether `at` is within `distance` of the player.
fn near(at: [f64; 3], distance: f32) -> bool {
    game::get_player().is_some_and(|p| {
        let d = [p.x - at[0], p.y - at[1], p.z - at[2]];
        d.iter().map(|v| v * v).sum::<f64>() <= f64::from(distance) * f64::from(distance)
    })
}

/// What the view finds in the cell at `cell`: its block, or `None` when the
/// view lands elsewhere or on nothing. The game looks far past a player's
/// arm's reach.
fn glance(cell: [i32; 3]) -> Option<String> {
    let hit = game::raycast(REACH)?;
    if [hit.block_x, hit.block_y, hit.block_z] == cell {
        Some(hit.block)
    } else {
        None
    }
}

/// Dust in a block, replacing no block.
fn particles(id: &str, at: [f64; 3], count: u32) -> String {
    format!(
        "particle minecraft:{id} {:.2} {:.2} {:.2} 0.4 0.4 0.4 0.02 {count} force",
        at[0], at[1], at[2]
    )
}

impl InsaneTnt {
    fn run(&mut self, line: String) {
        self.queue.push_back(line);
    }

    /// Puts a charge of `kind` down at `at` and follows it. The block stands
    /// where the view landed, the same cell, so flint and steel aimed at it
    /// hits it: a charge replaces whatever was there, as TNT would.
    fn place(&mut self, kind: &'static Kind, at: [f64; 3]) {
        let cell = cell_of(at);
        let [x, y, z] = cell;
        // A block of the same state is not set again: it would not be "new"
        // to the game, and it is still the charge that was there.
        if glance(cell).is_none_or(|seen| seen != BLOCK) {
            self.run(format!("setblock {x} {y} {z} {BLOCK}"));
        }
        self.follow(cell, kind);
    }

    fn ignite(&mut self, cell: [i32; 3], tick: u64) {
        if let Some(charge) = self.charges.iter_mut().find(|charge| charge.cell == cell) {
            charge.lit.get_or_insert(tick);
            charge.seen = false;
            chat::show("Charge lit");
        }
    }

    /// Follows a charge just put down; one already at the spot takes the
    /// newest type.
    fn follow(&mut self, cell: [i32; 3], kind: &'static Kind) {
        self.charges.retain(|c| c.cell != cell);
        if self.charges.len() >= CHARGES {
            self.charges.remove(0);
        }
        self.charges.push(Charge {
            cell,
            center: [
                f64::from(cell[0]) + 0.5,
                f64::from(cell[1]) + 0.5,
                f64::from(cell[2]) + 0.5,
            ],
            kind,
            seen: true,
            lit: None,
        });
    }

    /// The game's index of a type's item: its charges are summoned by index
    /// when the type has no item of its own, which is all of them but the
    /// block (yet).
    fn index(&self, kind: &Kind) -> Result<u32, String> {
        let id = format!("{}:{}", NS, kind.path);
        recipes::items()
            .iter()
            .position(|item| item.id == id)
            .map(|i| i as u32)
            .ok_or_else(|| format!("{id} is not a known item"))
    }

    /// Hands one charge of `kind` over, as the creative inventory does: no
    /// command, so it works without cheats and in a survival world it fails
    /// with the game's reason.
    fn give(&mut self, kind: &Kind) -> Result<(), String> {
        let index = self.index(kind)?;
        recipes::give(&Stack {
            item: index,
            count: 1,
            components: String::new(),
        })
    }

    /// `/tnt menu`: every type with its item icon; a click hands one over.
    fn open_picker(&mut self) {
        let widgets = KINDS
            .iter()
            .enumerate()
            .map(|(i, kind)| {
                Widget::Item(ItemWidget {
                    id: i as u32,
                    item: format!("{}:{}", NS, kind.path),
                    name: kind.name.to_owned(),
                })
            })
            .collect();
        ui::open_screen(&Screen {
            title: "Insane TNT".to_owned(),
            widgets,
        });
    }

    /// Follows the placed charges. A charge the view no longer finds at its
    /// spot was taken by the game to light: TNT's own priming removes the
    /// block, so the mod knows the type and sets the charge off there, and
    /// the game's TNT does the rest of the fuse.
    fn watch(&mut self, tick: u64) {
        if !self.in_world {
            return;
        }
        let mut out = Vec::new();
        self.charges.retain(|c| near(c.center, WATCH + AWAY));
        for c in self.charges.iter_mut().filter(|c| c.lit.is_none()) {
            let seen = near(c.center, WATCH) && glance(c.cell).is_some_and(|s| s == BLOCK);
            if seen {
                c.seen = true;
                continue;
            }
            let was = std::mem::replace(&mut c.seen, false);
            // Once per absence: the first sighting gone is the lighting.
            if was {
                c.lit = Some(tick);
            }
        }
        // A lit charge booms as it flashes: fire over it, then its blast.
        let due: Vec<Charge> = self
            .charges
            .extract_if(.., |c| {
                c.lit.is_some_and(|lit| tick >= lit.saturating_add(FUSE))
            })
            .collect();
        for c in &due {
            out.push(particles("flame", c.center, 8));
            let [x, y, z] = c.cell;
            out.push(format!("setblock {x} {y} {z} minecraft:air"));
        }
        self.queue.extend(out);
        for c in due {
            self.detonate_at(c.kind, c.center);
        }
        // Flashing lights, as a primed TNT has, for those still burning.
        let on = tick / FLASH % 2 == 0;
        let mut lines = 0;
        for c in &self.charges {
            if c.lit.is_some() && on && lines < WATCH_LINES {
                self.queue.push_back(particles("smoke", c.center, 4));
                lines += 1;
            }
        }
    }

    /// Queues everything `kind` does at `at`.
    fn detonate_at(&mut self, kind: &Kind, at: [f64; 3]) {
        self.seed = self.seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut rng = Rng(self.seed ^ (kind.path.len() as u64));
        let lines = {
            let mut b = Blast {
                lines: Vec::new(),
                rng: &mut rng,
            };
            for effect in kind.effects {
                b.apply(*effect, at);
            }
            b.lines
        };
        self.queue.extend(lines);
        chat::show(&format!("{}!", kind.name));
    }
}

/// A small xorshift: no crate, no imports, the same every run.
struct Rng(u64);
impl Rng {
    fn unit(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    /// `-1.0..=1.0`.
    fn signed(&mut self) -> f64 {
        self.unit() * 2.0 - 1.0
    }
    /// `0..n`.
    fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            (self.unit() * f64::from(n)) as u32
        }
    }
    /// A point in the sphere of `radius` around `at`.
    fn around(&mut self, at: [f64; 3], radius: f64, height: f64) -> [f64; 3] {
        let r = radius * self.unit().cbrt();
        let z = self.signed();
        let theta = self.unit() * std::f64::consts::TAU;
        let flat = (1.0 - z * z).max(0.0).sqrt();
        [
            at[0] + r * flat * theta.cos(),
            at[1] + self.signed() * height,
            at[2] + r * flat * theta.sin(),
        ]
    }
}

/// Builds the command lines of one blast.
struct Blast<'a> {
    lines: Vec<String>,
    rng: &'a mut Rng,
}

impl Blast<'_> {
    fn apply(&mut self, effect: Effect, at: [f64; 3]) {
        match effect {
            Effect::Blast { count, spread } => {
                for _ in 0..count.min(1_200) {
                    let p = self.rng.around(at, spread, spread * 0.6);
                    let fuse = self.rng.below(4) as i32;
                    self.tnt(p, fuse);
                }
            }
            Effect::Fire { count, spread } => {
                for _ in 0..count {
                    let p = self.rng.around(at, spread, spread * 0.5);
                    self.line(format!(
                        "setblock {} {} {} minecraft:fire replace",
                        p[0].floor() as i32,
                        p[1].floor() as i32,
                        p[2].floor() as i32
                    ));
                }
            }
            Effect::Bolt {
                count,
                spread,
                thunder,
            } => {
                if thunder {
                    self.line("weather thunder 120s".to_string());
                }
                for _ in 0..count {
                    let p = self.rng.around(at, spread, spread * 0.3);
                    self.line(format!(
                        "summon minecraft:lightning_bolt {:.2} {:.2} {:.2}",
                        p[0], p[1], p[2]
                    ));
                }
            }
            Effect::Meteor { count, block } => {
                for _ in 0..count {
                    let p = self.rng.around(at, 4.0, 0.0);
                    self.line(format!(
                        "summon minecraft:falling_block {:.2} {:.2} {:.2} \
                         {{BlockState:\"{block}\",Time:1}}",
                        p[0],
                        at[1] + 24.0,
                        p[2]
                    ));
                }
            }
            Effect::Anvil { count } => {
                for _ in 0..count {
                    let p = self.rng.around(at, 5.0, 0.0);
                    self.line(format!(
                        "summon minecraft:falling_block {:.2} {:.2} {:.2} \
                         {{BlockState:\"minecraft:anvil\",Time:1}}",
                        p[0],
                        at[1] + 14.0,
                        p[2]
                    ));
                }
            }
            Effect::Arrow { count } => {
                for _ in 0..count {
                    let p = self.rng.around(at, 5.0, 0.0);
                    self.line(format!(
                        "summon minecraft:arrow {:.2} {:.2} {:.2} {{Motion:[0.0d,-1.4d,0.0d]}}",
                        p[0],
                        at[1] + 12.0,
                        p[2]
                    ));
                }
            }
            Effect::Mobs { kinds, count } => {
                for i in 0..count {
                    let mob = kinds[i as usize % kinds.len()];
                    let p = self.rng.around(at, 4.0, 1.0);
                    self.line(format!(
                        "summon {mob} {:.2} {:.2} {:.2}",
                        p[0],
                        p[1] + 1.0,
                        p[2]
                    ));
                }
            }
            Effect::Crystal { count } => {
                for i in 0..count {
                    let angle = std::f64::consts::TAU * f64::from(i) / f64::from(count);
                    self.line(format!(
                        "summon minecraft:end_crystal {:.2} {:.2} {:.2}",
                        at[0] + 2.0 * angle.cos(),
                        at[1],
                        at[2] + 2.0 * angle.sin()
                    ));
                }
            }
            Effect::Sponge { radius } => {
                self.cube(at, radius, "minecraft:air", "minecraft:water");
            }
            Effect::Freeze { radius } => {
                self.cube(at, radius, "minecraft:ice", "minecraft:water");
            }
            Effect::Slime { count } => {
                for _ in 0..count {
                    let p = self.rng.around(at, 3.0, 1.0);
                    self.line(format!(
                        "summon minecraft:slime {:.2} {:.2} {:.2} {{Size:3}}",
                        p[0],
                        p[1] + 2.0,
                        p[2]
                    ));
                }
            }
            Effect::Fill {
                radius,
                block,
                replace,
            } => {
                self.cube(at, radius, block, replace);
            }
            Effect::Ring { count, radius } => {
                for i in 0..count {
                    let angle = std::f64::consts::TAU * f64::from(i) / f64::from(count);
                    let fuse = self.rng.below(30) as i32;
                    self.tnt(
                        [
                            at[0] + radius * angle.cos(),
                            at[1] + 0.5,
                            at[2] + radius * angle.sin(),
                        ],
                        fuse,
                    );
                }
            }
            Effect::Column { count } => {
                for i in 0..count {
                    let fuse = self.rng.below(10) as i32;
                    self.tnt([at[0], at[1] + f64::from(i), at[2]], fuse);
                }
            }
            Effect::Lava { radius } => {
                self.cube(at, radius, "minecraft:lava", "minecraft:air");
            }
            Effect::Dissolve { radius } => {
                self.cube(at, radius, "minecraft:air", "minecraft:air");
            }
            Effect::Wool { radius } => {
                let colors = [
                    "white",
                    "orange",
                    "magenta",
                    "light_blue",
                    "yellow",
                    "lime",
                    "pink",
                    "gray",
                    "light_gray",
                    "cyan",
                    "purple",
                    "blue",
                    "brown",
                    "green",
                    "red",
                    "black",
                ];
                let r = radius.max(1);
                for (i, color) in colors.iter().enumerate() {
                    // Four columns of four colours, as a 4x4 grid of cubes.
                    let (cx, cz) = ((i % 4) as i32 - 1, (i / 4) as i32 - 1);
                    let x = at[0].floor() as i32 + cx * r;
                    let z = at[2].floor() as i32 + cz * r;
                    let y = at[1].floor() as i32;
                    self.line(format!(
                        "fill {} {} {} {} {} {} minecraft:{color}_wool",
                        x - r / 2,
                        y,
                        z - r / 2,
                        x + r / 2,
                        y + r,
                        z + r / 2
                    ));
                }
            }
            Effect::Night => {
                self.line("time set midnight".to_string());
                self.line("weather clear".to_string());
            }
            Effect::Noon => {
                self.line("time set noon".to_string());
                self.line("weather clear".to_string());
            }
            Effect::SpawnPoint => {
                self.line(format!(
                    "spawnpoint @s {} {} {}",
                    at[0].floor() as i32,
                    at[1].floor() as i32,
                    at[2].floor() as i32
                ));
            }
            Effect::Buff => {
                for (effect, seconds, level) in [
                    ("minecraft:speed", 60, 1),
                    ("minecraft:strength", 60, 1),
                    ("minecraft:regeneration", 30, 1),
                    ("minecraft:resistance", 60, 1),
                    ("minecraft:fire_resistance", 60, 0),
                    ("minecraft:night_vision", 60, 0),
                ] {
                    self.line(format!("effect give @s {effect} {seconds} {level} true"));
                }
            }
            Effect::Boom => {
                self.line(format!(
                    "playsound minecraft:entity.generic.explode master @a {:.2} {:.2} {:.2} 6 0.9",
                    at[0], at[1], at[2]
                ));
                self.line(format!(
                    "particle minecraft:explosion_emitter {:.2} {:.2} {:.2} 0 0 0 1 1",
                    at[0], at[1], at[2]
                ));
            }
        }
    }

    fn line(&mut self, line: String) {
        self.lines.push(line);
    }

    fn tnt(&mut self, at: [f64; 3], fuse: i32) {
        self.line(format!(
            "summon minecraft:tnt {:.2} {:.2} {:.2} {{fuse:{fuse}s}}",
            at[0], at[1], at[2]
        ));
    }

    /// A cube of `block` over everything that is `replace`, at most the
    /// 32768 blocks a `/fill` takes.
    fn cube(&mut self, at: [f64; 3], radius: i32, block: &str, replace: &str) {
        let r = radius.clamp(0, 15);
        let (x, y, z) = (
            at[0].floor() as i32,
            at[1].floor() as i32,
            at[2].floor() as i32,
        );
        // Slice the cube into per-chunk-sized columns so `/fill` stays under
        // vanilla's 32768-block limit while a top-tier charge clears 31^3.
        for z0 in (-r..=r).step_by(8) {
            let z1 = (z0 + 7).min(r);
            for y0 in (-r..=r).step_by(8) {
                let y1 = (y0 + 7).min(r);
                for x0 in (-r..=r).step_by(8) {
                    let x1 = (x0 + 7).min(r);
                    self.line(format!(
                        "fill {} {} {} {} {} {} {block} replace {replace}",
                        x + x0,
                        y + y0,
                        z + z0,
                        x + x1,
                        y + y1,
                        z + z1
                    ));
                }
            }
        }
    }
}

horizon_mod_sdk::export_mod!(InsaneTnt);
