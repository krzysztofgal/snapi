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
