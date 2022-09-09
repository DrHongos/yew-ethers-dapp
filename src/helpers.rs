#![allow(dead_code)]

use ethers::prelude::*;
use std::num::ParseIntError;

pub fn short_address(address: &String) -> String {
    let shortened = format!("{}...{}", &address[0..5], &address[&address.len()-5..]);
    shortened.to_owned()
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

async fn check_tx(tx_hash: H256) -> Option<TransactionReceipt> {
    let endpoint = "wss://rinkeby-light.eth.linkpool.io/ws"; // public rpc node
    let provider = Provider::new(Ws::connect(endpoint).await.unwrap());
    let receipt = provider.get_transaction_receipt(tx_hash).await;
    match receipt {
        Ok(res) => {
            if let Some(rec) = res {
                return Some(rec)
            } else {
                return None
            }
        }
        Err(_err) => return None
    }
}

pub async fn wait_receipt(tx_hash: H256) -> Result<Option<TransactionReceipt>, String> {    
    let mut receipt: Option<TransactionReceipt> = None;
    while receipt.is_none() {
        if let Some(rec) = check_tx(tx_hash).await {
            receipt = Some(rec);
        } else {
            continue
        }
    }
    Ok(receipt)
}
