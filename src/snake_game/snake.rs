use super::{
    game_level::{SiblingPosition, Tile},
    GameError, GameLevel, MovementDirection, SnakeBehavior, TileType,
};

use std::collections::VecDeque;

#[derive(Debug)]
pub struct SnakeUnbounded {
    tail: VecDeque<usize>,
    movement_direction: MovementDirection,
}

enum MovementResult<'a> {
    Ok(&'a Tile),
    GrowOn(&'a Tile),
    SnakeCollision,
    LevelBoundary,
}

impl SnakeUnbounded {
    pub fn new(initial_direction: MovementDirection) -> Self {
        Self {
            tail: VecDeque::new(),
            movement_direction: initial_direction,
        }
    }

    fn try_move_to<'l>(&self, to_tile: &Option<&'l Tile>) -> MovementResult<'l> {
        match to_tile {
            Some(tile) => match tile.tile_type() {
                TileType::Empty => MovementResult::Ok(tile),
                TileType::Fruit => MovementResult::GrowOn(tile),
                TileType::Snake => MovementResult::SnakeCollision,
            },
            None => MovementResult::LevelBoundary,
        }
    }
}

impl SnakeBehavior for SnakeUnbounded {
    fn put_on(&mut self, level: &mut GameLevel, tail_size: usize) -> Result<(), GameError> {
        use MovementDirection::*;

        if tail_size < 1 {
            return Err(GameError::InvalidInternalState);
        }

        let d = level.level_coordinates();
        let x_center = d.x_max / 2;
        let y_center = d.y_max / 2;

        // get head tile
        let head = level
            .get_tile_on(x_center, y_center)
            .ok_or(GameError::InvalidInternalState)?;

        // set on which tiles snake lives
        let mut tail = Vec::new();
        tail.push(head.get_index());

        // add tail tiles to direction opposite to movement
        let grow_direction = match self.movement_direction {
            Up => SiblingPosition::Down,
            Down => SiblingPosition::Up,
            Left => SiblingPosition::Right,
            Right => SiblingPosition::Left,
        };

        let mut sibling = level.tile_sibling(head, grow_direction);
        for _ in 0..tail_size {
            let tail_tile = match sibling {
                Some(tile) => tile,
                None => return Err(GameError::InvalidInternalState),
            };
            tail.push(tail_tile.get_index());
            sibling = level.tile_sibling(tail_tile, grow_direction);
        }

        // put snake on selected tiles
        for tile_index in tail.iter() {
            let tile = level.get_tile_mut(*tile_index).unwrap();
            tile.set_to(TileType::Snake);
        }

        // save tail tiles to track movement
        self.tail = tail.into_iter().collect::<VecDeque<usize>>();

        Ok(())
    }

    fn make_move(&mut self, level: &mut GameLevel) -> Result<(), GameError> {
        use MovementDirection::*;

        let head_index = self.tail.front().ok_or(GameError::InvalidInternalState)?;
        let head = level
            .get_tile(*head_index)
            .ok_or(GameError::InvalidInternalState)?;

        let to_sibling = match self.movement_direction {
            Up => SiblingPosition::Up,
            Down => SiblingPosition::Down,
            Left => SiblingPosition::Left,
            Right => SiblingPosition::Right,
        };
        let to_tile = level.tile_sibling(head, to_sibling);

        let movement_result = self.try_move_to(&to_tile);

        // normal movement - true, set to false on snake grow.
        let mut delete_tail_end = true;

        let next_tile = match movement_result {
            MovementResult::Ok(tile) => level
                .get_tile_mut(tile.get_index())
                .ok_or(GameError::InvalidInternalState)?,
            MovementResult::GrowOn(tile) => {
                delete_tail_end = false; // make snake grow by one tile
                level
                    .get_tile_mut(tile.get_index())
                    .ok_or(GameError::InvalidInternalState)?
            }
            MovementResult::SnakeCollision => return Err(GameError::GameOver),
            // "globe" level behavior logic
            MovementResult::LevelBoundary => {
                let head_pos = level.get_tile_position(head);
                let level_dim = level.level_coordinates();

                match self.movement_direction {
                    Up => level.get_tile_mut_on(head_pos.x, level_dim.y_max),
                    Down => level.get_tile_mut_on(head_pos.x, level_dim.y_min),
                    Left => level.get_tile_mut_on(level_dim.x_max, head_pos.y),
                    Right => level.get_tile_mut_on(level_dim.x_min, head_pos.y),
                }
                .ok_or(GameError::InvalidInternalState)?
            }
        };

        next_tile.set_to(TileType::Snake);
        self.tail.push_front(next_tile.get_index());

        // delete last segment
        if delete_tail_end {
            let tail_end_index = self
                .tail
                .pop_back()
                .ok_or(GameError::InvalidInternalState)?;
            let tail_end = level
                .get_tile_mut(tail_end_index)
                .ok_or(GameError::InvalidInternalState)?;
            tail_end.set_to(TileType::Empty);
        }

        Ok(())
    }

    fn direction(&self) -> MovementDirection {
        self.movement_direction
    }

    fn set_direction(&mut self, new_direction: MovementDirection) -> Result<(), GameError> {
        if self.movement_direction.is_opposite_to(&new_direction) {
            return Err(GameError::GameOver);
        }
        self.movement_direction = new_direction;
        Ok(())
    }

    fn len(&self) -> usize {
        self.tail.len()
    }
}
