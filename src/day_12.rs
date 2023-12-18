use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Instant;

use chrono::{DateTime, Datelike, Utc, Weekday};
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::{get, post, routes, Route, State};
use ulid::Ulid;
use uuid::Uuid;

use crate::common::Error;

pub struct Timekeeper {
    store: RwLock<HashMap<String, Instant>>,
}

impl Timekeeper {
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
        }
    }

    fn put(&self, string: String) -> Result<(), Error> {
        self.store.write()?.insert(string, Instant::now());
        Ok(())
    }

    fn get(&self, string: String) -> Option<u64> {
        self.store
            .read()
            .ok()?
            .get(&string)
            .map(|inst| inst.elapsed().as_secs())
    }
}

#[post("/12/save/<string>")]
fn store_string(timekeeper: &State<Timekeeper>, string: String) -> Result<(), Error> {
    timekeeper.put(string)?;
    Ok(())
}

#[get("/12/load/<string>")]
fn get_string(timekeeper: &State<Timekeeper>, string: String) -> Option<String> {
    timekeeper.get(string).map(|u| u.to_string())
}

#[post("/12/ulids", data = "<ulids>")]
fn ulid2uuid(ulids: Json<Vec<&str>>) -> Result<Json<Vec<String>>, Error> {
    let try_uuids: Result<Vec<_>, _> = ulids
        .iter()
        .map(|s| {
            Ulid::from_string(s).map(|ulid| {
                let bytes = ulid.to_bytes();
                Uuid::from_bytes(bytes).to_string()
            })
        })
        .rev()
        .collect();

    Ok(Json(try_uuids?))
}

#[derive(Serialize, Default)]
#[serde(crate = "rocket::serde")]
struct UlidsAnalysis {
    #[serde(rename = "christmas eve")]
    xmas_eve: usize,
    weekday: usize,
    #[serde(rename = "in the future")]
    future: usize,
    #[serde(rename = "LSB is 1")]
    lsb1: usize,
}

impl UlidsAnalysis {
    fn new(ulids: &[Ulid], weekday: Weekday) -> Self {
        ulids.iter().fold(Self::default(), |mut analysis, ulid| {
            let datetime: DateTime<Utc> = ulid.datetime().into();

            if datetime.month() == 12 && datetime.day() == 24 {
                analysis.xmas_eve += 1;
            }
            if datetime.weekday() == weekday {
                analysis.weekday += 1;
            }
            if datetime > Utc::now() {
                analysis.future += 1;
            }
            if (ulid.random()).trailing_ones() >= 1 {
                analysis.lsb1 += 1;
            }
            analysis
        })
    }
}

#[post("/12/ulids/<weekday>", data = "<ulids>")]
fn ulids_analyze(weekday: u8, ulids: Json<Vec<&str>>) -> Result<Json<UlidsAnalysis>, Error> {
    let weekday = Weekday::try_from(weekday)?;
    let ulids = ulids
        .iter()
        .map(|s| Ulid::from_string(s))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(UlidsAnalysis::new(&ulids, weekday)))
}

pub fn routes() -> Vec<Route> {
    routes![store_string, get_string, ulid2uuid, ulids_analyze,]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::test_client_stateful;
    use rocket::serde::json::serde_json::json;

    #[test]
    fn ulid2uuid_test() {
        let client = test_client_stateful(routes(), Timekeeper::new());
        let response = client
        .post("/12/ulids")
        .body(
            r#"["01BJQ0E1C3Z56ABCD0E11HYX4M","01BJQ0E1C3Z56ABCD0E11HYX5N","01BJQ0E1C3Z56ABCD0E11HYX6Q","01BJQ0E1C3Z56ABCD0E11HYX7R","01BJQ0E1C3Z56ABCD0E11HYX8P"]"#,
        )
        .dispatch();

        assert_eq!(
            r#"["015cae07-0583-f94c-a5b1-a070431f7516","015cae07-0583-f94c-a5b1-a070431f74f8","015cae07-0583-f94c-a5b1-a070431f74d7","015cae07-0583-f94c-a5b1-a070431f74b5","015cae07-0583-f94c-a5b1-a070431f7494"]"#,
            response.into_string().unwrap()
        );
    }

    #[test]
    fn ulids_analyze_test() {
        let client = test_client_stateful(routes(), Timekeeper::new());
        let response = client
            .post("/12/ulids/5")
            .body(
                json!([
                    "00WEGGF0G0J5HEYXS3D7RWZGV8",
                    "76EP4G39R8JD1N8AQNYDVJBRCF",
                    "018CJ7KMG0051CDCS3B7BFJ3AK",
                    "00Y986KPG0AMGB78RD45E9109K",
                    "010451HTG0NYWMPWCEXG6AJ8F2",
                    "01HH9SJEG0KY16H81S3N1BMXM4",
                    "01HH9SJEG0P9M22Z9VGHH9C8CX",
                    "017F8YY0G0NQA16HHC2QT5JD6X",
                    "03QCPC7P003V1NND3B3QJW72QJ"
                ])
                .to_string(),
            )
            .dispatch();

        assert_eq!(
            r#"{"christmas eve":3,"weekday":1,"in the future":2,"LSB is 1":5}"#,
            response.into_string().unwrap()
        );
    }
}
