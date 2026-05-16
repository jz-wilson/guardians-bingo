use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Waiting,
    Active,
    Finished,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoomMeta {
    pub code: String,
    pub host_name: String,
    pub host_token: String,
    pub team_id: u32,
    pub roster: Vec<RosterPlayer>,
    pub event_types: Vec<String>,
    pub created_at: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoomState {
    pub version: u64,
    pub status: Status,
    pub players: HashMap<String, Player>,
    pub events: Vec<CalledEvent>,
    pub winners: Vec<Winner>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub token: String,
    pub board: Board,
    pub marked_cells: HashSet<usize>,
    pub is_host: bool,
    pub connected_at: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Board {
    pub cells: Vec<Cell>,
    pub seed: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Cell {
    pub player_name: String,
    pub event_type: String,
    pub is_free: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CalledEvent {
    pub index: usize,
    pub player_name: String,
    pub event_type: String,
    pub timestamp: u64,
    pub undone: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RosterPlayer {
    pub id: u32,
    pub name: String,
    pub position: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Winner {
    pub player_id: String,
    pub player_name: String,
    pub line: [usize; 5],
    pub at_event_index: usize,
}
