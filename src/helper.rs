use super::MovementDirection;
use std::collections::HashMap;

pub fn get_most_move_occurrences_in(
    move_iter: impl Iterator<Item = MovementDirection>,
) -> (usize, HashMap<MovementDirection, usize>) {
    let mut selected_count = HashMap::with_capacity(4);
    for mov in move_iter {
        let entry = selected_count.entry(mov).or_insert(0usize);
        *entry += 1;
    }

    let most = selected_count
        .iter()
        .max_by(|(_, a), (_, b)| a.cmp(b))
        .map(|(_, v)| *v)
        .unwrap_or_default();

    (most, selected_count)
}

#[cfg(test)]
mod tests {

    #[test]
    fn move_occurrence_counting() {
        use super::MovementDirection::*;

        let moves = vec![Up, Down, Left, Left, Down, Right];
        let (most, moves_map) = super::get_most_move_occurrences_in(moves.into_iter());
        assert_eq!(most, 2);

        let moves: Vec<_> = moves_map
            .into_iter()
            .filter_map(|(mov, c)| if c == most { Some(mov) } else { None })
            .collect();

        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Down));
        assert!(moves.contains(&Left));
    }
}
