use rocket::http::Status;
use rocket::post;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{routes, Route};

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
struct Password {
    input: String,
}

type Validator = fn(&str) -> bool;

fn vowel_count() -> Validator {
    const VOWELS: &str = "aeiouy";
    |input| {
        let mut vowel_count = 0;

        for char in input.chars() {
            if VOWELS.contains(char) {
                vowel_count += 1;
            }
        }

        vowel_count >= 3
    }
}

fn has_repeated() -> Validator {
    |input| {
        let mut has_repeated = false;

        for (i, char) in input.char_indices() {
            if i != 0 && char.is_alphabetic() && input.chars().nth(i - 1) == Some(char) {
                has_repeated = true;
            }
        }

        has_repeated
    }
}

fn has_naughty_substring() -> Validator {
    const NAUGHTY_SUBSTRS: [&str; 4] = ["ab", "cd", "pq", "xy"];

    |input| {
        let mut has_naughty_substring = false;

        for i in 0..input.len() {
            if i != 0 && NAUGHTY_SUBSTRS.contains(&&input[i - 1..=i]) {
                has_naughty_substring = true;
            }
        }

        !has_naughty_substring
    }
}

fn validate_password(password: &Password, rules: &[Validator]) -> bool {
    rules.iter().all(|r| r(&password.input))
}

#[allow(non_camel_case_types)]
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
enum NiceOrNaughty {
    nice,
    naughty,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ValidationResult {
    result: NiceOrNaughty,
}

impl ValidationResult {
    fn nice() -> Self {
        Self {
            result: NiceOrNaughty::nice,
        }
    }

    fn naughty() -> Self {
        Self {
            result: NiceOrNaughty::naughty,
        }
    }
}

impl From<&ValidationResult> for Status {
    fn from(value: &ValidationResult) -> Self {
        match value.result {
            NiceOrNaughty::nice => Status::Ok,
            NiceOrNaughty::naughty => Status::BadRequest,
        }
    }
}

#[post("/nice", data = "<password>")]
fn nice(password: Json<Password>) -> (Status, Json<ValidationResult>) {
    let rules = [vowel_count(), has_repeated(), has_naughty_substring()];
    let res = if validate_password(&password, &rules) {
        ValidationResult::nice()
    } else {
        ValidationResult::naughty()
    };
    ((&res).into(), Json(res))
}

pub fn routes() -> Vec<Route> {
    routes![nice]
}
