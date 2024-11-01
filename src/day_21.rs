use std::time::Duration;

use rocket::{get, routes, Route};
use s2::cellid::CellID;
use s2::latlng::LatLng;
use serde::Deserialize;

use crate::common::Error;

fn decimal_to_dms<'dir>(degrees: f64, neg_sign: &'dir str, pos_sign: &'dir str) -> String {
    let d = degrees.trunc();
    let m = (degrees.fract() * 60.0).trunc();
    let s = 3600.0 * degrees.fract() - m * 60.0;

    let dir = if degrees >= 0.0 { pos_sign } else { neg_sign };

    format!("{:.0}°{:.0}'{:.3}''{}", d.abs(), m.abs(), s.abs(), dir)
}

#[get("/coords/<coords>")]
fn coords(coords: &str) -> Result<String, Error> {
    let cellid = u64::from_str_radix(coords, 2).map_err(Error::from)?;
    let latlng: LatLng = CellID(cellid).into();
    let lat_str = decimal_to_dms(latlng.lat.deg(), "S", "N");
    let lng_str = decimal_to_dms(latlng.lng.deg(), "W", "E");

    Ok(format!("{lat_str} {lng_str}"))
}

pub struct GeocodeApiKey {
    pub key: String,
}

#[derive(Deserialize)]
pub struct GeocodeResponse {
    pub address: Address,
}

#[derive(Deserialize)]
pub struct Address {
    pub country: String,
}

#[get("/country/<coords>")]
async fn country(coords: &str, api_key: &rocket::State<GeocodeApiKey>) -> Result<String, Error> {
    let cellid = u64::from_str_radix(coords, 2).map_err(Error::from)?;
    let latlng: LatLng = CellID(cellid).into();

    let url = format!(
        "https://geocode.maps.co/reverse?lat={}&lon={}&api_key={}",
        latlng.lat.deg(),
        latlng.lng.deg(),
        api_key.key
    );

    // API key only allows one request per second
    std::thread::sleep(Duration::from_secs(1));

    let res: GeocodeResponse = reqwest::get(url).await?.json().await?;

    Ok(res.address.country)
}

pub fn routes() -> Vec<Route> {
    routes![coords, country]
}

#[cfg(test)]
mod tests {
    use crate::common::test_client_stateful;

    #[test]
    fn test_coords() {
        let client = test_client_stateful(
            super::routes(),
            super::GeocodeApiKey {
                key: "apikeyaikeyapikey".into(),
            },
        );

        for (coords, expected) in [
            (
                "0100111110010011000110011001010101011111000010100011110001011011",
                "83°39'54.324''N 30°37'40.584''W",
            ),
            (
                "0010000111110000011111100000111010111100000100111101111011000101",
                "18°54'55.944''S 47°31'17.976''E",
            ),
        ] {
            let res = client.get(format!("/coords/{coords}")).dispatch();
            assert_eq!(expected, res.into_string().unwrap());
        }
    }
}
