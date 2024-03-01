use cosmwasm_schema::write_api;

use aggregator::aggregator::QueryMsg;
use slinky_avs_contracts::msg::InstantiateMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        query: QueryMsg,
    }
}
