use bincode::deserialize;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;
use std::collections::BTreeMap;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct SudoMsg {
    pub data: Vec<GenericVE>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Stake {},
    SubmitRoot {
        chain_id: String,
        root: Binary,
    },
}

#[cw_serde]
pub struct Vote {
    pub roots: BTreeMap<String, Binary>,
}

#[cw_serde]
pub struct VoteExtension {
    pub vote: Vote,
    pub ve_power: u64,
}

#[cw_serde]
pub struct GenericVE {
    pub vote: Binary,
    pub ve_power: u64,
}

impl From<GenericVE> for VoteExtension {
    fn from(value: GenericVE) -> Self {
        let vote = deserialize(value.vote.as_ref()).unwrap();
        VoteExtension {
            vote,
            ve_power: value.ve_power,
        }
    }
}

impl From<Binary> for VoteExtension {
    fn from(value: Binary) -> Self {
        deserialize(value.as_ref()).unwrap()
    }
}
