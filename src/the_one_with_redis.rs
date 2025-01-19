use std::collections::{HashMap, HashSet};

use base64::{engine::general_purpose, Engine as _};
use bstr::BString;
use rdb_parser::Construct;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::Hackattic;

pub struct TheOneWithRedis;

#[derive(Deserialize, Debug)]
pub struct TheOneWithRedisProblem {
    rdb: String,
    requirements: Requirements,
}

#[derive(Deserialize, Debug)]
struct Requirements {
    check_type_of: String,
}

#[derive(Serialize, Debug)]
pub struct TheOneWithRedisAnswer {
    db_count: u32,
    emoji_key_value: String,
    expiry_millis: u64,
    #[serde(flatten)]
    check_type_of: HashMap<String, Value>,
}

impl Hackattic for TheOneWithRedis {
    const NAME: &'static str = "the_redis_one";
    type Problem = TheOneWithRedisProblem;
    type Answer = TheOneWithRedisAnswer;

    async fn solve(problem: Self::Problem) -> anyhow::Result<Self::Answer> {
        let mut bytes = general_purpose::STANDARD.decode(&problem.rdb)?;

        bytes[0] = b'R';
        bytes[1] = b'E';
        bytes[2] = b'D';
        bytes[3] = b'I';
        bytes[4] = b'S';

        let mut input = bytes.as_ref();

        let elements = rdb_parser::parse(&mut input)?;

        let num_dbs = find_num_dbs(&elements);
        let value = find_emoji_key_value(&elements);
        let expiry = find_expiry_millis(&elements);
        let type_of = find_type_of(&elements, BString::from(problem.requirements.check_type_of.clone()));

        let mut mp: HashMap<String, Value> = HashMap::new();

        mp.insert(problem.requirements.check_type_of.to_owned(), Value::String(type_of.unwrap()));

        Ok(Self::Answer {
            db_count: num_dbs as u32,
            emoji_key_value: value.unwrap().to_string(),
            expiry_millis: expiry.unwrap(),
            check_type_of: mp
        })
    }
}

fn find_num_dbs(elems: &[Construct]) -> usize {
    let mut uniq = HashSet::new();

    for e in elems {
        if let Construct::DatabaseSelector(selector) = e {
            uniq.insert(selector);
        }
    }

    uniq.len()
}

fn find_emoji_key_value(elems: &[Construct]) -> Option<BString> {
    for e in elems {
        if let Construct::Entry(key, value, _) = e {
            if !key.is_ascii() {
                if let rdb_parser::Value::String(s) = value {
                    return Some(s.to_owned());
                }
            }
        }
    }

    None
}

fn find_expiry_millis(elems: &[Construct]) -> Option<u64> {
    for e in elems {
        if let Construct::Entry(_, _, Some(duration)) = e {
            return Some(duration.as_millis() as u64);
        }
    }

    None
}

fn find_type_of(elems: &[Construct], match_key: BString) -> Option<String> {
    for e in elems {
        if let Construct::Entry(key, value, _) = e {
            if *key == match_key {
                return Some(match value {
                    rdb_parser::Value::String(_) => String::from("string"),
                    rdb_parser::Value::List(_) => String::from("list"),
                    rdb_parser::Value::Set(_) => String::from("set"),
                    rdb_parser::Value::Hash(_) => String::from("hash"),
                    rdb_parser::Value::SortedSet(_) => String::from("sortedset"),
                });
            }
        }
    }
    None
}
