use std::env;
use std::ops::Add;
use std::path::{Path, PathBuf};

use image::io::Reader;
use image::DynamicImage;
use rocket::form::Form;
use rocket::fs::{relative, NamedFile, TempFile};
use rocket::{get, post, routes, FromForm, Route};

use crate::common::Error;

#[get("/11/assets/<path..>")]
async fn assets(path: PathBuf) -> Option<NamedFile> {
    let path = Path::new(relative!("assets")).join(path);

    NamedFile::open(path).await.ok()
}

#[derive(FromForm)]
struct Image<'f> {
    image: TempFile<'f>,
}

#[post("/11/red_pixels", data = "<image>")]
async fn count_red_pixels(mut image: Form<Image<'_>>) -> Result<String, Error> {
    let name = image.image.name().unwrap_or("some-image.png");
    let path = env::temp_dir().join(name);
    image.image.persist_to(path).await?;

    let img = Reader::open(image.image.path().ok_or(Error {
        message: "Temp file had no path",
    })?)?
    .with_guessed_format()?
    .decode()?;

    macro_rules! count_pixels {
        ($rgb_image:ident) => {
            count_pixels!($rgb_image, saturating_add)
        };
        ($rgb_image:ident, $add:ident) => {{
            let red_pxl_count = $rgb_image
                .pixels()
                .filter(|p| {
                    let [red, green, blue] = p.0;
                    red > green.$add(blue)
                })
                .count();

            Ok(red_pxl_count.to_string())
        }};
    }

    match img {
        DynamicImage::ImageRgb8(rgb_image) => count_pixels!(rgb_image),
        DynamicImage::ImageRgb16(rgb_image) => count_pixels!(rgb_image),
        DynamicImage::ImageRgb32F(rgb_image) => count_pixels!(rgb_image, add),

        _ => Err(Error {
            message: "Image was not RGB",
        }),
    }
}

pub fn routes() -> Vec<Route> {
    routes![assets, count_red_pixels,]
}
