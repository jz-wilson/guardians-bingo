use worker::{event, Context, Env, Request, Response, Router};

mod bingo;
mod board;
mod codes;
mod cors;
mod handlers;
mod kv;
mod models;
mod rng;
mod util;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> worker::Result<Response> {
    let router = Router::new()
        .options_async("/*path", cors::preflight)
        .post_async("/api/rooms", handlers::rooms::create)
        .get_async("/api/rooms/:code", handlers::rooms::info)
        .post_async("/api/rooms/:code/join", handlers::rooms::join)
        .get_async("/api/rooms/:code/state", handlers::state::poll)
        .post_async("/api/rooms/:code/events", handlers::events::call)
        .get_async("/api/roster/:teamId", handlers::roster::get);

    router.run(req, env).await.map(cors::with_cors)
}
