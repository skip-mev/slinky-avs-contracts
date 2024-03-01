use cosmwasm_schema::schema_for;
use schemars::_serde_json::{to_string_pretty};
use slinky_avs_contracts::msg::{SudoMsg};

fn main() {
    let schema = schema_for!(SudoMsg);
    println!("{}", to_string_pretty(&schema).unwrap());
}
