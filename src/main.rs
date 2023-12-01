use rocket::{get, routes, Responder};

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Responder)]
#[response(status = 500)]
struct Error {
    message: &'static str,
}

#[get("/-1/error")]
fn fake_error() -> Error {
    Error {
        message: "This is an error",
    }
}

#[get("/1/<num1>/<num2>")]
fn exclusive_cube(num1: u32, num2: u32) -> String {
    (num1 ^ num2).pow(3).to_string()
}

#[shuttle_runtime::main]
async fn main() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount("/", routes![index, fake_error, exclusive_cube]);

    Ok(rocket.into())
}
