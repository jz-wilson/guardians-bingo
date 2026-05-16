use worker::{Request, Response, RouteContext};

use crate::kv;

pub async fn poll(req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let code = match ctx.param("code") {
        Some(c) => c.to_string(),
        None => return Response::error("missing code", 400),
    };

    let url = req.url()?;
    let mut version: u64 = 0;
    let mut player_id: Option<String> = None;
    for (k, v) in url.query_pairs() {
        match k.as_ref() {
            "version" => version = v.parse().unwrap_or(0),
            "player_id" => player_id = Some(v.into_owned()),
            _ => {}
        }
    }

    let kv_store = ctx.kv("BINGO_ROOMS")?;
    let state = match kv::get_state(&kv_store, &code).await? {
        Some(s) => s,
        None => return Response::error("room not found", 404),
    };

    if state.version == version {
        return Ok(Response::empty()?.with_status(304));
    }

    let player_summaries: Vec<serde_json::Value> = state
        .players
        .values()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "is_host": p.is_host,
                "marked_count": p.marked_cells.len(),
            })
        })
        .collect();

    let you = player_id
        .as_deref()
        .and_then(|pid| state.players.get(pid))
        .map(|p| {
            serde_json::json!({
                "board": p.board,
                "marked_cells": p.marked_cells,
            })
        });

    Response::from_json(&serde_json::json!({
        "version": state.version,
        "status": state.status,
        "events": state.events,
        "winners": state.winners,
        "players": player_summaries,
        "you": you,
    }))
}
