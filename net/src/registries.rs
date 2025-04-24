use std::{collections::BTreeMap, sync::OnceLock};

use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Registries {
    #[serde(rename = "minecraft:entity_type")]
    pub entity_types: Registry<EntityType>,
}

#[derive(Debug, Deserialize)]
struct Registry<T> {
    pub entries: BTreeMap<String, T>,
}

#[derive(Debug, Deserialize)]
pub struct EntityType {
    pub protocol_id: i32,
}

#[derive(Debug, Deserialize)]
struct Block {
    pub states: Vec<BlockState>,
}

#[derive(Debug, Deserialize)]
pub struct BlockState {
    pub id: i32,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

fn registries() -> &'static Registries {
    static CACHE: OnceLock<Registries> = OnceLock::new();
    CACHE.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../target/registries/generated/reports/registries.json"
        ))
        .unwrap()
    })
}

fn blocks() -> &'static BTreeMap<String, Block> {
    static CACHE: OnceLock<BTreeMap<String, Block>> = OnceLock::new();
    CACHE.get_or_init(|| {
        serde_json::from_str(include_str!(
            "../../target/registries/generated/reports/blocks.json"
        ))
        .unwrap()
    })
}

pub fn entity_type(id: &str) -> anyhow::Result<&'static EntityType> {
    let registries = registries();
    let entity_type = registries
        .entity_types
        .entries
        .get(id)
        .with_context(|| format!("No such entity type: {id}"))?;
    Ok(entity_type)
}

pub fn block_state(id: &str, properties: &[(&str, &str)]) -> anyhow::Result<&'static BlockState> {
    let blocks = blocks();
    let block = blocks
        .get(id)
        .with_context(|| format!("No such block: {id}"))?;
    let state = block
        .states
        .iter()
        .find(|s| {
            properties
                .iter()
                .all(|&(k, v)| s.properties.get(k).is_some_and(|x| x == v))
        })
        .with_context(|| format!("No such block state for block: {id}"))?;
    Ok(state)
}
