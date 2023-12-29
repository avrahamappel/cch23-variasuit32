use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use rocket::futures::channel::mpsc::{self, Receiver, Sender};
use rocket::futures::prelude::*;
use rocket::serde::json::serde_json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{get, post, routes, Route};
use rocket_ws as ws;
use ws::Message;

use crate::common::Error;

#[get("/ws/ping")]
fn ws_ping(ws: ws::WebSocket) -> ws::Stream![] {
    ws::Stream! { ws =>
        let mut game_started = false;
        for await msg in ws {
            if let Message::Text(text) = msg? {
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

struct User<'u> {
    id: &'u str,
    room_id: u32,
    channel: (Sender<RoomMessage<'u>>, Receiver<RoomMessage<'u>>),
}

impl<'u> User<'u> {
    fn new(id: &'u str, room_id: u32) -> Self {
        Self {
            id,
            room_id,
            channel: mpsc::channel(MSG_CHAR_LIMIT),
        }
    }
}

#[derive(Default)]
struct State<'u> {
    views: u32,
    users: Vec<&'u User<'u>>,
}

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct UserMessage {
    message: String,
}

impl TryFrom<&str> for UserMessage {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let msg = serde_json::from_str(value)?;
        Ok(msg)
    }
}


#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct RoomMessage<'u> {
    user: &'u str,
    message: String,
}


const MSG_CHAR_LIMIT: usize = 128;

impl<'u> RoomMessage<'u> {
    fn new(user: &'u str, message: String) -> Option<Self> {
        if message.chars().count() > MSG_CHAR_LIMIT {
            None
        } else {
            Some(Self { user, message })
        }
    }
}

impl<'u> From<RoomMessage<'u>> for Message {
    fn from(value: RoomMessage) -> Self {
        Message::text(
            serde_json::to_string(&value).expect("JSON serialization failed for some reason"),
        )
    }
}


pub struct ChatState {
    state: RwLock<State<'static>>,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(State::default()),
        }
    }

    fn read(&self) -> Result<RwLockReadGuard<'_, State>, Error> {
        let state = self.state.read()?;
        Ok(state)
    }

    fn write(&self) -> Result<RwLockWriteGuard<'_, State>, Error> {
        let state = self.state.write()?;
        Ok(state)
    }
}

#[post("/reset")]
fn reset(chat_state: &rocket::State<ChatState>) -> Result<(), Error> {
    let mut state = chat_state.write()?;
    state.views = 0;
    Ok(())
}

#[get("/views")]
fn views(chat_state: &rocket::State<ChatState>) -> Result<String, Error> {
    let views = chat_state.read()?.views;
    Ok(views.to_string())
}

#[get("/ws/room/<room_id>/user/<user_id>")]
fn ws_room<'r>(
    chat_state: &'r rocket::State<ChatState>,
    ws: ws::WebSocket,
    room_id: u32,
    user_id: &'r str,
) -> ws::Channel<'r> {
    ws.channel(move |mut stream| {
        Box::pin(async {
            // Connect user to room
            let mut state = chat_state
                .write()
                .expect("couldn't get write lock on state to add user");
            let user = User::new(user_id, room_id);
            state.users.push(&user);

            let res = future::join(
                Box::pin(async {
                    while let Some(msg) = stream.next().await {
                        if let Message::Text(text) = msg? {
                            // Sending a message to the room
                            if let Ok(user_msg) = UserMessage::try_from(text.as_str()) {
                                if let Some(room_msg) = RoomMessage::new(user_id, user_msg.message)
                                {
                                    if let Ok(state) = chat_state.read() {
                                        for user in state.users {
                                            user.channel.0.send(room_msg);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(())
                }),
                Box::pin(async {
                    // Receiving a message from the room
                    while let Some(room_msg) = user.channel.1.next().await {
                        stream.send(room_msg.into()).await;
                    }
                    Ok(())
                }),
            )
            .await;

            if let Ok(mut state) = chat_state.write() {
                state.users.retain(|u| u.id != user_id);
            }

            match res {
                (Err(e), _) | (_, Err(e)) => Err(e),
                _ => Ok(()),
            }
        })
    })
}

pub fn routes() -> Vec<Route> {
    routes![ws_ping, reset, views, ws_room]
}
