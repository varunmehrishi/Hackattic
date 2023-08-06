use anyhow::{anyhow, Result};
use std::sync::OnceLock;

static INSTANCE: OnceLock<HackatticContext> = OnceLock::new();

pub struct HackatticContext {
    pub access_token: String,
    pub playground: bool,
}

impl HackatticContext {
    pub fn global() -> &'static HackatticContext {
        INSTANCE.get().expect("Context not initialized")
    }

    pub fn init() -> Result<()> {
        let access_token = std::env::var("HA_ACCESS_TOKEN")?;
        let playground = std::env::var("HA_PLAYGROUND")
            .ok()
            .into_iter()
            .flat_map(|s| s.parse::<bool>())
            .take(1)
            .next()
            .unwrap_or(false);

        INSTANCE
            .set(HackatticContext {
                access_token,
                playground,
            })
            .map_err(|_| anyhow!("failed to init context"))?;

        Ok(())
    }
}
