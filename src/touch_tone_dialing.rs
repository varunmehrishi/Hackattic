use serde::{Deserialize, Serialize};

use crate::Hackattic;

pub struct TouchToneDialing;

#[derive(Deserialize, Debug)]
pub struct TouchToneDialingProblem {
    wav_url: String,
}

#[derive(Serialize, Debug)]
pub struct TouchToneDialingAnswer {
    secret: String,
}

impl Hackattic for TouchToneDialing {
    const NAME: &'static str = "touch_tone_dialing";
    type Problem = TouchToneDialingProblem;
    type Answer = TouchToneDialingAnswer;

    async fn solve(problem: Self::Problem) -> anyhow::Result<Self::Answer> {
        todo!()
    }
}
