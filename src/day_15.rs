use rocket::http::Status;
use rocket::post;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::{routes, Route};

/// I don't like how this came out
/// Rocket doesn't support validation of json request bodies out of the box
/// Also returning an error from a `FromData` tries to forward to the next handler, there doesn't
/// seem to be a way to return a response directly from a request guard
/// Next time I would try using the `validator` crate and not rely on the framework

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
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'static str>,
}

impl ValidationResult {
    fn nice() -> Self {
        Self {
            result: NiceOrNaughty::nice,
            reason: None,
        }
    }

    fn naughty() -> Self {
        Self {
            result: NiceOrNaughty::naughty,
            reason: None,
        }
    }

    fn naughty_with_reason(reason: &'static str) -> Self {
        Self {
            result: NiceOrNaughty::naughty,
            reason: Some(reason),
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

type Reason = Result<(), (Status, &'static str)>;
type ValidatorWithReason = fn(&str) -> Reason;

fn validate_password_with_reason(password: &Password, rules: &[ValidatorWithReason]) -> Reason {
    rules.iter().try_for_each(|r| r(&password.input))
}

fn eight_chars() -> ValidatorWithReason {
    |input| {
        if input.len() >= 8 {
            Ok(())
        } else {
            Err((Status::BadRequest, "8 chars"))
        }
    }
}

fn upper_lower_digit() -> ValidatorWithReason {
    |input| {
        let (uppers, lowers, digits) =
            input.chars().fold((false, false, false), |mut scores, ch| {
                if ch.is_uppercase() {
                    scores.0 = true;
                }
                if ch.is_lowercase() {
                    scores.1 = true;
                }
                if ch.is_numeric() {
                    scores.2 = true;
                }
                scores
            });
        if uppers && lowers && digits {
            Ok(())
        } else {
            Err((Status::BadRequest, "more types of chars"))
        }
    }
}

fn five_digits() -> ValidatorWithReason {
    |input| {
        let count = input.chars().filter(|c| c.is_numeric()).count();

        if count >= 5 {
            Ok(())
        } else {
            Err((Status::BadRequest, "55555"))
        }
    }
}

fn math_is_hard() -> ValidatorWithReason {
    |input| {
        let groups = input.chars().fold(vec![String::new()], |mut groups, c| {
            if c.is_numeric() {
                groups
                    .last_mut()
                    .expect("Groups should never be empty")
                    .push(c);
            } else {
                groups.push(String::new());
            }
            groups
        });

        let sum: u32 = groups
            .iter()
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<u32>().ok())
            .sum();

        if sum == 2023 {
            Ok(())
        } else {
            Err((Status::BadRequest, "math is hard"))
        }
    }
}

fn joyful() -> ValidatorWithReason {
    |input| {
        let word = "joy";

        // find number of instances and check none are > 1
        let one_of_each = word
            .chars()
            .map(|c| input.chars().filter(|cc| c == *cc).count())
            .all(|n| n == 1);

        // find index of each char and make sure they are in ascending order
        let in_order = {
            let indices: Vec<_> = word
                .chars()
                .map(|c| input.chars().position(|cc| c == cc))
                .collect();
            indices.windows(2).all(|pair| pair[0] < pair[1])
        };

        if one_of_each && in_order {
            Ok(())
        } else {
            Err((Status::NotAcceptable, "not joyful enough"))
        }
    }
}

fn sandwich() -> ValidatorWithReason {
    |input| {
        let sandwiches = input
            .bytes()
            .enumerate()
            .filter(|(i, c)| {
                c.is_ascii_alphabetic()
                    && matches!(
                    (input.as_bytes().get(i + 1), input.as_bytes().get(i + 2)),
                    (Some(d), Some(e)) if d != c && e == c)
            })
            .count();

        if sandwiches >= 1 {
            Ok(())
        } else {
            Err((Status::UnavailableForLegalReasons, "illegal: no sandwich"))
        }
    }
}

fn math_unicode() -> ValidatorWithReason {
    |input| {
        let passes = input
            .chars()
            .any(|c| ('\u{2980}'..='\u{2BFF}').contains(&c));

        if passes {
            Ok(())
        } else {
            Err((Status::RangeNotSatisfiable, "outranged"))
        }
    }
}

#[post("/game", data = "<password>")]
fn game(password: Json<Password>) -> (Status, Json<ValidationResult>) {
    let rules = [
        eight_chars(),
        upper_lower_digit(),
        five_digits(),
        math_is_hard(),
        joyful(),
        sandwich(),
        math_unicode(),
    ];
    if let Err((status, reason)) = validate_password_with_reason(&password, &rules) {
        (status, Json(ValidationResult::naughty_with_reason(reason)))
    } else {
        (Status::Ok, Json(ValidationResult::nice()))
    }
}

pub fn routes() -> Vec<Route> {
    routes![nice, game]
}
