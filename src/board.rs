use crate::models::{Board, Cell, RosterPlayer};
use crate::rng::{hash_seed, shuffle, XorShift64};

pub fn build_board(
    room_code: &str,
    player_id: &str,
    roster: &[RosterPlayer],
    event_types: &[String],
) -> Board {
    let seed = hash_seed(&format!("{room_code}|{player_id}"));
    let mut rng = XorShift64(seed | 1);

    let mut pool: Vec<(String, String)> = roster
        .iter()
        .flat_map(|p| event_types.iter().map(move |e| (p.name.clone(), e.clone())))
        .collect();
    shuffle(&mut pool, &mut rng);

    let mut cells: Vec<Cell> = pool
        .into_iter()
        .take(24)
        .map(|(player_name, event_type)| Cell {
            player_name,
            event_type,
            is_free: false,
        })
        .collect();
    cells.insert(
        12,
        Cell {
            player_name: String::new(),
            event_type: String::from("FREE"),
            is_free: true,
        },
    );

    Board { cells, seed }
}
