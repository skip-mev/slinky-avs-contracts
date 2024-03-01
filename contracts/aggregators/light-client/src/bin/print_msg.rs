use cosmwasm_schema::schema_for;
use schemars::_serde_json::to_string_pretty;
use slinky_avs_contracts::msg::Vote;

fn main() {
    // let schema = schema_for!(SudoMsg);
    let schema = schema_for!(Vote);
    println!("{}", to_string_pretty(&schema).unwrap());
}
