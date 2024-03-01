use crate::{
    error::{
        ContractError,
        ContractError::{InvalidMerkleProof, InvalidTransactionReceiptToProve},
        ContractResponse,
    },
    helpers::{assert_correct_funds, burn_vault_tokens, mint_vault_tokens},
    msg::FastTransfer,
    state::{AGGREGATOR_CONTRACT, BASE_TOKEN, LP_TOKEN_DENOM, PROCESSED_IDS},
};
use aggregator::aggregator::{LookupHashResponse, QueryMsg as AggQueryMsg};
use cosmwasm_std::{
    coins, Addr, BankMsg, Binary, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128,
};
use hex::{decode as hex_decode, encode as hex_encode};
use sha2::{Digest, Sha256};

const STATIC_ID: u64 = 1;

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
    info: MessageInfo,
    id: u64,
    recipient: String,
) -> ContractResponse {
    let was_fast_transferred = PROCESSED_IDS.has(deps.storage, id);

    if was_fast_transferred {
        // transfer was already processed, funds are kept to rebalance the pool
        return Ok(Response::new());
    }

    let send_msg: CosmosMsg = BankMsg::Send {
        to_address: recipient.to_string(),
        amount: info.funds,
    }
    .into();

    Ok(Response::new().add_message(send_msg))
}

pub fn execute_fast_transfer(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    fast_transfer: FastTransfer,
) -> ContractResponse {
    // Info needed to verify the merkle proof
    let tx_hash_to_prove = fast_transfer.tx_hash_to_prove;
    let all_tx_hashes = fast_transfer.all_tx_hashes;
    let sent_root_hash = fast_transfer.root_hash;
    let chain_id = fast_transfer.chain_id;
    // In real world, amount and receiver should be info that can be pulled
    // from the transaction receipt
    let amount: Uint128 = Uint128::from(fast_transfer.amount);
    let receiver = fast_transfer.receiver;

    // Query the aggregator contract to check if the root hash exists in the keeper
    let aggregator_contract = AGGREGATOR_CONTRACT.load(deps.storage)?;
    let _: LookupHashResponse = deps.querier.query_wasm_smart(
        aggregator_contract,
        &AggQueryMsg::LookupHash {
            chain_id,
            hash: Binary(hex_decode(&sent_root_hash)?),
        },
    )?;

    // Verify the transaction receipt is the hash as the undice wanting to be merkle proofed
    if !all_tx_hashes.contains(&tx_hash_to_prove) {
        return Err(InvalidTransactionReceiptToProve {});
    }

    // create a Sha256 object
    let mut hasher = Sha256::new();

    let data_to_hash: Vec<u8> =
        all_tx_hashes
            .iter()
            .fold(Vec::new(), |mut data_to_hash, tx_hash| {
                data_to_hash.extend(hex_decode(tx_hash).unwrap());
                data_to_hash
            });

    // Add data to hash
    hasher.update(&data_to_hash);

    // Read hash digest and consume hasher
    let root_hash = hasher.finalize();

    let root_hash_hex = hex_encode(root_hash);

    // Verify the root hash is the same as the one in the transaction
    if root_hash_hex != sent_root_hash {
        return Err(InvalidMerkleProof {});
    }

    // Verify denom in is the same as the base token
    let base_token = BASE_TOKEN.load(deps.storage)?;
    if fast_transfer.denom != base_token {
        return Err(ContractError::InvalidFastTransferDenom {});
    }

    // Send the amount to the receiver
    let bank_msg = BankMsg::Send {
        to_address: receiver,
        amount: coins(amount.u128(), base_token),
    };

    // Store the unique id to flag it as fast transferred
    PROCESSED_IDS.save(deps.storage, STATIC_ID, &true)?;

    Ok(Response::new().add_message(bank_msg))
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    pub fn test() {
        let all_tx_hashes: Vec<String> = vec!["68656C6C6F206D79206E616D6520697320626F62".to_string(), "68656C6C6F206D79206E616D6520697320616C696365".to_string()];
        // create a Sha256 object
        let mut hasher = Sha256::new();

        let data_to_hash: Vec<u8> =
            all_tx_hashes
                .iter()
                .fold(Vec::new(), |mut data_to_hash, tx_hash| {
                    data_to_hash.extend(hex_decode(tx_hash).unwrap());
                    data_to_hash
                });

        // Add data to hash
        hasher.update(&data_to_hash);

        // Read hash digest and consume hasher
        let root_hash = hasher.finalize();

        let root_hash_hex = hex_encode(root_hash);
        println!("root hash hex: {:?}", root_hash_hex);
    }
}