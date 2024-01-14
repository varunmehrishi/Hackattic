use std::{collections::HashMap, env, fmt::Debug};

use anyhow::{Context, Result};
use reqwest::{Client, ClientBuilder};
use serde::{de::DeserializeOwned, Serialize};
use tracing::*;

mod help_me_unpack;
use help_me_unpack::HelpMeUnpack;
mod mini_miner;
use mini_miner::MiniMiner;
mod hackattic_context;
use hackattic_context::HackatticContext;
mod password_hashing;
use password_hashing::PasswordHashing;
mod tales_of_ssl;
use tales_of_ssl::TalesOfSsl;
mod backup_restore;
use backup_restore::BackupRestore;

trait Hackattic {
    const NAME: &'static str;
    type Problem: DeserializeOwned + Debug;
    type Answer: Serialize + Debug;

    async fn solve(problem: Self::Problem) -> Result<Self::Answer>;
    fn problem_url() -> String {
        format!("https://hackattic.com/challenges/{}/problem/", Self::NAME)
    }
    fn solve_url() -> String {
        format!("https://hackattic.com/challenges/{}/solve/", Self::NAME)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    HackatticContext::init()?;
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .context("setting default tracing subscriber failed")?;

    let client = ClientBuilder::new().cookie_store(true).build()?;

    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        anyhow::bail!("Challenge name not provided")
    }

    let response = match args[1].as_str() {
        HelpMeUnpack::NAME => solve::<HelpMeUnpack>(client).await?,
        MiniMiner::NAME => solve::<MiniMiner>(client).await?,
        PasswordHashing::NAME => solve::<PasswordHashing>(client).await?,
        TalesOfSsl::NAME => solve::<TalesOfSsl>(client).await?,
        BackupRestore::NAME => solve::<BackupRestore>(client).await?,
        _ => anyhow::bail!("No such challenge found"),
    };

    info!("{}", response);

    Ok(())
}

async fn solve<T: Hackattic>(client: Client) -> Result<String> {
    let context = HackatticContext::global();
    let mut map = HashMap::new();
    map.insert("access_token", context.access_token.as_str());
    if context.playground {
        map.insert("playground", "1");
    }

    debug!("{}", T::problem_url());

    let resp = client.get(T::problem_url()).query(&map).send().await?;

    debug!("{:?}", resp);

    let body = resp.text().await?;

    debug!("{:?}", body);

    // let problem = resp.json().await?;
    let problem = serde_json::from_str(&body)?;

    info!("{:?}", problem);

    let ans = T::solve(problem).await?;

    let string = serde_json::to_string(&ans).context("Unable to serialize")?;

    info!("{}", string);

    let resp = client
        .post(T::solve_url())
        .query(&map)
        .json(&ans)
        .send()
        .await?
        .text()
        .await?;

    Ok(resp)
}
