use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use worker::{Request, Response, RouteContext};

use crate::models::{Player, RoomMeta, RoomState, Status};
use crate::{board, codes, kv, util};
use crate::handlers::roster;

fn default_event_types() -> Vec<String> {
    vec![
        "single", "double", "triple", "home run", "strikeout",
        "walk", "stolen base", "error", "double play", "hit by pitch",
        "wild pitch", "passed ball", "sacrifice fly", "ground out", "fly out",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

pub async fn create(mut req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    #[derive(Deserialize)]
    struct Body {
        team_id: u32,
        host_name: String,
    }
    let body: Body = req.json().await?;
    let kv_store = ctx.kv("BINGO_ROOMS")?;
    let roster = roster::fetch_cached(&kv_store, body.team_id).await?;
    let event_types = default_event_types();

    let code = {
        let mut result = None;
        for _ in 0..5 {
            let c = codes::generate_code()?;
            if kv::get_meta(&kv_store, &c).await?.is_none() {
                result = Some(c);
                break;
            }
        }
        match result {
            Some(c) => c,
            None => return Response::error("could not generate unique room code", 500),
        }
    };

    let host_token = util::generate_token()?;
    let host_id = util::generate_id()?;
    let host_player_token = util::generate_token()?;
    let host_board = board::build_board(&code, &host_id, &roster, &event_types);
    let mut host_marked = HashSet::new();
    host_marked.insert(12usize);
    let now = util::now_ms();

    let host_player = Player {
        id: host_id.clone(),
        name: body.host_name.clone(),
        token: host_player_token.clone(),
        board: host_board,
        marked_cells: host_marked,
        is_host: true,
        connected_at: now,
    };

    let meta = RoomMeta {
        code: code.clone(),
        host_name: body.host_name,
        host_token: host_token.clone(),
        team_id: body.team_id,
        roster,
        event_types,
        created_at: now,
    };

    let mut players = HashMap::new();
    players.insert(host_id.clone(), host_player.clone());

    let state = RoomState {
        version: 1,
        status: Status::Waiting,
        players,
        events: vec![],
        winners: vec![],
    };

    kv::put_meta(&kv_store, &meta).await?;
    kv::put_state(&kv_store, &code, &state).await?;

    Response::from_json(&serde_json::json!({
        "room_code": code,
        "host_token": host_token,
        "host_player_id": host_id,
        "host_player_token": host_player_token,
        "board": host_player.board,
        "ws_url": null,
    }))
}

pub async fn info(_req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let code = match ctx.param("code") {
        Some(c) => c.to_string(),
        None => return Response::error("missing code", 400),
    };
    let kv_store = ctx.kv("BINGO_ROOMS")?;
    let meta = match kv::get_meta(&kv_store, &code).await? {
        Some(m) => m,
        None => return Response::error("room not found", 404),
    };
    let state = match kv::get_state(&kv_store, &code).await? {
        Some(s) => s,
        None => return Response::error("room not found", 404),
    };

    Response::from_json(&serde_json::json!({
        "host_name": meta.host_name,
        "player_count": state.players.len(),
        "status": state.status,
        "team_name": "Guardians",
        "event_types": meta.event_types,
    }))
}

pub async fn join(mut req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    #[derive(Deserialize)]
    struct Body {
        name: String,
    }
    let code = match ctx.param("code") {
        Some(c) => c.to_string(),
        None => return Response::error("missing code", 400),
    };
    let body: Body = req.json().await?;
    let kv_store = ctx.kv("BINGO_ROOMS")?;

    let meta = match kv::get_meta(&kv_store, &code).await? {
        Some(m) => m,
        None => return Response::error("room not found", 404),
    };
    let mut state = match kv::get_state(&kv_store, &code).await? {
        Some(s) => s,
        None => return Response::error("room not found", 404),
    };

    if state.status == Status::Finished {
        return Response::error("room is finished", 409);
    }

    let name_lower = body.name.to_lowercase();
    let conflict = state
        .players
        .values()
        .any(|p| p.name.to_lowercase() == name_lower);
    if conflict {
        return Response::error("name already taken", 409);
    }

    let player_id = util::generate_id()?;
    let player_token = util::generate_token()?;
    let player_board = board::build_board(&code, &player_id, &meta.roster, &meta.event_types);
    let mut marked_cells = HashSet::new();
    marked_cells.insert(12usize);

    let player = Player {
        id: player_id.clone(),
        name: body.name,
        token: player_token.clone(),
        board: player_board.clone(),
        marked_cells,
        is_host: false,
        connected_at: util::now_ms(),
    };

    state.players.insert(player_id.clone(), player);
    state.version += 1;
    kv::put_state(&kv_store, &code, &state).await?;

    Response::from_json(&serde_json::json!({
        "player_id": player_id,
        "player_token": player_token,
        "board": player_board,
    }))
}
