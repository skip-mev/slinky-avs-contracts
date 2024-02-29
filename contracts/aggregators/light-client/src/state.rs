use cosmwasm_std::{Binary};
use cw_storage_plus::{Map};

pub const MERKLE_ROOTS: Map<String, Vec<Binary>> = Map::new("chain_id_to_merkle_root");
