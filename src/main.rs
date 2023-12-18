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
mod day_4;
mod day_6;
mod day_7;
mod day_8;

use day_12::Timekeeper;
use day_13::DB;

#[allow(clippy::unused_async)]
#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build()
        .mount("/", day_0::routes())
        .mount("/", day_1::routes())
        .mount("/", day_4::routes())
        .mount("/", day_6::routes())
        .mount("/", day_7::routes())
        .mount("/", day_8::routes())
        .mount("/", day_11::routes())
        .mount("/", day_12::routes())
        .mount("/", day_13::routes())
        .mount("/14", day_14::routes())
        .mount("/15", day_15::routes())
        .manage(Timekeeper::new())
        .manage(DB { pool })
        .attach(Template::fairing());

    Ok(rocket.into())
}
