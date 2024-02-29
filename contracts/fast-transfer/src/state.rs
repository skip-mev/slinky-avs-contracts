use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;
use cw_storage_plus::{Item, Map};

/// The base token that is accepted for deposits.
pub const BASE_TOKEN: Item<String> = Item::new("base_token");

/// The denom of the native vault token that represents shares of the vault.
pub const LP_TOKEN_DENOM: Item<String> = Item::new("lp_token_vault");

/// Stores the state of the vault.
pub const STATE: Item<VaultState> = Item::new("state");

pub const PROCESSED_IDS: Map<u64, bool> = Map::new("processed_ids");

#[cw_serde]
/// A struct that represents the state of the vault.
pub struct VaultState {
    /// The total amount of base tokens staked in the vault.
    pub staked_base_tokens: Uint128,
    /// The total amount of vault tokens in circulation.
    pub vault_token_supply: Uint128,
}
