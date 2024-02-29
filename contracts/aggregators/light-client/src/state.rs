use cosmwasm_std::{Binary};
use cw_storage_plus::{Map};
use serde::{Deserialize, Serialize};

pub const MERKLE_ROOTS: Map<String, ChainHashes> = Map::new("chain_hashes_map");

#[derive(Serialize, Deserialize)]
pub struct ChainHashes {
    pub chain_id: String,
    pub hashes: Vec<Binary>,
    pub max_size: usize,
}
