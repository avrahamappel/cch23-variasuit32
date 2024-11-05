use std::collections::VecDeque;
use std::num::ParseIntError;
use std::str::FromStr;

use rocket::data::{Data, ToByteUnit};
use rocket::{post, routes, Responder, Route};

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

#[derive(Responder, Debug)]
#[response(status = 400)]
pub struct InputError {
    pub message: &'static str,
}

impl From<Error> for InputError {
    fn from(error: Error) -> Self {
        Self {
            message: error.message,
        }
    }
}

impl From<ParseIntError> for InputError {
    fn from(error: ParseIntError) -> Self {
        Error::from(error).into()
    }
}

#[derive(Clone, Copy)]
struct StarCoords {
    x: i32,
    y: i32,
    z: i32,
}

impl FromStr for StarCoords {
    type Err = InputError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || InputError {
            message: "Star coordinates string did not contain enough entries",
        };
        let mut s_iter = s.split_whitespace();
        let x = s_iter.next().ok_or_else(err)?.parse()?;
        let y = s_iter.next().ok_or_else(err)?.parse()?;
        let z = s_iter.next().ok_or_else(err)?.parse()?;

        Ok(Self { x, y, z })
    }
}

impl StarCoords {
    /// Distance between two stars
    fn distance_from(&self, other: &Self) -> f32 {
        fn square<T: std::ops::Mul<Output = T> + Copy>(x: T) -> T {
            x * x
        }

        #[allow(clippy::cast_precision_loss)]
        ((square(other.x - self.x) + square(other.y - self.y) + square(other.z - self.z)) as f32)
            .sqrt()
    }
}

#[derive(Clone, Copy)]
struct Portal {
    start: usize,
    end: usize,
}

impl FromStr for Portal {
    type Err = InputError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || InputError {
            message: "Portal string did not contain enough indices",
        };
        let mut s_iter = s.split_whitespace();
        let start = s_iter.next().ok_or_else(err)?.parse()?;
        let end = s_iter.next().ok_or_else(err)?.parse()?;

        Ok(Self { start, end })
    }
}

struct Galaxy {
    stars: Vec<StarCoords>,
    portals: Vec<Portal>,
}

impl FromStr for Galaxy {
    type Err = InputError;
    #[allow(clippy::len_zero)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines = s.lines().collect::<Vec<_>>();

        // Parse stars length
        if lines.len() == 0 {
            return Err(InputError {
                message: "Stars length parameter not found",
            });
        }
        let stars_len = lines[0].parse::<usize>()?;
        let lines = &lines[1..];

        // Parse stars
        if lines.len() < stars_len {
            return Err(InputError {
                message: "Star coordinates not found",
            });
        }
        let stars = lines[..stars_len]
            .iter()
            .map(|s| s.parse())
            .filter_map(Result::ok)
            .collect();
        let lines = &lines[stars_len..];

        // Parse portals length
        if lines.len() == 0 {
            return Err(InputError {
                message: "Portals length parameter not found",
            });
        }
        let portals_len = lines[0].parse::<usize>()?;
        let lines = &lines[1..];

        // Parse portals
        if lines.len() < portals_len {
            return Err(InputError {
                message: "Portals list not found",
            });
        }
        let portals = lines[..portals_len]
            .iter()
            .map(|s| s.parse())
            .filter_map(Result::ok)
            .collect();

        Ok(Self { stars, portals })
    }
}

impl Galaxy {
    /// Get the shortest path from star 0 to star N-1 using portals
    fn get_portal_path(&self) -> Vec<Portal> {
        let mut queue : VecDeque<_> =
        // push all portals where start == 0 into queue
         self.portals.iter().copied().filter(|p|p.start == 0).map(|p| vec![p]).collect();

        // loop
        while let Some(path) = queue.pop_front() {
            if path.is_empty() {
                continue;
            }
            let last_segment = path.last().unwrap();
            if last_segment.end == self.stars.len() - 1 {
                return path;
            }
            // for each portal where start == path.last.end
            // add path back into queue
            queue.extend(
                self.portals
                    .iter()
                    .copied()
                    .filter(|p| p.start == last_segment.end)
                    .map(|p| {
                        let mut new_path = path.clone();
                        new_path.push(p);
                        new_path
                    }),
            );
        }

        vec![]
    }

    /// Determine the physical distance traveled over the given path
    fn get_total_distance(&self, path: Vec<Portal>) -> f32 {
        if path.is_empty() {
            return 0.0;
        }

        path.into_iter().fold(0.0, |acc, seg| {
            let star1 = self.stars[seg.start];
            let star2 = self.stars[seg.end];
            let distance = star1.distance_from(&star2);

            acc + distance
        })
    }

    /// Get the summary as expected by the user (path length and total distance) as a string
    fn get_summary(&self) -> String {
        let path = self.get_portal_path();

        format!("{} {:.3}", path.len(), self.get_total_distance(path))
    }
}

#[post("/rocket", data = "<galaxy_chart>")]
fn galaxy(galaxy_chart: &str) -> Result<String, InputError> {
    let g = galaxy_chart.parse::<Galaxy>()?;
    Ok(g.get_summary())
}

pub fn routes() -> Vec<Route> {
    routes![integers, galaxy]
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

    #[test]
    fn test_galaxy() {
        let galaxy = "5
0 1 0
-2 2 3
3 -3 -5
1 1 5
4 3 5
4
0 1
2 4
3 4
1 2
"
        .parse::<super::Galaxy>()
        .unwrap();

        assert_eq!("3 26.123", galaxy.get_summary());
    }
}
