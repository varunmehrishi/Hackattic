use std::{env, fs::File, io::BufReader, path::PathBuf, process::Command, thread::sleep, time::Duration};

use anyhow::Context;
use image::{self, GenericImageView};
use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::Hackattic;

pub struct VisualBasicMath;

#[derive(Deserialize, Debug)]
pub struct VisualBasicMathProblem {
    image_url: String,
}

#[derive(Serialize, Debug)]
pub struct VisualBasicMathAnswer {
    result: String,
}

impl Hackattic for VisualBasicMath {
    const NAME: &'static str = "visual_basic_math";
    type Problem = VisualBasicMathProblem;
    type Answer = VisualBasicMathAnswer;

    async fn solve(problem: Self::Problem) -> anyhow::Result<Self::Answer> {
        info!("Received image url {}", problem.image_url);
        let client = ClientBuilder::new().cookie_store(true).build()?;
        let response = client.get(problem.image_url).send().await?;
        let bytes = response.bytes().await?;
        let image = image::load_from_memory(bytes.as_ref())?;
        let image = image.grayscale();
        info!("Dimensions: {:?}", image.dimensions());
        info!("Color: {:?}", image.color());
        
        let mut path = env::current_dir()?;
        path.push("image.png");
        image.save(path)?;
        run_ocr()?;
        sleep(Duration::from_secs(5));

        let data = read_ocr_output()?;

        let mut acc: i128 = 0;
        for line in data.lines {
            let t = line.text;
            let op: Vec<char> = t.chars().take(1).collect();
            let rest: String = t.chars().skip(1)
                .filter(|c| !c.is_whitespace())
                .collect();

            debug!("op: {:?}", op);
            debug!("rest: {:?}", rest);

            let num: i128 = rest.parse()?;

            match op[0] {
                '+' => acc += num,
                '-' => acc -= num,
                'x' => acc *= num,
                '×' => acc *= num,
                '*' => acc *= num,
                '÷' => acc /= num,
                x => return Err(anyhow::bail!(format!("Unexpected character {}", x)))
            }
        }

        Ok(VisualBasicMathAnswer {
            result: acc.to_string()
        })
    }
}

/// Given a file path relative to the crate root, return the absolute path.
fn file_path(path: &str) -> PathBuf {
    let mut abs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    abs_path.push(path);
    abs_path
}

/// Calls Apple Live Text to do the OCR via https://github.com/xulihang/macOCR
/// Tried other OCR libraries such as ocrs, tesseract, easyocr - but all of them failed to
/// recognise the unicode division symbol ÷
fn run_ocr() -> anyhow::Result<()> {
    let mut path = env::current_dir()?;
    path.push("OCR");
    let binary_path = path.to_str().context("OCR path")?;
    let child = Command::new(binary_path)
        .arg("en-us")
        .arg("false")
        .arg("true")
        .arg("image.png")
        .arg("out.json")
        .spawn()?;
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct OcrOutput {
    text: String,
    lines: Vec<LineDetails>,
}

#[derive(Serialize, Deserialize)]
struct LineDetails {
    confidence: f32,
    text: String,
    y: i32,
    x: i32,
    height: i32,
    width: i32
}

fn read_ocr_output() -> anyhow::Result<OcrOutput> {
    let mut path = env::current_dir()?;
    path.push("out.json");

    let file = File::open(path)?;
    let data = std::io::read_to_string(BufReader::new(file))?;

    let data: OcrOutput = serde_json::from_str(&data)?;

    Ok(data)
}
