use cosmwasm_std::{Coin, OverflowError, StdError};
use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error(transparent)]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.

    // #[error(transparent)]
    // CwDex(#[from] CwDexError),
    #[error(transparent)]
    Overflow(#[from] OverflowError),

    #[error("Unexpected funds sent. Expected: {expected:?}, Actual: {actual:?}")]
    UnexpectedFunds {
        expected: Vec<Coin>,
        actual: Vec<Coin>,
    },

    #[error("Invalid fast transfer merkle proof")]
    InvalidMerkleProof {},

    #[error("Invalid fast transfer denom")]
    InvalidFastTransferDenom {},

    #[error("Invalid tx receipt in merkle proof")]
    InvalidTransactionReceiptToProve,

    #[error("Invalid hex")]
    InvalidHex(#[from] FromHexError),
}

pub type ContractResult<T> = Result<T, ContractError>;
pub type ContractResponse = ContractResult<cosmwasm_std::Response>;
