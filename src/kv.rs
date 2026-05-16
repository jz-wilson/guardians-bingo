use worker::kv::KvStore;

use crate::models::{RoomMeta, RoomState};

const ROOM_TTL_SECS: u64 = 4 * 60 * 60;
const ROSTER_TTL_SECS: u64 = 60 * 60;

fn kv_err(e: worker::kv::KvError) -> worker::Error {
    worker::Error::RustError(format!("{e:?}"))
}

pub async fn get_meta(kv: &KvStore, code: &str) -> worker::Result<Option<RoomMeta>> {
    kv.get(&format!("room:{}:meta", code))
        .json::<RoomMeta>()
        .await
        .map_err(kv_err)
}

pub async fn put_meta(kv: &KvStore, m: &RoomMeta) -> worker::Result<()> {
    let json = serde_json::to_string(m)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;
    kv.put(&format!("room:{}:meta", m.code), json)
        .map_err(kv_err)?
        .expiration_ttl(ROOM_TTL_SECS)
        .execute()
        .await
        .map_err(kv_err)
}

pub async fn get_state(kv: &KvStore, code: &str) -> worker::Result<Option<RoomState>> {
    kv.get(&format!("room:{}:state", code))
        .json::<RoomState>()
        .await
        .map_err(kv_err)
}

pub async fn put_state(kv: &KvStore, code: &str, s: &RoomState) -> worker::Result<()> {
    let json = serde_json::to_string(s)
        .map_err(|e| worker::Error::RustError(e.to_string()))?;
    kv.put(&format!("room:{}:state", code), json)
        .map_err(kv_err)?
        .expiration_ttl(ROOM_TTL_SECS)
        .execute()
        .await
        .map_err(kv_err)
}

pub async fn get_roster_cached(kv: &KvStore, team_id: u32) -> worker::Result<Option<String>> {
    kv.get(&format!("roster:{}", team_id))
        .text()
        .await
        .map_err(kv_err)
}

pub async fn put_roster_cached(kv: &KvStore, team_id: u32, json: &str) -> worker::Result<()> {
    kv.put(&format!("roster:{}", team_id), json)
        .map_err(kv_err)?
        .expiration_ttl(ROSTER_TTL_SECS)
        .execute()
        .await
        .map_err(kv_err)
}
