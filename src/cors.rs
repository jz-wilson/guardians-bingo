use worker::{Headers, Request, Response, RouteContext};

pub async fn preflight(_req: Request, _ctx: RouteContext<()>) -> worker::Result<Response> {
    let h = Headers::new();
    h.set("Access-Control-Allow-Origin", "*")?;
    h.set("Access-Control-Allow-Methods", "GET,POST,OPTIONS")?;
    h.set("Access-Control-Allow-Headers", "Content-Type,Authorization")?;
    h.set("Access-Control-Max-Age", "86400")?;
    Ok(Response::empty()?.with_headers(h))
}

pub fn with_cors(mut resp: Response) -> Response {
    let h = resp.headers_mut();
    let _ = h.set("Access-Control-Allow-Origin", "*");
    let _ = h.set("Access-Control-Allow-Methods", "GET,POST,OPTIONS");
    let _ = h.set("Access-Control-Allow-Headers", "Content-Type,Authorization");
    resp
}
