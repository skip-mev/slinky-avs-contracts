use cosmwasm_schema::cw_serde;
use rs_merkle::Hasher;
use tiny_keccak::{Hasher as KeccakHasher, Keccak};

#[cw_serde]
pub struct Keccak256Algorithm {}

impl Hasher for Keccak256Algorithm {
    type Hash = [u8; 32]; // Keccak-256 outputs 32 bytes

    fn hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Keccak::v256();
        let mut output = [0u8; 32]; // Create a buffer for the hash

        hasher.update(data);
        hasher.finalize(&mut output);

        output
    }
}
