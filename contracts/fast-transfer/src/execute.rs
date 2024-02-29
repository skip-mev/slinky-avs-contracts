use cosmwasm_std::{coins, Addr, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128};

use crate::{
    error::ContractResponse,
    helpers::{assert_correct_funds, burn_vault_tokens, mint_vault_tokens},
    state::{BASE_TOKEN, LP_TOKEN_DENOM, PROCESSED_IDS},
};

pub fn execute_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    recipient: Addr,
) -> ContractResponse {
    let base_token = BASE_TOKEN.load(deps.storage)?;
    let vt_denom = LP_TOKEN_DENOM.load(deps.storage)?;

    assert_correct_funds(&info, &base_token, amount)?;

    let (mint_msg, mint_amount) = mint_vault_tokens(deps, env, amount, &vt_denom)?;

    // Send minted vault tokens to recipient
    let send_msg: CosmosMsg = BankMsg::Send {
        to_address: recipient.to_string(),
        amount: coins(mint_amount.u128(), vt_denom),
    }
    .into();

    Ok(Response::new().add_message(mint_msg).add_message(send_msg))
}

pub fn execute_withdraw(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    recipient: Addr,
) -> ContractResponse {
    let base_token = BASE_TOKEN.load(deps.storage)?;
    let vt_denom = LP_TOKEN_DENOM.load(deps.storage)?;

    // Check that only vault tokens were sent and that the amount is correct
    assert_correct_funds(&info, &vt_denom, amount)?;

    // Calculate claim amount and create msg to burn vault tokens
    let (burn_msg, claim_amount) = burn_vault_tokens(deps.branch(), &env, amount, &vt_denom)?;

    let send_msg: CosmosMsg = BankMsg::Send {
        to_address: recipient.to_string(),
        amount: coins(claim_amount.u128(), base_token),
    }
    .into();

    Ok(Response::new().add_message(send_msg).add_message(burn_msg))
}

pub fn execute_slow_transfer(
    deps: DepsMut,
    id: u64,
    recipient: String,
    amount: Uint128,
) -> ContractResponse {
    let was_fast_transferred = PROCESSED_IDS.has(deps.storage, id);

    if was_fast_transferred {
        // transfer was already processed, funds are kept to rebalance the pool
        return Ok(Response::new());
    }

    let base_token = BASE_TOKEN.load(deps.storage)?;

    let send_msg: CosmosMsg = BankMsg::Send {
        to_address: recipient.to_string(),
        amount: coins(amount.u128(), base_token),
    }
    .into();

    Ok(Response::new().add_message(send_msg))
}
