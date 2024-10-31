use std::str::FromStr;

use rocket::{get, routes, Route};

use crate::common::Error;

fn decimal_to_dms<'dir>(
    degrees: f64,
    neg_sign: &'dir str,
    pos_sign: &'dir str,
) -> (u32, u32, f64, &'dir str) {
    let d = degrees.trunc();
    let m = (degrees.fract() * 60.0).trunc();
    let s = 3600.0 * degrees.fract() - m * 60.0;

    let dir = if degrees >= 0.0 { pos_sign } else { neg_sign };

    (d.abs() as u32, m.abs() as u32, s.abs(), dir)
}

struct S2Cell(u64);

impl FromStr for S2Cell {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str_radix(s, 2).map(Self).map_err(Error::from)
    }
}

impl S2Cell {
    fn to_dms_string(&self) -> String {
        let (lat, long) = self.to_lat_long();
        let (latd, latm, lats, latdir) = decimal_to_dms(lat, "E", "W");
        let (lngd, lngm, lngs, lngdir) = decimal_to_dms(long, "N", "S");

        format!("{lngd}°{lngm}'{lngs:.3}''{lngdir} {latd}°{latm}'{lats:.3}''{latdir}")
    }

    fn to_lat_long(&self) -> (f64, f64) {
        let level = self.get_level();

        let mut lat_min = -90.0;
        let mut lat_max = 90.0;
        let mut lng_min = -180.0;
        let mut lng_max = 180.0;

        for i in 0..level {
            let mask: u64 = 1 << (2 * (level - i - 1));
            if self.0 & mask == 0 {
                lat_max = (lat_min + lat_max) / 2.0;
            } else {
                lat_min = (lat_min + lat_max) / 2.0;
            }

            let mask: u64 = 1 << (2 * (level - i - 1) + 1);
            if self.0 & mask == 0 {
                lng_max = (lng_min + lng_max) / 2.0;
            } else {
                lng_min = (lng_min + lng_max) / 2.0;
            }
        }
        // Determine center lat long
        let lat_cntr = (lat_min + lat_max) / 2.0;
        let lng_cntr = (lng_min + lng_max) / 2.0;

        (lat_cntr, lng_cntr)
    }

    fn get_level(&self) -> u32 {
        let mut level = 0;
        let mut id = self.0;
        while id > 0 {
            id >>= 2;
            level += 1;
        }
        level
    }
}

#[get("/coords/<coords>")]
fn coords(coords: &str) -> Result<String, Error> {
    coords.parse::<S2Cell>().map(|cell| cell.to_dms_string())
}

pub fn routes() -> Vec<Route> {
    routes![coords]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::test_client;

    #[test]
    fn test_decimal_to_dms() {
        for (dec, dms) in [
            (37.50715714646503, (37, 30, 25.765727274119854, "pos")),
            (-99.62292075622827, (99, 37, 22.514722421765327, "neg")),
        ] {
            assert_eq!(dms, decimal_to_dms(dec, "neg", "pos"));
        }
    }

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
