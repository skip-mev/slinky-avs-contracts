use crate::contract::execute::write_merkle_roots;
use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use std::collections::BTreeMap;

use crate::error::{ContractError, ContractResult};
use crate::msg::{ExecuteMsg, InstantiateMsg, SudoMsg, VoteExtension};
use crate::state::{MERKLE_ROOTS, QUARUM, STAKE_MAP};
use aggregator::aggregator::{LookupHashResponse, QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:slinky-avs-contracts";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const CACHE_SIZE: usize = 6;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> ContractResult<Response> {
    match msg {
        ExecuteMsg::Stake { .. } => {
            let sender_staked_coins = STAKE_MAP
                .load(deps.storage, info.sender.clone())
                .unwrap_or_else(|_| vec![]);

            let updated_coins =
                info.funds
                    .iter()
                    .fold(sender_staked_coins, |mut staked_coins, sent_coin| {
                        match staked_coins
                            .iter_mut()
                            .find(|staked_coin| staked_coin.denom == sent_coin.denom)
                        {
                            Some(staked_coin) => {
                                staked_coin.amount = staked_coin
                                    .amount
                                    .checked_add(sent_coin.amount)
                                    .expect("Overflow in coin amount")
                            }
                            None => staked_coins.push(sent_coin.clone()),
                        }
                        staked_coins
                    });

            STAKE_MAP.save(deps.storage, info.sender, &updated_coins)?;

            Ok(Response::new().add_attribute("action", "stake"))
        }
        ExecuteMsg::SubmitRoot {chain_id, root} => {
            write_merkle_roots(deps, vec![(chain_id, root)])
        }
    }
}

/// sudo is the main entrypoint for the contract. It can only be called by modules.
/// This function handles:
///  * the deserialization of the input msg
///  * aggregation over the VE light client inputs
///  * updating contract state to store agreed upon state updates
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _: Env, msg: SudoMsg) -> ContractResult<Response> {
    // Store a map of chain_id to Vec<VoteExtension>
    // Each chain has its own set of VoteExtension--settled separately
    let mut data_map: BTreeMap<String, Vec<VoteExtension>> =
        BTreeMap::<String, Vec<VoteExtension>>::new();
    for generic_hash_vp in msg.data {
        let hash_vp: VoteExtension = From::from(generic_hash_vp);
        println!("hash_vp: {:?}", hash_vp);
        for (chain_id, _) in hash_vp.vote.roots.iter() {
            match data_map.get(chain_id) {
                Some(result) => {
                    let existing_data: Vec<VoteExtension> = Vec::<VoteExtension>::new();
                    result.to_vec().push(hash_vp.clone());
                    data_map.insert(chain_id.clone(), existing_data);
                }
                None => {
                    let new_data: Vec<VoteExtension> = vec![hash_vp.clone()];
                    data_map.insert(chain_id.clone(), new_data);
                }
            }
        }
    }

    // aggregate over all the collected vote data
    let mut vote_roots: Vec<(String, Binary)> = Vec::new();
    for (chain_id, vote_extensions) in data_map.iter() {
        if let Some(root) = aggregate_ves(chain_id.clone(), vote_extensions.to_vec()) {
            vote_roots.push((chain_id.clone(), root));
        }
    }
    write_merkle_roots(deps, vote_roots)
}

fn aggregate_ves(chain_id: String, votes: Vec<VoteExtension>) -> Option<Binary> {
    // aggregate over all the collected vote data
    let mut hashes_to_vp: BTreeMap<Binary, u64> = BTreeMap::new();
    let mut max_power: u64 = 0;
    let mut total_power: u64 = 0;
    let mut best_hash: Option<Binary> = None;
    for ve in votes {
        let voted_root = ve.vote.roots.get(&chain_id)?;
        let mut existing_power: u64 = 0;
        if let Some(power) = hashes_to_vp.get(voted_root) {
            existing_power = *power;
        }
        total_power += ve.ve_power;
        let new_hash_power = existing_power + ve.ve_power;
        if new_hash_power > max_power {
            max_power = new_hash_power;
            best_hash = Some(voted_root.clone());
        }
        hashes_to_vp.insert(voted_root.clone(), new_hash_power);
    }
    if (max_power as f64) / (total_power as f64) >= QUARUM {
        return best_hash.clone();
    }
    None
}

/// instantiate is used to construct the contract
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

pub mod execute {
    use super::*;
    use crate::state::ChainHashes;

    /// write_merkle_roots implements the state update method of the contract.
    /// Merkle roots are input using a map of chain_id to hash value.
    /// If a chain has reached the maximum cache size, it evicts the oldest entry and
    /// inserts a new one.
    /// Otherwise, it writes a new vector to state for the chain.
    pub fn write_merkle_roots(
        deps: DepsMut,
        merkle_roots: Vec<(String, Binary)>,
    ) -> Result<Response, ContractError> {
        for (chain_id, merkle_hash) in merkle_roots.iter() {
            // Get the existing vector of merkle roots for the chain_id
            // let mut root_set: Vec<Binary>;
            let mut root_set: ChainHashes;
            if MERKLE_ROOTS.has(deps.storage, chain_id.to_string()) {
                root_set = MERKLE_ROOTS.load(deps.storage, chain_id.clone()).unwrap();
                let mut seen = false;
                for root in root_set.hashes.iter() {
                    if root.eq(merkle_hash) {
                        seen = true;
                    }
                }
                if seen {
                    continue;
                }
                if root_set.hashes.len() == root_set.max_size {
                    root_set.hashes.remove(0);
                }
                root_set.hashes.push(merkle_hash.clone());
            } else {
                root_set = ChainHashes {
                    chain_id: chain_id.clone(),
                    hashes: Vec::new(),
                    max_size: CACHE_SIZE,
                };
                root_set.hashes.push(merkle_hash.clone());
            }
            MERKLE_ROOTS.save(deps.storage, chain_id.clone(), &root_set)?;
        }
        Ok(Response::new())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LookupHash { chain_id, hash } => {
            to_json_binary(&query::lookup_hash(deps, chain_id, hash)?)
        }
    }
}

pub mod query {
    use super::*;
    use cosmwasm_std::StdError;

    pub fn lookup_hash(
        deps: Deps,
        chain_id: String,
        hash: Binary,
    ) -> StdResult<LookupHashResponse> {
        let chain_hashes = MERKLE_ROOTS.load(deps.storage, chain_id)?;
        for (index, chain_hash) in chain_hashes.hashes.iter().enumerate() {
            if chain_hash.eq(&hash) {
                return Ok(LookupHashResponse {
                    age: (chain_hashes.hashes.len() - index) as u64,
                });
            }
        }
        Err(StdError::not_found("HashNotFound".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::msg::{GenericVE, Vote};
    use bincode::serialize;
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn do_some_hash_stuff() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let test_case_msg = SudoMsg { data: vec![] };
        assert!(Ok(Response::new()).eq(&sudo(deps.as_mut(), mock_env(), test_case_msg)));

        // let bin = Binary::from_base64("eyJyb290cyI6eyJmb28iOiJZbUZ5In19Cg==").unwrap();

        let mut map_thing = BTreeMap::<String, Binary>::new();
        map_thing.insert(
            "foo".to_string(),
            Binary::from_base64("eyJyb290cyI6eyJmb28iOiJZbUZ5In19Cg").unwrap(),
        );
        let vote_ex = Vote {
            roots: map_thing.clone(),
        };
        println!("vote_ex: {:?}", hex::encode(serialize(&vote_ex).unwrap()));
        let second_case = SudoMsg {
            data: vec![GenericVE {
                vote: cosmwasm_std::Binary(serialize(&vote_ex).unwrap()),
                ve_power: 1000,
            }],
        };

        assert!(Ok(Response::new()).eq(&sudo(deps.as_mut(), mock_env(), second_case)));

        println!(
            "{:?}",
            query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::LookupHash {
                    chain_id: "foo".to_string(),
                    hash: Binary::from_base64("eyJyb290cyI6eyJmb28iOiJZbUZ5In19Cg").unwrap()
                }
            )
            .unwrap()
        );
    }
}
