use rocket::{get, routes, Route};
use rustemon::client::RustemonClient;
use rustemon::pokemon::pokemon;

use crate::common::Error;

async fn pokemon_weight_kg(id: i64) -> Result<f64, Error> {
    let client = RustemonClient::default();
    let pkm = pokemon::get_by_id(id, &client).await.map_err(|_| Error {
        message: "Something went wrong",
    })?;

    #[allow(clippy::cast_precision_loss)]
    Ok((pkm.weight as f64) / 10.0)
}

#[get("/8/weight/<id>")]
async fn pokemon_weight(id: i64) -> Result<String, Error> {
    Ok(pokemon_weight_kg(id).await?.to_string())
}

#[get("/8/drop/<id>")]
async fn pokemon_drop(id: i64) -> Result<String, Error> {
    let mass = pokemon_weight_kg(id).await?;
    let height = 10.0;
    let gravitational_acceleration = 9.825;
    // Thanks ChatGPT for these formulas
    let velocity = (2.0f64 * height * gravitational_acceleration).sqrt();
    let momentum = velocity * mass;

    Ok(momentum.to_string())
}

pub fn routes() -> Vec<Route> {
    routes![pokemon_weight, pokemon_drop]
}

#[cfg(test)]
mod tests {
    use crate::common::test_client;

    #[test]
    fn pokemon_weight_test() {
        let client = test_client(super::routes());
        let response = client.get("/8/weight/25").dispatch();

        assert_eq!("6", response.into_string().unwrap());
    }

    #[test]
    fn pokemon_drop_test() {
        let client = test_client(super::routes());
        let response = client.get("/8/drop/25").dispatch();

        assert_eq!("84.10707461325713", response.into_string().unwrap());
    }
}
