use crate::merkle::Keccak256Algorithm;
use aggregator::aggregator::{LookupHashResponse, QueryMsg as AggQueryMsg};
use cosmwasm_std::{coins, Addr, BankMsg, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128};
use rs_merkle::{Hasher, MerkleProof, algorithms::Sha256};

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
    // Leaf to prove inclusion in merkle root
    let transaction_receipt = fast_transfer.transaction_receipt;
    // Info needed to verify the merkle proof
    let branch = fast_transfer.branch;
    let root_hash = fast_transfer.root_hash;
    let indices = fast_transfer.indices;
    let total_leaves = fast_transfer.total_leaves;
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
            hash: root_hash.into(),
        },
    )?;

    // Verify the transaction receipt is the hash as the undice wanting to be merkle proofed
    let receipt_hash = Sha256::hash(&transaction_receipt);
    if !branch.contains(&receipt_hash) {
        return Err(InvalidTransactionReceiptToProve {});
    }

    // Create the merkle proof and verify it against the root hash retrieved from the aggregator
    let merkle_proof: MerkleProof<Sha256> = MerkleProof::new(branch.clone());
    if !merkle_proof.verify(root_hash, &indices, &branch, total_leaves) {
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
    use rs_merkle::MerkleTree;
    use hex::encode as hex_encode;

    #[test]
    fn test_merkle() {
        let mut leaves: Vec<[u8; 32]> = vec![];
        for i in 0u8..10 {
            let leaf = [i; 32];
            leaves.push(leaf);
        }

        let tree: MerkleTree<Sha256> = rs_merkle::MerkleTree::from_leaves(&leaves);
        let root = tree.root();
        
        // Convert root to hex string and print
        let root_hex = hex_encode(root);
        println!("Root: {:?}", root_hex);
    }
}