use std::{collections::BTreeMap, sync::OnceLock};

use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Block {
    pub states: Vec<BlockState>,
}

#[derive(Debug, Deserialize)]
pub struct BlockState {
    pub id: i32,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

pub fn blocks() -> &'static BTreeMap<String, Block> {
    static CACHE: OnceLock<BTreeMap<String, Block>> = OnceLock::new();
    CACHE.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../target/registries/generated/reports/blocks.json"
        ))
        .unwrap()
    })
}

pub fn block_state(id: &str, properties: &[(&str, &str)]) -> anyhow::Result<&'static BlockState> {
    let blocks = blocks();
    let block = blocks.get(id).context("No such block")?;
    let state = block
        .states
        .iter()
        .find(|s| {
            properties
                .iter()
                .all(|&(k, v)| s.properties.get(k).is_some_and(|x| x == v))
        })
        .context("No such block state")?;
    Ok(state)
}
