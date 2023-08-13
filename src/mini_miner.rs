use super::Hackattic;
use anyhow::{anyhow, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_tuple::{Deserialize_tuple, Serialize_tuple};
use sha2::{
    digest::generic_array::{typenum, GenericArray},
    Digest, Sha256,
};
use tracing::info;
use std::borrow::Cow;

#[derive(Deserialize, Debug)]
pub struct MiniMinerProblem {
    pub difficulty: u32,
    pub block: ProblemBlock,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProblemBlock {
    pub data: Vec<Data>,
    pub nonce: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block<'a> {
    pub data: Cow<'a, Vec<Data>>,
    pub nonce: Option<i32>,
}

impl<'a> Block<'a> {
    pub fn with_nonce(&self, nonce: i32) -> Self {
        Block {
            data: self.data.clone(),
            nonce: Some(nonce),
        }
    }

    pub fn from(block: ProblemBlock) -> Self {
        Block {
            data: Cow::Owned(block.data),
            nonce: block.nonce,
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
        let difficulty = problem.difficulty;

        let block = Block::from(problem.block);

        let found_block = (0..=i32::MAX)
            .into_par_iter()
            .map(|nonce| block.with_nonce(nonce))
            .find_any(|block| is_block_valid(&block, difficulty));

        info!("{found_block:?}");

        if let Some(valid_block) = found_block {
            Ok(MiniMinerAnswer {
                nonce: valid_block.nonce.expect("None nonce"),
            })
        } else {
            Err(anyhow!("No block found"))
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

    check_difficulty(&hash, difficulty)
}

#[cfg(test)]
mod tests {
    use crate::mini_miner::{calculate_sha256, check_difficulty};

    use super::Block;

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
            data: Cow::owned(vec![]),
            nonce: Some(45),
        };

        let s = serde_json::to_string(&b).expect("Could not Serialize");
        let hash = calculate_sha256(s);
        assert!(check_difficulty(&hash, 8))
    }
}
