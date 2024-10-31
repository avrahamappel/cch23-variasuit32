use rocket::{get, routes, Route};
use s2::cellid::CellID;
use s2::latlng::LatLng;

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

pub fn routes() -> Vec<Route> {
    routes![coords]
}

#[cfg(test)]
mod tests {
    use crate::common::test_client;

    #[test]
    fn test_coords() {
        let client = test_client(super::routes());

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
