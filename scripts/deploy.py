import os
import sys
import toml
import httpx
import time
from hashlib import sha256
from base64 import b64encode
from bip_utils import Bip39SeedGenerator, Bip44, Bip44Coins
from cosmpy.aerial.client import LedgerClient, NetworkConfig
from cosmpy.aerial.tx import Transaction, SigningCfg
from cosmpy.aerial.wallet import LocalWallet
from cosmpy.crypto.keypairs import PrivateKey
from cosmpy.protos.cosmwasm.wasm.v1.tx_pb2 import (
    MsgStoreCode, 
    MsgInstantiateContract, 
    )
from cosmpy.common.utils import json_encode

CHAIN = sys.argv[1]
NETWORK = sys.argv[2]

# Match the CHAIN to the file name in the configs folder
found_config = False
for file in os.listdir("configs"):
    if file == f"{CHAIN}.toml":
        config = toml.load(f"configs/{file}")
        found_config = True
        break

# Raise exception if config not found
if not found_config:
    raise Exception(
        f"Could not find config for chain {CHAIN}; Must enter a chain as 1st cli arg."
    )

# Choose network to deploy to based on cli args
if NETWORK == "mainnet":
    REST_URL = config["MAINNET_REST_URL"]
    RPC_URL = config["MAINNET_RPC_URL"]
    CHAIN_ID = config["MAINNET_CHAIN_ID"]
elif NETWORK == "testnet":
    REST_URL = config["TESTNET_REST_URL"]
    RPC_URL = config["TESTNET_RPC_URL"]
    CHAIN_ID = config["TESTNET_CHAIN_ID"]
else:
    raise Exception(
        "Must specify either 'mainnet' or 'testnet' for 2nd cli arg."
    )

ADDRESS_PREFIX = config["ADDRESS_PREFIX"]
DENOM = config["DENOM"]
GAS_PRICE = config["GAS_PRICE"]

# Contract Paths
AVS_CONTRACT_PATH = config["AVS_CONTRACT"]
FAST_TRANSFER_CONRACT_PATH = config["FAST_TRANSFER_CONTRACT"]

BASE_DENOM = config["BASE_DENOM"]

MNEMONIC = config["MNEMONIC"]
del config["MNEMONIC"]

DEPLOYED_CONTRACTS_INFO = {}

def main():
    # Create network config and client
    cfg = NetworkConfig(
        chain_id=CHAIN_ID,
        url=f"rest+{REST_URL}",
        fee_minimum_gas_price=.01,
        fee_denomination=DENOM,
        staking_denomination=DENOM,
    )
    client = LedgerClient(cfg)

    # Create wallet from mnemonic
    wallet = create_wallet(client)
            
    # Store and instantiate placeholder contract
    avs_contract_code_id = store_contract(
        client, 
        wallet, 
        AVS_CONTRACT_PATH, 
        "avs", 
        None,
    )
    print("AVS Contract Code ID: ", avs_contract_code_id)
    avs_contract_address = instantiate_contract(
        client, 
        wallet, 
        str(wallet.address()),
        avs_contract_code_id, 
        {}, 
        "Slinky Plus Plus AVS", 
        "avs_contract"
    )
    print("AVS Contract Address: ", avs_contract_address)

    fast_transfer_code_id = store_contract(
        client,
        wallet,
        FAST_TRANSFER_CONRACT_PATH,
        "fast_transfer",
        None,
    )
    print("Fast Transfer Code ID: ", fast_transfer_code_id)
    
    ft_init_args = {
        "base_denom": BASE_DENOM,
        "lp_sub_denom": "fast_transfer_lp",
        "aggregator_contract": avs_contract_address,
    }
    fast_transfer_contract_address = instantiate_contract(
        client, 
        wallet, 
        str(wallet.address()),
        fast_transfer_code_id, 
        ft_init_args, 
        "Fast Transfer", 
        "fast_transfer_contract"
    )
    print("Fast Transfer Contract Address: ", fast_transfer_contract_address)
    
    
def create_tx(msg,
              client, 
              wallet, 
              gas_limit: int, 
              fee: str,
              ) -> tuple[bytes, str]:
    time.sleep(5)
    tx = Transaction()
    tx.add_message(msg)
    
    # Get account
    account = client.query_account(str(wallet.address()))
    
    # Seal, Sign, and Complete Tx
    tx.seal(
        signing_cfgs=[SigningCfg.direct(wallet.public_key(), account.sequence)], 
        fee=fee, 
        gas_limit=gas_limit
    )
    tx.sign(wallet.signer(), client.network_config.chain_id, account.number)
    tx.complete()
    
    return tx

    
def create_wallet(client) -> LocalWallet:
    """ Create a wallet from a mnemonic and return it"""
    seed_bytes = Bip39SeedGenerator(MNEMONIC).Generate()
    bip44_def_ctx = Bip44.FromSeed(seed_bytes, Bip44Coins.COSMOS).DeriveDefaultPath()
    wallet = LocalWallet(
        PrivateKey(bip44_def_ctx.PrivateKey().Raw().ToBytes()), 
        prefix=ADDRESS_PREFIX
    )  
    balance = client.query_bank_balance(str(wallet.address()), DENOM)
    print("Wallet Address: ", wallet.address(), " with account balance: ", balance)
    return wallet


def store_contract(
    client, 
    wallet, 
    file_path, 
    name, 
    permissioned_uploader_address
) -> int:
    gas_limit = 5000000
        
    msg = MsgStoreCode(
        sender=str(wallet.address()),
        wasm_byte_code=open(file_path, "rb").read(),
        instantiate_permission=None
    )
    store_tx = create_tx(
        msg=msg, 
        client=client, 
        wallet=wallet, 
        gas_limit=gas_limit,
        fee=f"{int(GAS_PRICE*gas_limit)}{DENOM}"
    )
    tx_hash = sha256(store_tx.tx.SerializeToString()).hexdigest()
    print("Tx hash: ", tx_hash)
    resp: httpx.Response = broadcast_tx(store_tx)
    contract_code_id: str = get_attribute_value(resp, "store_code", "code_id")
    return int(contract_code_id)


def instantiate_contract(client, wallet, admin, code_id, args, label, name) -> str:
    gas_limit = 300000
    msg = MsgInstantiateContract(
        sender=str(wallet.address()),
        admin=admin,
        code_id=code_id,
        msg=json_encode(args).encode("UTF8"),
        label=label,
    )
    instantiate_tx = create_tx(
        msg=msg, 
        client=client, 
        wallet=wallet, 
        gas_limit=gas_limit,
        fee=f"{int(GAS_PRICE*gas_limit)}{DENOM}"
    )
    tx_hash = sha256(instantiate_tx.tx.SerializeToString()).hexdigest()
    print("Tx hash: ", tx_hash)
    resp: httpx.Response = broadcast_tx(instantiate_tx)
    contract_address: str = get_attribute_value(resp, "instantiate", "_contract_address")
    return contract_address

def broadcast_tx(tx) -> httpx.Response:
    tx_bytes = tx.tx.SerializeToString()
    encoded_tx = b64encode(tx_bytes).decode("utf-8")
    data = {
        'jsonrpc': '2.0',
        'method': "broadcast_tx_sync",
        'params': [encoded_tx],
        'id': 1
    }
    postResp = httpx.post(RPC_URL, json=data, timeout=60)
    print("postResp.json(): ", postResp.json())
    print("Sleeping for 20 seconds...")
    time.sleep(20)
    resp = httpx.get(
        REST_URL + f"/cosmos/tx/v1beta1/txs/{sha256(tx_bytes).hexdigest()}", 
        timeout=60
    )
    return resp


def get_attribute_value(resp, event_type, attr_key):
    for event in resp.json()['tx_response']['logs'][0]['events']:
        if event['type'] == event_type:
            for attr in event['attributes']:
                if attr['key'] == attr_key:
                    return attr['value']
    return None
    
    
if __name__ == "__main__":
    main()