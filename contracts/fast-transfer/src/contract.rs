use cosmwasm_std::{
    entry_point, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult,
};
use cw2::set_contract_version;
use osmosis_std::types::osmosis::tokenfactory::v1beta1::MsgCreateDenom;

use crate::error::ContractError;
use crate::execute::{execute_deposit, execute_withdraw};
use crate::helpers::{convert_to_assets, convert_to_shares};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query::query_vault_info;
use crate::state::{BASE_TOKEN, LP_TOKEN_DENOM, STATE};

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// instantiate is used to construct the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Store base token and vault token denom
    let lp_token_denom = format!("factory/{}/{}", env.contract.address, msg.lp_sub_denom);

    BASE_TOKEN.save(deps.storage, &msg.base_denom)?;
    LP_TOKEN_DENOM.save(deps.storage, &lp_token_denom)?;

    // Create the LP token denom
    let created_denom_msg: CosmosMsg = MsgCreateDenom {
        sender: env.contract.address.to_string(),
        subdenom: msg.lp_sub_denom,
    }
    .into();

    Ok(Response::new().add_message(created_denom_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Deposit(deposit) => {
            execute_deposit(deps, env, info.clone(), deposit.amount, info.sender)
        }
        ExecuteMsg::Withdraw(withdraw) => {
            execute_withdraw(deps, env, info.clone(), withdraw.amount, info.sender)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::VaultInfo {} => to_json_binary(&query_vault_info(deps)?),
        QueryMsg::PreviewDeposit { amount } => to_json_binary(&convert_to_shares(deps, amount)),
        QueryMsg::PreviewWithdraw { amount } => to_json_binary(&convert_to_assets(deps, amount)),
        QueryMsg::TotalAssets {} => {
            let state = STATE.load(deps.storage)?;
            to_json_binary(&state.staked_base_tokens)
        }
        QueryMsg::TotalVaultTokenSupply {} => {
            let state = STATE.load(deps.storage)?;
            to_json_binary(&state.vault_token_supply)
        }
    }
}
