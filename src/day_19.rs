use rocket::{get, routes, Route};
use rocket_ws as ws;

#[get("/ws/ping")]
fn ws_ping(ws: ws::WebSocket) -> ws::Stream![] {
    ws::Stream! { ws =>
        let mut game_started = false;
        for await msg in ws {
            if let ws::Message::Text(text) = msg? {
                if text == "serve" {
                    game_started = true;
                }

                if text == "ping" && game_started {
                    yield "pong".into()
                }
            }
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![ws_ping]
}
