use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::{post, routes, Route};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct ElfCount {
    elf: usize,
    #[serde(rename = "elf on a shelf")]
    on_shelf: usize,
    #[serde(rename = "shelf with no elf on it")]
    shelf_no_elf: usize,
}

impl From<String> for ElfCount {
    fn from(elfstring: String) -> Self {
        let elf = elfstring.matches("elf").count();
        let prefix = "elf on a ";
        let on_shelf = elfstring
            .match_indices("shelf")
            .filter(|(i, _)| &elfstring[(i - prefix.len())..*i] == prefix)
            .count();
        let shelf_no_elf = elfstring
            .match_indices("shelf")
            .filter(|(i, _)| &elfstring[(i - prefix.len())..*i] != prefix)
            .count();

        Self {
            elf,
            on_shelf,
            shelf_no_elf,
        }
    }
}

#[post("/6", data = "<elfstring>")]
fn elf_count(elfstring: String) -> Json<ElfCount> {
    Json(ElfCount::from(elfstring))
}

pub fn routes() -> Vec<Route> {
    routes![elf_count,]
}

#[cfg(test)]
mod tests {
    use crate::common::test_client;

    #[test]
    fn elf_count_test() {
        let client = test_client(super::routes());

        for (expected, data) in [
        (r#"{"elf":4,"elf on a shelf":0,"shelf with no elf on it":1}"#, "The mischievous elf peeked out from behind the toy workshop, and another elf joined in the festive dance. Look, there is also an elf on that shelf!"),
        (r#"{"elf":5,"elf on a shelf":1,"shelf with no elf on it":1}"#, "there is an elf on a shelf on an elf. there is also another shelf in Belfast."),
        (r#"{"elf":4,"elf on a shelf":2,"shelf with no elf on it":0}"#, "In Belfast I heard an elf on a shelf on a shelf on a ")
    ] {
        let response = client.post("/6").body(data).dispatch();

        assert_eq!(expected, response.into_string().unwrap());
    }
    }
}
