use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
// use rs_merkle::MerkleProof;
// use crate::merkle::Keccak256Algorithm;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit(Deposit),
    FastTransfer {
        leaf: Binary,
        branch: Vec<[u8; 32]>,
        root_hash: [u8; 32],
        sender: String,
        amount: u128,
    },
    SlowTransfer {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

#[cw_serde]
pub struct Deposit {
    pub confirmations: u8,
}
