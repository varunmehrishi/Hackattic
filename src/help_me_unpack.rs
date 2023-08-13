use super::Hackattic;
use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct HelpMeUnpackProblem {
    bytes: String,
}

#[derive(Serialize, Debug)]
#[serde(rename = "")]
pub struct HelpMeUnpackAnswer {
    int: i32,
    uint: u32,
    short: i16,
    float: f32,
    double: f64,
    big_endian_double: f64,
}

pub struct HelpMeUnpack;

fn take<const N: usize>(it: &mut impl Iterator<Item = u8>) -> Result<[u8; N]> {
    let mut v = [0u8; N];
    for elem in v.iter_mut() {
        *elem = it.next().context("No Item in iterator")?;
    }
    Ok(v)
}

impl Hackattic for HelpMeUnpack {
    const NAME: &'static str = "help_me_unpack";
    type Problem = HelpMeUnpackProblem;
    type Answer = HelpMeUnpackAnswer;

    fn solve(problem: Self::Problem) -> Result<Self::Answer> {
        let eng = general_purpose::STANDARD.decode(problem.bytes)?;

        let mut iter = eng.iter().copied();
        let int = i32::from_le_bytes(take::<4>(&mut iter)?);
        let uint = u32::from_le_bytes(take::<4>(&mut iter)?);
        let short = i16::from_le_bytes(take::<2>(&mut iter)?);
        let _ = take::<2>(&mut iter);
        let float = f32::from_le_bytes(take::<4>(&mut iter)?);
        let double = f64::from_le_bytes(take::<8>(&mut iter)?);
        let big_endian_double = f64::from_be_bytes(take::<8>(&mut iter)?);

        Ok(HelpMeUnpackAnswer {
            int,
            uint,
            short,
            float,
            double,
            big_endian_double,
        })
    }
}
