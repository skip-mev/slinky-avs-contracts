use cosmwasm_std::Binary;
use schemars::_serde_json::json;
use slinky_avs_contracts::msg::{GenericVE, SudoMsg};

fn main() {
    let mut data = Vec::<GenericVE>::new();
    data.push(GenericVE{vote: Binary::from(Vec::<u8>::new()), ve_power: 123});
    let sudo_msg = SudoMsg{
        data,
    };
    println!("{:?}", json!(sudo_msg))
}
