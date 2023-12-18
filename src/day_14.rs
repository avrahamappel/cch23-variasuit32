use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{post, routes, Route};
use rocket_dyn_templates::Template;

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct TemplateContext {
    content: String,
}

#[post("/unsafe", data = "<context>")]
fn unescaped(context: Json<TemplateContext>) -> Template {
    Template::render("day_14/unsafe", &*context)
}

#[post("/safe", data = "<context>")]
fn escaped(context: Json<TemplateContext>) -> Template {
    Template::render("day_14/safe", &*context)
}

pub fn routes() -> Vec<Route> {
    routes![unescaped, escaped]
}
