//! Which blocks are TNT to a player though the game's own `minecraft:tnt` is
//! not the block there: a mod block declared with `"template": "minecraft:tnt"`
//! (`BlockFile` in `horizon-mods`, which copies the behaviour, states and
//! placement rules of its template), read when the game starts.
//!
//! Such a block is TNT everywhere a player treats TNT as TNT: flint and steel
//! or a fire charge primes it (`TntBlock.useItemOn`), redstone primes it
//! (a TNT template fires its neighbour updates), and water does not stop the
//! fuse — a mod's type TNT is placed, lit and ticks off exactly as TNT is.
//! Only `minecraft:tnt`'s look is TNT's: a mod's own model draws its charge.
//!
//! With no mods the list is empty and TNT is `minecraft:tnt` alone.

use horizon_world::registry::{BlockRegistry, BlockStateId};
use std::collections::HashSet;

/// The mod blocks that are TNT.
#[derive(Default)]
pub(crate) struct TntKinds {
    states: HashSet<BlockStateId>,
}

impl TntKinds {
    /// The TNT-template blocks of the mods that are on, in the registry of a
    /// game starting with `enabled` on.
    #[cfg(feature = "mods")]
    pub(crate) fn load(
        enabled: &std::collections::BTreeSet<String>,
        dev: &[std::path::PathBuf],
        registry: &BlockRegistry,
    ) -> Self {
        let mut kinds = Self::default();
        kinds.add(&super::mods::mod_tnt_blocks(enabled, dev), registry);
        kinds
    }

    /// A build without the mod runtime has no mod blocks, so only
    /// `minecraft:tnt` is TNT.
    #[cfg(not(feature = "mods"))]
    pub(crate) fn load(
        _enabled: &std::collections::BTreeSet<String>,
        _dev: &[std::path::PathBuf],
        registry: &BlockRegistry,
    ) -> Self {
        let mut kinds = Self::default();
        kinds.add(&[], registry);
        kinds
    }

    /// Adds the states of `minecraft:tnt` and of every block named in
    /// `blocks`.
    #[cfg_attr(not(feature = "mods"), allow(dead_code))]
    fn add(&mut self, blocks: &[String], registry: &BlockRegistry) {
        self.add_block(registry, "minecraft:tnt");
        for block in blocks {
            self.add_block(registry, block);
        }
    }

    /// Every state of `block`: its default and each other state whose name
    /// is `block` (`tnt[unstable=true]`); `tnt[unstable=true]` alone is
    /// another block.
    fn add_block(&mut self, registry: &BlockRegistry, block: &str) {
        let mut found = false;
        for (id, state) in registry.states() {
            if state.name == block {
                self.states.insert(id);
                found = true;
            }
        }
        if !found {
            log::warn!("Horizon: TNT block {block} is not in the registry");
        }
    }

    /// Whether `state` is TNT that can be primed: vanilla's or a mod's.
    pub(crate) fn is_tnt(&self, state: BlockStateId) -> bool {
        self.states.contains(&state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `minecraft:tnt` with `[unstable=true]` after it, `minecraft:stone`,
    /// `a_mod:charge` with `[lit=true]` after it, `a_mod:other`.
    fn registry() -> BlockRegistry {
        let states = [
            "minecraft:tnt",
            "minecraft:tnt[unstable=true]",
            "minecraft:stone",
            "a_mod:charge",
            "a_mod:charge[lit=true]",
            "a_mod:other",
        ]
        .into_iter()
        .map(str::to_string)
        .collect();
        let defaults = [
            ("minecraft:tnt", 0),
            ("minecraft:stone", 2),
            ("a_mod:charge", 3),
            ("a_mod:other", 5),
        ]
        .into_iter()
        .map(|(name, id)| (name.to_string(), id))
        .collect();
        BlockRegistry::from_states_for_test(states, defaults)
    }

    #[test]
    fn a_mods_tnt_template_block_is_tnt_in_every_state_among_the_vanilla_ones() {
        let registry = registry();
        let mut kinds = TntKinds::default();
        kinds.add(
            &["a_mod:charge".to_string(), "gone:missing".to_string()],
            &registry,
        );
        assert!(kinds.is_tnt(BlockStateId(0)), "minecraft:tnt");
        assert!(
            kinds.is_tnt(BlockStateId(1)),
            "minecraft:tnt[unstable=true]"
        );
        assert!(!kinds.is_tnt(BlockStateId(2)), "minecraft:stone");
        assert!(kinds.is_tnt(BlockStateId(3)), "a_mod:charge");
        assert!(kinds.is_tnt(BlockStateId(4)), "a_mod:charge[lit=true]");
        assert!(!kinds.is_tnt(BlockStateId(5)), "a_mod:other");
        assert!(!TntKinds::default().is_tnt(BlockStateId(3)));
    }
}
