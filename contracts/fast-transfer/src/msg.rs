use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    /// Base token denom
    pub base_denom: String,
    /// LP token sub-denom
    pub lp_sub_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit(Deposit),
    Withdraw(Withdraw),
    SlowTransfer(SlowTransfer),
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
