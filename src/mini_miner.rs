use super::Hackattic;
use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_tuple::{Deserialize_tuple, Serialize_tuple};
use sha2::{
    digest::generic_array::{typenum, GenericArray},
    Digest, Sha256,
};
use std::sync::Arc;
use tracing::info;

#[derive(Deserialize, Debug)]
pub struct MiniMinerProblem {
    pub difficulty: u32,
    pub block: Block,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub data: Arc<Vec<Data>>,
    pub nonce: Option<i32>,
}

impl Block {
    pub fn with_nonce(&self, nonce: i32) -> Self {
        Block {
            data: Arc::clone(&self.data),
            nonce: Some(nonce),
        }
    }
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct Data {
    pub data: String,
    pub nonce: i32,
}

#[derive(Serialize, Debug)]
#[serde(rename = "")]
pub struct MiniMinerAnswer {
    pub nonce: i32,
}

pub struct MiniMiner;

impl Hackattic for MiniMiner {
    const NAME: &'static str = "mini_miner";
    type Problem = MiniMinerProblem;
    type Answer = MiniMinerAnswer;

    fn solve(problem: Self::Problem) -> Result<Self::Answer> {
        let found_block = (0..=i32::MAX)
            .into_par_iter()
            .map(|nonce| problem.block.with_nonce(nonce))
            .find_any(|block| is_block_valid(block, problem.difficulty));

        info!("{found_block:?}");

        if let Some(valid_block) = found_block {
            Ok(MiniMinerAnswer {
                nonce: valid_block.nonce.context("nonce is None")?,
            })
        } else {
            anyhow::bail!("No block found")
        }
    }
}

fn check_difficulty(hash: &[u8], mut difficulty: u32) -> bool {
    let mut index = 0;
    while difficulty > 0 && index < hash.len() {
        let current_byte = hash[index];
        let mask = get_mask(difficulty);

        if current_byte & mask != 0 {
            return false;
        }

        difficulty = difficulty.saturating_sub(8);
        index += 1;
    }
    difficulty == 0
}

fn get_mask(difficulty: u32) -> u8 {
    match difficulty {
        0 => 0b0000_0000,
        1 => 0b1000_0000,
        2 => 0b1100_0000,
        3 => 0b1110_0000,
        4 => 0b1111_0000,
        5 => 0b1111_1000,
        6 => 0b1111_1100,
        7 => 0b1111_1110,
        _ => 0b1111_1111,
    }
}

fn calculate_sha256(s: String) -> GenericArray<u8, typenum::U32> {
    let mut hasher = Sha256::new();
    hasher.update(s.into_bytes());
    hasher.finalize()
}

fn is_block_valid(block: &Block, difficulty: u32) -> bool {
    let s = serde_json::to_string(block).expect("Unable to serialize");
    let hash = calculate_sha256(s);

    check_difficulty(hash.as_ref(), difficulty)
}

#[cfg(test)]
mod tests {
    use super::Block;
    use super::{calculate_sha256, check_difficulty};
    use std::sync::Arc;

    #[test]
    fn test_check_difficulty() {
        for d in 0..=24 {
            assert!(check_difficulty(&[0, 0, 0, 0xFF], d));
        }
        for d in 25..=32 {
            assert!(!check_difficulty(&[0, 0, 0, 0xFF], d));
        }
        for d in 28..=32 {
            assert!(!check_difficulty(&[0, 0, 0, 0xF0], d));
        }
    }

    #[test]
    fn test_empty_block_with_known_nonce() {
        let b = Block {
            data: Arc::new(vec![]),
            nonce: Some(45),
        };

        let s = serde_json::to_string(&b).expect("Could not Serialize");
        let hash = calculate_sha256(s);
        assert!(check_difficulty(&hash, 8))
    }
}
