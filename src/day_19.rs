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

struct User {
    id: String,
    rx: Receiver<RoomMessage>,
}

impl User {
    fn new(id: String) -> (Self, Sender<RoomMessage>) {
        let (tx, rx) = mpsc::channel(MSG_CHAR_LIMIT);
        (Self { id, rx }, tx)
    }
}

struct RoomUser {
    id: String,
    room_id: u32,
    tx: Sender<RoomMessage>,
}

impl RoomUser {
    fn new(id: String, room_id: u32, tx: Sender<RoomMessage>) -> Self {
        Self { id, room_id, tx }
    }
}

#[derive(Default)]
struct State {
    views: u32,
    users: Vec<RoomUser>,
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

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
struct RoomMessage {
    user: String,
    message: String,
}

const MSG_CHAR_LIMIT: usize = 128;

impl RoomMessage {
    fn new(user: String, message: String) -> Option<Self> {
        if message.chars().count() > MSG_CHAR_LIMIT {
            None
        } else {
            Some(Self { user, message })
        }
    }
}

impl From<RoomMessage> for Message {
    fn from(value: RoomMessage) -> Self {
        Message::text(
            serde_json::to_string(&value).expect("JSON serialization failed for some reason"),
        )
    }
}

pub struct ChatState {
    state: RwLock<State>,
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
    user_id: String,
) -> ws::Channel<'r> {
    ws.channel(move |stream| {
        let (mut sink, mut stream) = stream.split();
        Box::pin(async move {
            // Connect user to room
            let mut user = {
                let mut state = chat_state
                    .write()
                    .expect("couldn't get write lock on state to add user");
                let (user, tx) = User::new(user_id);
                state
                    .users
                    .push(RoomUser::new(user.id.clone(), room_id, tx.clone()));
                user
            };

            let res = future::join(
                // Sending a message to the room
                Box::pin(async {
                    while let Some(msg) = stream.next().await {
                        if let Message::Text(text) = msg? {
                            if let Ok(user_msg) = UserMessage::try_from(text.as_str()) {
                                if let Some(room_msg) =
                                    RoomMessage::new(user.id.clone(), user_msg.message)
                                {
                                    let txs: Result<Vec<_>, Error> =
                                        chat_state.read().map(|state| {
                                            state
                                                .users
                                                .iter()
                                                .filter(|u| u.room_id == room_id)
                                                .map(|u| u.tx.clone())
                                                .collect()
                                        });

                                    if let Ok(txs) = txs {
                                        for mut tx in txs {
                                            tx.send(room_msg.clone())
                                                .await
                                                .expect("couldn't send msg to user");
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Log user out of room when socket closes
                    if let Ok(mut state) = chat_state.write() {
                        eprintln!("logging out {}", &user.id);
                        state.users.retain(|u| u.id != user.id);
                    }

                    Ok(())
                }),
                // Receiving a message from the room
                Box::pin(async {
                    while let Some(room_msg) = user.rx.next().await {
                        sink.send(room_msg.into())
                            .await
                            .expect("couldn't receive msg from room");
                    }
                    Ok(())
                }),
            )
            .await;

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
