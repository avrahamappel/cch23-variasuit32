#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::no_effect_underscore_binding)]

use rocket_dyn_templates::Template;
use sqlx::PgPool;

mod common;
mod day_0;
mod day_1;
mod day_11;
mod day_12;
mod day_13;
mod day_14;
mod day_15;
mod day_18;
mod day_19;
mod day_20;
mod day_21;
mod day_22;
mod day_4;
mod day_5;
mod day_6;
mod day_7;
mod day_8;

use common::DB;
use day_12::Timekeeper;
use day_19::ChatState;
use day_21::GeocodeApiKey;

#[allow(clippy::unused_async)]
#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: shuttle_runtime::SecretStore,
) -> shuttle_rocket::ShuttleRocket {
    let key = secrets.get("GEOCODE_API_KEY").expect("Couldn't get secret");
    let rocket = rocket::build()
        .mount("/", day_0::routes())
        .mount("/", day_1::routes())
        .mount("/", day_4::routes())
        .mount("/5", day_5::routes())
        .mount("/", day_6::routes())
        .mount("/", day_7::routes())
        .mount("/", day_8::routes())
        .mount("/", day_11::routes())
        .mount("/", day_12::routes())
        .mount("/", day_13::routes())
        .mount("/14", day_14::routes())
        .mount("/15", day_15::routes())
        .mount("/18", day_18::routes())
        .mount("/19", day_19::routes())
        .mount("/20", day_20::routes())
        .mount("/21", day_21::routes())
        .mount("/22", day_22::routes())
        .manage(Timekeeper::new())
        .manage(DB { pool })
        .manage(ChatState::new())
        .manage(GeocodeApiKey { key })
        .attach(Template::fairing());

    Ok(rocket.into())
}
