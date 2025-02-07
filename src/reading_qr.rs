use anyhow::anyhow;
use image::{self, GenericImageView};
use reqwest::ClientBuilder;
use rqrr::PreparedImage;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::Hackattic;

pub struct ReadingQR;

#[derive(Deserialize, Debug)]
pub struct ReadingQRProblem {
    image_url: String,
}

#[derive(Serialize, Debug)]
pub struct ReadingQRAnswer {
    code: String,
}

impl Hackattic for ReadingQR {
    const NAME: &'static str = "reading_qr";
    type Problem = ReadingQRProblem;
    type Answer = ReadingQRAnswer;

    async fn solve(problem: Self::Problem) -> anyhow::Result<Self::Answer> {
        info!("Received image url {}", problem.image_url);
        let client = ClientBuilder::new().cookie_store(true).build()?;
        let response = client.get(problem.image_url).send().await?;
        let bytes = response.bytes().await?;
        let image = image::load_from_memory(bytes.as_ref())?;
        info!("Dimensions: {:?}", image.dimensions());
        info!("Color: {:?}", image.color());

        let mut image = PreparedImage::prepare(image.to_luma8());
        for grid in image.detect_grids() {
            if let Ok((metadata, data)) = grid.decode() {
                info!("Metadata: {:?}", metadata);
                info!("Data: {}", data);
                return Ok(ReadingQRAnswer { code: data });
            }
        }

        Err(anyhow!("Did not detect any qr in the image"))
    }
}
