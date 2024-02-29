use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(LookupHashResponse)]
    LookupHash {
        chain_id: String,
        hash: Binary
    },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct LookupHashResponse {
    pub age: u64,
}

#[cw_serde]
pub struct SudoMsg {
    pub data: Vec<Binary>
}
