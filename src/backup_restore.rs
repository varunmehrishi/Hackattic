use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    time::Duration,
};

use base64::{engine::general_purpose, Engine};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tokio_postgres::NoTls;

use crate::Hackattic;

pub struct BackupRestore;

#[derive(Deserialize, Debug)]
pub struct BackupRestoreProblem {
    dump: String,
}

#[derive(Serialize, Debug)]
pub struct BackupRestoreAnswer {
    alive_ssns: Vec<String>,
}

impl Hackattic for BackupRestore {
    const NAME: &'static str = "backup_restore";
    type Problem = BackupRestoreProblem;
    type Answer = BackupRestoreAnswer;

    // create a new postgres instance
    // docker run --name pg -p 5432:5432 -e POSTGRES_PASSWORD=toor -d postgres:10.21
    async fn solve(problem: Self::Problem) -> anyhow::Result<Self::Answer> {
        let sql_dump = get_uncompressed_sql_dump(&problem.dump)?;
        write_dump_to_database(&sql_dump)?;

        sleep(Duration::from_millis(500)).await;

        let (client, connection) =
            tokio_postgres::connect("host=localhost user=postgres password=toor", NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let res = client
            .query(
                "select ssn from criminal_records where status like 'alive'",
                &[],
            )
            .await?;

        let ssns: Vec<String> = res.iter().map(|r| r.get("ssn")).collect();
        Ok(BackupRestoreAnswer { alive_ssns: ssns })
    }
}

fn get_uncompressed_sql_dump(encoded: &str) -> anyhow::Result<String> {
    let compressed_bytes = general_purpose::STANDARD.decode(encoded)?;
    let mut decoder = GzDecoder::new(compressed_bytes.as_slice());
    let mut s = String::new();
    decoder.read_to_string(&mut s)?;
    Ok(s)
}

fn write_dump_to_database(sql_dump: &str) -> anyhow::Result<()> {
    let mut child = Command::new("psql")
        .arg("-h")
        .arg("localhost")
        .arg("-U")
        .arg("postgres")
        .arg("-f")
        .arg("-") // read file from stdin
        .env("PGPASSWORD", "toor")
        .stdin(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.as_mut().unwrap();
    stdin.write_all(sql_dump.as_bytes())?;

    stdin.flush()?;
    Ok(())
}
