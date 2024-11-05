use rocket::data::{Data, ToByteUnit};
use rocket::{post, routes, Route};

use crate::common::Error;

#[post("/integers", data = "<int_strs>")]
async fn integers(int_strs: Data<'_>) -> Result<String, Error> {
    let uniq_int = int_strs
        // One of the tests has a very large input
        .open(10.megabytes())
        .into_string()
        .await?
        .split_whitespace()
        .map(str::parse::<usize>)
        .filter_map(Result::ok)
        // XOR all strings to find the unique one
        // This only works because there is exactly one unique string
        .fold(0, |acc, int| acc ^ int);

    Ok("🎁".repeat(uniq_int))
}

pub fn routes() -> Vec<Route> {
    routes![integers]
}

#[cfg(test)]
mod tests {
    use crate::common::test_client;

    #[test]
    fn test_integers() {
        let client = test_client(super::routes());

        let res = client
            .post("/integers/")
            .body(
                "888
77
888
22
77",
            )
            .dispatch();

        assert_eq!(
            "🎁🎁🎁🎁🎁🎁🎁🎁🎁🎁🎁🎁🎁🎁🎁🎁🎁🎁🎁🎁🎁🎁",
            res.into_string().unwrap()
        );
    }
}
