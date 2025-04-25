use std::{collections::BTreeMap, sync::OnceLock};

use serde::Deserialize;
use snafu::prelude::*;

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

#[derive(Debug, Snafu)]
#[snafu(display("Entity type not found: {id}"))]
pub struct EntityTypeNotFound {
    id: String,
}

#[derive(Debug, Snafu)]
pub enum BlockStateNotFound {
    #[snafu(display("Block not found: {id}"))]
    BlockNotFound { id: String },
    #[snafu(display("Block state not found: {id} {properties:?}"))]
    BlockStateNotFound {
        id: String,
        properties: Vec<(&'static str, &'static str)>,
    },
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

pub fn entity_type(id: &str) -> Result<&'static EntityType, EntityTypeNotFound> {
    let registries = registries();
    let entity_type = registries
        .entity_types
        .entries
        .get(id)
        .context(EntityTypeNotFoundSnafu { id })?;
    Ok(entity_type)
}

pub fn block_state(
    id: &str,
    properties: &[(&'static str, &'static str)],
) -> Result<&'static BlockState, BlockStateNotFound> {
    let blocks = blocks();
    let block = blocks.get(id).context(BlockNotFoundSnafu { id })?;
    let state = block
        .states
        .iter()
        .find(|s| {
            properties
                .iter()
                .all(|&(k, v)| s.properties.get(k).is_some_and(|x| x == v))
        })
        .context(BlockStateNotFoundSnafu { id, properties })?;
    Ok(state)
}
