use cosmwasm_std::{Deps, StdResult};

use crate::{
    msg::VaultInfoResponse,
    state::{BASE_TOKEN, LP_TOKEN_DENOM},
};

pub fn query_vault_info(deps: Deps) -> StdResult<VaultInfoResponse> {
    let base_token = BASE_TOKEN.load(deps.storage)?;
    let vault_token_denom = LP_TOKEN_DENOM.load(deps.storage)?;

    Ok(VaultInfoResponse {
        base_token: base_token.to_string(),
        lp_token: vault_token_denom,
    })
}
