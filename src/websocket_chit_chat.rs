use std::time::Instant;

use anyhow::Context;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use websocket::{ClientBuilder, OwnedMessage};

use crate::Hackattic;

pub struct WebsocketChitChat;

#[derive(Deserialize, Debug)]
pub struct WebsocketChitChatProblem {
    token: String,
}

#[derive(Serialize, Debug)]
pub struct WebsocketChitChatAnswer {
    secret: String,
}

impl Hackattic for WebsocketChitChat {
    const NAME: &'static str = "websocket_chit_chat";
    type Problem = WebsocketChitChatProblem;
    type Answer = WebsocketChitChatAnswer;

    async fn solve(problem: Self::Problem) -> anyhow::Result<Self::Answer> {
        let token = problem.token;

        let url = format!("wss://hackattic.com/_/ws/{}", token);
        debug!("URL: {url}");

        const DELAY_ADJUSTMENT: u128 = 250;
        let re = Regex::new("^.*\"(.*)\".*$")?;
        let durs = [700, 1500, 2000, 2500, 3000];

        let mut client = ClientBuilder::new(&url)?.connect(None)?;
        let mut start = Instant::now();
        let mut secret = None;

        loop {
            let mess = client.recv_message()?;

            match mess {
                websocket::OwnedMessage::Text(s) => match s.as_ref() {
                    "ping!" => {
                        info!("Received ping!");
                        let elapsed = start.elapsed().as_millis();
                        start = Instant::now();
                        for &d in durs.iter().rev() {
                            if d < elapsed + DELAY_ADJUSTMENT {
                                info!("Elapsed: {elapsed}, d: {d}");
                                client.send_message(&OwnedMessage::Text(format!("{d}")))?;
                                break;
                            }
                        }
                    }
                    "good!" => {
                        info!("Received good!");
                    }
                    s if s.starts_with("congratulations!") => {
                        let caps = re.captures(s).context("No capture found")?;
                        let str = caps.get(1).unwrap().as_str();
                        secret = Some(str.to_owned());
                    }
                    other => {
                        info!("{other}")
                    }
                },
                websocket::OwnedMessage::Ping(_) => info!("Received Ping"),
                websocket::OwnedMessage::Close(_) => {
                    info!("Received Close");
                    break;
                }
                _ => {}
            }
        }

        client.shutdown()?;

        if let Some(secret) = secret {
            Ok(Self::Answer { secret })
        } else {
            unreachable!("Code should not reach here");
        }
    }
}
