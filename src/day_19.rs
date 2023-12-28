use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

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

struct Room {
    id: u32,
    users: Vec<String>,
}

impl Room {
    fn new(id: u32) -> Self {
        Self { id, users: vec![] }
    }
}

#[derive(Default)]
struct State {
    views: u32,
    rooms: Vec<Room>,
}

#[derive(Serialize, Deserialize)]
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

impl From<UserMessage> for Message {
    fn from(value: UserMessage) -> Self {
        Message::text(
            serde_json::to_string(&value).expect("JSON serialization failed for some reason"),
        )
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct RoomMessage {
    user: String,
    message: String,
}

impl TryFrom<&str> for RoomMessage {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let msg = serde_json::from_str(value)?;
        Ok(msg)
    }
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

impl State {
    fn get_room_mut<'a>(&'a mut self, id: u32) -> &'a mut Room {
        match self.rooms.iter_mut().find(|r| r.id == id) {
            Some(room) => room,
            None => {
                let mut room = Room::new(id);
                let borrow = &mut room;
                self.rooms.push(room);
                borrow
            }
        }
    }

    fn broadcast_msg_to_user(&mut self, msg: &RoomMessage, user_id: &String) {
        if self
            .rooms
            .iter()
            .any(|r| r.users.contains(user_id) && r.users.contains(&msg.user))
        {
            self.views += 1;
        }
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
        let mut state = self.state.write()?;
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
) -> ws::Stream!['r] {
    ws::Stream! { ws =>
        // Connect user to room
        if let Ok(mut state) = chat_state.write() {
            let room = state.get_room_mut(room_id);
            room.users.push(user_id.clone());
        }

        for await msg in ws {
            if let Message::Text(text) = msg? {
                // Sending a message to the room
                if let Ok(user_msg) = UserMessage::try_from(text.as_str()) {
                    if let Some(room_msg) = RoomMessage::new(user_id.clone(), user_msg.message) {
                        yield room_msg.into();
                    }
                }

                // Receiving a message from the room
                if let Ok(room_msg) = RoomMessage::try_from(text.as_str()) {
                    if let Ok(mut state) = chat_state.write() {
                        state.broadcast_msg_to_user(&room_msg, &user_id);
                    }
                }
            }
        }

        if let Ok(mut state) = chat_state.write() {
            for room in &mut state.rooms {
                room.users.retain(|u|u != &user_id);
            }
        }
    }
}

pub fn routes() -> Vec<Route> {
    routes![ws_ping, reset, views, ws_room]
}
