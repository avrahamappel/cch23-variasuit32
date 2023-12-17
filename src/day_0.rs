use rocket::{get, routes, Route};

use crate::common::Error;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/-1/error")]
fn fake_error() -> Error {
    Error {
        message: "This is an error",
    }
}

#[cfg(test)]
pub fn routes() -> Vec<Route> {
    routes![index, fake_error]
}

#[cfg(test)]
mod tests {
    use rocket::http::Status;

    use super::routes;
    use crate::common::test_client;

    #[test]
    fn index_test() {
        let client = test_client(routes());
        let response = client.get("/").dispatch();
        assert_eq!(Status::Ok, response.status());
    }

    #[test]
    fn fake_error_test() {
        let client = test_client(routes());
        let response = client.get("/-1/error").dispatch();
        assert_eq!(Status::InternalServerError, response.status());
    }
}
