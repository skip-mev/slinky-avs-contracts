use cosmwasm_schema::write_api;

use slinky_avs_contracts::msg::{InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
    }
}
