use std::ffi::OsStr;
use std::path::PathBuf;

use rocket::{get, routes, Route};

#[get("/1/<nums..>")]
fn exclusive_cube(nums: PathBuf) -> String {
    nums.iter()
        .filter_map(OsStr::to_str)
        .filter_map(|s| s.parse::<i32>().ok())
        .fold(0, |acc, num| acc ^ num)
        .pow(3)
        .to_string()
}

pub fn routes() -> Vec<Route> {
    routes![exclusive_cube]
}

#[cfg(test)]
mod tests {
    use crate::common::test_client;

    #[test]
    fn exclusive_cube_test() {
        for (expected, url) in [
            ("1728", "/1/4/8"),
            ("1000", "/1/10"),
            ("27", "/1/4/5/8/10"),
            ("-64", "/1/-3/1"),
        ] {
            let client = test_client(super::routes());
            let response = client.get(url).dispatch();
            assert_eq!(expected, response.into_string().unwrap());
        }
    }
}
