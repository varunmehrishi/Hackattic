use openssl::{
    asn1::{Asn1Integer, Asn1Time},
    bn::BigNum,
    hash::MessageDigest,
    pkey::PKey,
    x509::{X509Builder, X509Name, X509NameBuilder},
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::Hackattic;

#[derive(Deserialize, Debug)]
pub struct SslProblem {
    private_key: String,
    required_data: RequiredData,
}

#[derive(Deserialize, Debug)]
pub struct RequiredData {
    country: String,
    domain: String,
    serial_number: String,
}

#[derive(Serialize, Debug)]
pub struct SslAnswer {
    certificate: String,
}

const BEGIN_RSA_PRIVATE_KEY: &str = "-----BEGIN RSA PRIVATE KEY-----";
const END_RSA_PRIVATE_KEY: &str = "-----END RSA PRIVATE KEY-----";

pub struct TalesOfSsl;

impl Hackattic for TalesOfSsl {
    const NAME: &'static str = "tales_of_ssl";

    type Problem = SslProblem;

    type Answer = SslAnswer;

    fn solve(problem: Self::Problem) -> anyhow::Result<Self::Answer> {
        debug!("{:?}", problem);
        let mut builder = X509Builder::new()?;

        let serial_number = BigNum::from_hex_str(
            &problem
                .required_data
                .serial_number
                .chars()
                .skip(2)
                .collect::<String>(),
        )?;

        let serial_number = Asn1Integer::from_bn(&serial_number)?;
        builder.set_serial_number(&serial_number)?;

        let subject_name = get_cert_subject_name(&problem.required_data)?;
        builder.set_subject_name(&subject_name)?;

        builder.set_version(1)?;

        let pkey_pem = get_rsa_private_key_pem(&problem.private_key);
        let key = PKey::private_key_from_pem(&pkey_pem)?;

        let public_key = PKey::public_key_from_der(&key.public_key_to_der()?)?;
        builder.set_pubkey(&public_key)?;

        let start_time = Asn1Time::days_from_now(0)?;
        let end_time = Asn1Time::days_from_now(365)?;
        builder.set_not_before(&start_time)?;
        builder.set_not_after(&end_time)?;

        builder.sign(&key, MessageDigest::sha256())?;
        let cert = builder.build();

        let der = cert.to_der()?;

        Ok(SslAnswer {
            certificate: openssl::base64::encode_block(&der),
        })
    }
}

fn get_rsa_private_key_pem(pkey: &str) -> Vec<u8> {
    let mut v = Vec::with_capacity(
        BEGIN_RSA_PRIVATE_KEY.len() + 1 + pkey.len() + 1 + END_RSA_PRIVATE_KEY.len(),
    );
    v.extend_from_slice(BEGIN_RSA_PRIVATE_KEY.as_bytes());
    v.push(b'\n');
    v.extend_from_slice(pkey.as_bytes());
    v.push(b'\n');
    v.extend_from_slice(END_RSA_PRIVATE_KEY.as_bytes());
    v
}

fn get_cert_subject_name(data: &RequiredData) -> anyhow::Result<X509Name> {
    let mut x509_name = X509NameBuilder::new().unwrap();
    x509_name.append_entry_by_text("C", &get_country_code(&data.country))?;
    x509_name.append_entry_by_text("CN", &data.domain)?;
    Ok(x509_name.build())
}

fn get_country_code(country_name: &str) -> String {
    country_name
        .split_whitespace()
        .map(|s| s.as_bytes()[0] as char)
        .collect()
}
