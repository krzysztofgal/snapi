use super::{FruitBehavior, GameError, GameLevel, TileType};

pub struct FruitRandomLimited {
    limit: usize,
    chance: f64,
}

impl FruitRandomLimited {
    pub fn new(limit: usize, chance: f64) -> Self {
        if !(0.01..=1.0).contains(&chance) {
            panic!("Invalid configuration: chance must be in range of 0.01 - 1.00")
        }

        Self { limit, chance }
    }
}

impl FruitBehavior for FruitRandomLimited {
    fn put_on(&mut self, level: &mut GameLevel) -> Result<(), GameError> {
        use rand::prelude::*;

        // count fruits on level
        let count_fruits = level
            .level()
            .iter()
            .filter(|t| matches!(t.tile_type(), TileType::Fruit))
            .count();

        // if under limit then draw a chance to put one fruit
        if count_fruits < self.limit {
            let draw = thread_rng().gen_range(0.01..1.0);
            if self.chance >= draw {
                let empty_tiles = level
                    .level()
                    .iter()
                    .filter(|t| matches!(t.tile_type(), TileType::Empty))
                    .collect::<Vec<_>>();

                // there is nowhere to put fruit
                if empty_tiles.is_empty() {
                    return Ok(());
                }

                // put fruit on empty field
                let random_index = thread_rng().gen_range(0..empty_tiles.len());
                let tile = level.get_tile_mut(random_index).unwrap();
                tile.set_to(TileType::Fruit);
            }
        }

        Ok(())
    }
}
