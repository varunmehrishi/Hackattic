use std::collections::HashMap;

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use crate::Hackattic;

pub struct TheOneWithRedis;

#[derive(Deserialize, Debug)]
pub struct TheOneWithRedisProblem {
    rdb: String,
    requirements: Requirements
}

#[derive(Deserialize, Debug)]
struct Requirements {
    check_type_of: String
}

#[derive(Serialize, Debug)]
pub struct TheOneWithRedisAnswer {
    db_count: u32,
    emoji_key_value: String,
    expiry_millis: u64,
    #[serde(flatten)]
    check_type_of: HashMap<String, Value>
}

impl Hackattic for TheOneWithRedis {
    const NAME: &'static str = "the_redis_one";
    type Problem = TheOneWithRedisProblem;
    type Answer = TheOneWithRedisAnswer;

    async fn solve(problem: Self::Problem) -> anyhow::Result<Self::Answer> {
        let bytes = general_purpose::STANDARD.decode(&problem.rdb)?;
        let s = hex::encode(&bytes);
        info!("{:?}", s);
        todo!()
    }
}
