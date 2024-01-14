use super::Hackattic;
use anyhow::Result;
use base16::encode_lower;
use base64::{engine::general_purpose, Engine};
use hmac::{
    digest::{generic_array::GenericArray, typenum},
    Hmac, Mac,
};
use scrypt::{scrypt, Params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::debug;

type HmacSha256 = Hmac<Sha256>;
type U8_32 = GenericArray<u8, typenum::U32>;

#[derive(Deserialize, Debug)]
pub struct PasswordHashingProblem {
    pub password: String,
    pub salt: String,
    pub pbkdf2: PBKDF2,
    pub scrypt: ScryptParameters,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PBKDF2 {
    pub hash: String,
    pub rounds: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScryptParameters {
    #[serde(rename = "N")]
    pub n: u32,
    #[serde(rename = "p")]
    pub parallization: u32,
    #[serde(rename = "r")]
    pub block_size: u32,
    pub buflen: usize,
    #[serde(rename = "_control")]
    pub control: String,
}

#[derive(Serialize, Debug)]
#[serde(rename = "")]
pub struct PasswordHashingAnswer {
    pub sha256: String,
    pub hmac: String,
    pub pbkdf2: String,
    pub scrypt: String,
}

pub struct PasswordHashing;

impl Hackattic for PasswordHashing {
    const NAME: &'static str = "password_hashing";
    type Problem = PasswordHashingProblem;
    type Answer = PasswordHashingAnswer;

    async fn solve(problem: Self::Problem) -> Result<Self::Answer> {
        debug!("{:?}", problem);
        let sha256 = encode_lower(&calculate_sha256(&problem.password));
        debug!(sha256);

        let key = general_purpose::STANDARD.decode(&problem.salt)?;
        let hmac = encode_lower(&calculate_hmac(problem.password.as_bytes(), &key));

        debug!(hmac);

        let pbkdf2 = encode_lower(&compute_pbkdf2(
            &problem.password,
            &key,
            problem.pbkdf2.rounds,
        ));
        debug!(pbkdf2);

        let scrypt = encode_lower(&(calculate_scrypt(&problem.password, &key, problem.scrypt)?));
        debug!(scrypt);

        Ok(PasswordHashingAnswer {
            sha256,
            hmac,
            pbkdf2,
            scrypt,
        })
    }
}

fn calculate_sha256(s: &str) -> U8_32 {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hasher.finalize()
}

fn calculate_hmac(s: &[u8], k: &[u8]) -> U8_32 {
    let mut mac = HmacSha256::new_from_slice(k).expect("Unable to create Hmaccer");
    mac.update(s);
    mac.finalize().into_bytes()
}

fn compute_pbkdf2(s: &str, salt: &[u8], rounds: u32) -> U8_32 {
    let mut key = salt.to_owned();
    key.append(&mut vec![0u8, 0u8, 0u8, 1u8]);

    let mut u_prev = calculate_hmac(&key, s.as_bytes());
    let mut dk = u_prev;

    for _ in 1..rounds {
        let u_cur = calculate_hmac(&u_prev, s.as_bytes());
        xor_update(&mut dk, u_cur);
        u_prev = u_cur;
    }

    dk
}

fn xor_update(a: &mut U8_32, b: U8_32) {
    for i in 0..32 {
        a[i] ^= b[i];
    }
}

fn calculate_scrypt(s: &str, salt: &[u8], parameters: ScryptParameters) -> Result<Vec<u8>> {
    let n_lg = f32::log2(parameters.n as f32) as u8;
    let params = Params::new(
        n_lg,
        parameters.block_size,
        parameters.parallization,
        parameters.buflen,
    )?;

    let mut output = vec![0u8; parameters.buflen];
    scrypt(s.as_bytes(), salt, &params, &mut output)?;

    Ok(output)
}

#[cfg(test)]
mod tests {}
