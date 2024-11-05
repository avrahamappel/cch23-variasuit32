use rocket::{post, routes, Route};

#[post("/integers", data = "<int_strs>")]
fn integers(int_strs: &str) -> String {
    // XOR all strings to find the unique one
    // This only works because there is exactly one unique string
    let uniq_int = int_strs
        .split_whitespace()
        .map(str::parse::<usize>)
        .filter_map(Result::ok)
        .fold(0, |acc, int| acc ^ int);

    "🎁".repeat(uniq_int)
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
