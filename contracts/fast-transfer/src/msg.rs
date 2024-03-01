use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    /// Base token denom
    pub base_denom: String,
    /// LP token sub-denom
    pub lp_sub_denom: String,
    /// Aggregator contract address
    pub aggregator_contract: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit(Deposit),
    FastTransfer(FastTransfer),
    Withdraw(Withdraw),
    SlowTransfer {
        id: u64,
        recipient: String,
        amount: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VaultInfoResponse)]
    VaultInfo {},

    #[returns(Uint128)]
    PreviewDeposit { amount: Uint128 },

    #[returns(Uint128)]
    PreviewWithdraw { amount: Uint128 },

    #[returns(Uint128)]
    TotalAssets {},

    #[returns(Uint128)]
    TotalVaultTokenSupply {},
}

#[cw_serde]
pub struct Deposit {
    pub amount: Uint128,
}

#[cw_serde]
pub struct Withdraw {
    pub amount: Uint128,
}

#[cw_serde]
pub struct VaultInfoResponse {
    pub base_token: String,
    pub lp_token: String,
}

#[cw_serde]
pub struct SlowTransfer {
    pub id: u64,
    pub recipient: String,
    pub amount: Uint128,
}

#[cw_serde]
pub struct FastTransfer {
    pub transaction_receipt: Binary,
    pub branch: Vec<[u8; 32]>,
    pub indices: Vec<usize>,
    pub root_hash: [u8; 32],
    pub sender: String,
    pub receiver: String,
    pub denom: String,
    pub amount: u128,
    pub total_leaves: usize,
    pub chain_id: String,
}
