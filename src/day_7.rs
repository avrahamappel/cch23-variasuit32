use std::collections::HashMap;

use base64::engine::general_purpose;
use base64::Engine;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::json::{serde_json, Json};
use rocket::serde::{Deserialize, Serialize};
use rocket::{get, routes, Request, Route};

use crate::common::Error;

struct CookieHeader {
    value: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CookieHeader {
    type Error = Error;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let error_outcome = || {
            Outcome::Error((
                Status::BadRequest,
                Error {
                    message: "Missing or invalid `Cookie` header",
                },
            ))
        };

        if let Some(h) = req.headers().get_one("Cookie") {
            if &h[0..7] != "recipe=" {
                return error_outcome();
            }

            let recipe = &h[7..];

            if let Ok(bytes) = general_purpose::STANDARD.decode(recipe) {
                let value = String::from_utf8_lossy(&bytes).into_owned();

                return Outcome::Success(CookieHeader { value });
            }
        }

        error_outcome()
    }
}

#[get("/7/decode")]
fn cookie_recipe(cookie_header: CookieHeader) -> String {
    cookie_header.value
}

type Ingredients = HashMap<String, u64>;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Recipe {
    recipe: Ingredients,
    pantry: Ingredients,
}

impl Recipe {
    fn bake(mut self) -> AfterBake {
        let cookies = self
            .recipe
            .iter()
            .map(|(ing, r_amt)| {
                self.pantry
                    .get(ing)
                    .map_or(0, |p_amt| if *r_amt == 0 { 0 } else { p_amt / r_amt })
            })
            .filter(|amt| *amt > 0)
            .min()
            .unwrap_or(0);

        for (ing, p_amt) in &mut self.pantry {
            if let Some(r_amt) = self.recipe.get(ing) {
                *p_amt -= r_amt * cookies;
            }
        }

        AfterBake {
            cookies,
            pantry: self.pantry,
        }
    }
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct AfterBake {
    cookies: u64,
    pantry: Ingredients,
}

#[get("/7/bake")]
fn bake_cookies(header: CookieHeader) -> Result<Json<AfterBake>, Error> {
    let recipe: Recipe = serde_json::from_str(&header.value).map_err(|e| {
        if cfg!(debug_assertions) {
            dbg!(e);
        }
        Error {
            message: "Invalid JSON",
        }
    })?;

    Ok(Json(recipe.bake()))
}

pub fn routes() -> Vec<Route> {
    routes![cookie_recipe, bake_cookies,]
}

#[cfg(test)]
mod tests {
    use crate::common::test_client;

    #[test]
    fn cookie_recipe_test() {
        use rocket::http::Header;

        let client = test_client(super::routes());
        let response = client
            .get("/7/decode")
            .header(Header::new(
                "Cookie",
                "recipe=eyJmbG91ciI6MTAwLCJjaG9jb2xhdGUgY2hpcHMiOjIwfQ==",
            ))
            .dispatch();

        assert_eq!(
            r#"{"flour":100,"chocolate chips":20}"#,
            response.into_string().unwrap()
        );
    }

    #[test]
    fn bake_cookies_test() {
        use rocket::http::Header;

        let client = test_client(super::routes());

        for (expected, header) in [
        (vec![
            r#""cookies":4"#,
            r#""flour":5"#,
            r#""butter":2002"#,
            r#""baking powder":825"#,
            r#""chocolate chips":257"#,
            r#""sugar":307"#,
        ], "recipe=eyJyZWNpcGUiOnsiZmxvdXIiOjk1LCJzdWdhciI6NTAsImJ1dHRlciI6MzAsImJha2luZyBwb3dkZXIiOjEwLCJjaG9jb2xhdGUgY2hpcHMiOjUwfSwicGFudHJ5Ijp7ImZsb3VyIjozODUsInN1Z2FyIjo1MDcsImJ1dHRlciI6MjEyMiwiYmFraW5nIHBvd2RlciI6ODY1LCJjaG9jb2xhdGUgY2hpcHMiOjQ1N319"),
        (vec![
            r#""cookies":0"#,
            r#""cobblestone":64"#,
            r#""stick":4"#,
        ], "recipe=eyJyZWNpcGUiOnsic2xpbWUiOjl9LCJwYW50cnkiOnsiY29iYmxlc3RvbmUiOjY0LCJzdGljayI6IDR9fQ==")
    ] {
        eprintln!("{header}");
        let response = client.get("/7/bake").header(Header::new("Cookie", header)).dispatch();

        let body = response.into_string().unwrap();

        for fragment in expected {
            assert!(
                body.contains(fragment),
                "Failed asserting that '{body}' contains '{fragment}'"
            );
        }
    }
    }
}
