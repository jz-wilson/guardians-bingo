use serde::Deserialize;
use worker::{Fetch, Method, Request, Response, RouteContext};
use worker::kv::KvStore;

use crate::{kv, models::RosterPlayer};

#[derive(Deserialize)]
struct MlbRosterResponse {
    roster: Vec<MlbPlayer>,
}

#[derive(Deserialize)]
struct MlbPlayer {
    person: MlbPerson,
    position: MlbPosition,
}

#[derive(Deserialize)]
struct MlbPerson {
    id: u32,
    #[serde(rename = "fullName")]
    full_name: String,
}

#[derive(Deserialize)]
struct MlbPosition {
    abbreviation: String,
}

pub async fn fetch_cached(kv_store: &KvStore, team_id: u32) -> worker::Result<Vec<RosterPlayer>> {
    if let Some(cached) = kv::get_roster_cached(kv_store, team_id).await? {
        return serde_json::from_str(&cached)
            .map_err(|e| worker::Error::RustError(e.to_string()));
    }

    let url = format!(
        "https://statsapi.mlb.com/api/v1/teams/{}/roster/active",
        team_id
    );
    let req = Request::new(&url, Method::Get)?;
    let mut resp = Fetch::Request(req).send().await?;
    let mlb: MlbRosterResponse = resp.json().await?;

    let players: Vec<RosterPlayer> = mlb
        .roster
        .into_iter()
        .map(|p| RosterPlayer {
            id: p.person.id,
            name: p.person.full_name,
            position: p.position.abbreviation,
        })
        .collect();

    let json = serde_json::to_string(&players)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;
    kv::put_roster_cached(kv_store, team_id, &json).await?;

    Ok(players)
}

pub async fn get(_req: Request, ctx: RouteContext<()>) -> worker::Result<Response> {
    let team_id: u32 = ctx
        .param("teamId")
        .map(String::as_str)
        .unwrap_or("114")
        .parse()
        .map_err(|_| worker::Error::RustError("invalid teamId".into()))?;

    let kv_store = ctx.kv("BINGO_ROOMS")?;
    let players = fetch_cached(&kv_store, team_id).await?;
    Response::from_json(&players)
}
