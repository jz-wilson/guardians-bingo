use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use worker::{Request, Response, RouteContext};

use crate::models::{CalledEvent, Status, Winner};
use crate::{bingo, kv, util};

pub async fn call(mut req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let code = match ctx.param("code") {
        Some(c) => c.to_string(),
        None => return Response::error("missing code", 400),
    };

    let auth = req
        .headers()
        .get("Authorization")?
        .unwrap_or_default();
    let bearer = auth.strip_prefix("Bearer ").unwrap_or("");
    if bearer.is_empty() {
        return Response::error("unauthorized", 401);
    }

    let kv_store = ctx.kv("BINGO_ROOMS")?;
    let meta = match kv::get_meta(&kv_store, &code).await? {
        Some(m) => m,
        None => return Response::error("room not found", 404),
    };

    if !util::constant_time_eq(bearer, &meta.host_token) {
        return Response::error("unauthorized", 401);
    }

    #[derive(Deserialize)]
    struct Body {
        player_name: String,
        event_type: String,
    }
    let body: Body = req.json().await?;

    let et_lower = body.event_type.to_lowercase();
    if !meta
        .event_types
        .iter()
        .any(|e| e.to_lowercase() == et_lower)
    {
        return Response::error("unknown event_type", 400);
    }

    let pn_lower = body.player_name.to_lowercase();
    if !meta.roster.iter().any(|p| p.name.to_lowercase() == pn_lower) {
        return Response::error("unknown player_name", 400);
    }

    let mut state = match kv::get_state(&kv_store, &code).await? {
        Some(s) => s,
        None => return Response::error("room not found", 404),
    };

    let event_index = state.events.len();
    state.events.push(CalledEvent {
        index: event_index,
        player_name: body.player_name.clone(),
        event_type: body.event_type.clone(),
        timestamp: util::now_ms(),
        undone: false,
    });

    // Mark matching cells for every player
    let mut new_marks: HashMap<String, Vec<usize>> = HashMap::new();
    for (pid, player) in state.players.iter_mut() {
        let mut newly = vec![];
        for (idx, cell) in player.board.cells.iter().enumerate() {
            if !cell.is_free
                && cell.player_name.to_lowercase() == pn_lower
                && cell.event_type.to_lowercase() == et_lower
                && player.marked_cells.insert(idx)
            {
                newly.push(idx);
            }
        }
        if !newly.is_empty() {
            new_marks.insert(pid.clone(), newly);
        }
    }

    // Check for new winners
    let existing: HashSet<String> = state.winners.iter().map(|w| w.player_id.clone()).collect();
    let mut new_winners: Vec<(String, String, [usize; 5])> = vec![];
    for (pid, player) in &state.players {
        if !existing.contains(pid) {
            if let Some(line) = bingo::winning_line(&player.marked_cells) {
                new_winners.push((pid.clone(), player.name.clone(), line));
            }
        }
    }
    for (pid, name, line) in &new_winners {
        state.winners.push(Winner {
            player_id: pid.clone(),
            player_name: name.clone(),
            line: *line,
            at_event_index: event_index,
        });
    }
    if !new_winners.is_empty() {
        state.status = Status::Finished;
    } else if state.status == Status::Waiting {
        state.status = Status::Active;
    }

    state.version += 1;
    kv::put_state(&kv_store, &code, &state).await?;

    Response::from_json(&serde_json::json!({
        "event_index": event_index,
        "version": state.version,
        "marks": new_marks,
        "winners": state.winners,
    }))
}
