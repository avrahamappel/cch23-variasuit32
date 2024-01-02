use rocket::local::blocking::Client;
use rocket::{Responder, Route};
use sqlx::PgPool;

#[allow(dead_code)]
pub fn test_client(routes: Vec<Route>) -> Client {
    Client::tracked(rocket::build().mount("/", routes)).unwrap()
}

#[allow(dead_code)]
pub fn test_client_stateful<T>(routes: Vec<Route>, state: T) -> Client
where
    T: Send + Sync + 'static,
{
    Client::tracked(rocket::build().mount("/", routes).manage(state)).unwrap()
}

#[derive(Responder, Debug)]
#[response(status = 500)]
pub struct Error {
    pub message: &'static str,
}

macro_rules! impl_from_error {
    ($type:ty, $msg:literal) => {
        impl From<$type> for Error {
            fn from(value: $type) -> Self {
                if cfg!(debug_assertions) {
                    dbg!(value);
                }

                Self { message: $msg }
            }
        }
    };
}

impl_from_error!(std::io::Error, "IO error");
impl_from_error!(image::ImageError, "Error processing image");
impl_from_error!(ulid::DecodeError, "Error decoding ULID");
impl_from_error!(std::num::ParseIntError, "Error parsing integer");
impl_from_error!(chrono::OutOfRange, "Number out of range of time type");
impl_from_error!(sqlx::Error, "Postgres error");
impl_from_error!(rocket::serde::json::serde_json::Error, "JSON error");
impl_from_error!(git2::Error, "Git error");

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(value: std::sync::PoisonError<T>) -> Self {
        if cfg!(debug_assertions) {
            dbg!(value);
        }
        Self {
            message: "Couldn't get lock",
        }
    }
}

pub struct DB {
    pub pool: PgPool,
}
