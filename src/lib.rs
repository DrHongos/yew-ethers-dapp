#![allow(dead_code)]
use ethers::{contract::abigen, prelude::*};
use std::sync::Arc;

//////////////////////////////////////////////////////////////////
// 
//         PROVIDER QUERIES
// 
/////////////////////////////////////////////////////////////////
pub async fn get_native_balance(address: String) -> Result<U256, String>{
    let endpoint = "wss://rinkeby.infura.io/ws/v3/88371c5dbe284f97bb2789cf7f9ca6f1";
    let provider = Provider::new(Ws::connect(endpoint).await.unwrap());
    let client = Arc::new(provider);
    let address = address.parse::<Address>().unwrap();
    let block = None;
    match client.get_balance(address, block).await {
        Ok(bal) => Ok(bal),
        Err(err) => Err(err.to_string())
    }
}


//////////////////////////////////////////////////////////////////
// 
//         SMART CONTRACTS QUERIES
// 
/////////////////////////////////////////////////////////////////
abigen!(
    IERC20,
    "src/abis/erc20.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ERC20Information {
    pub symbol: String,
    pub decimals: i32,
    pub balance: U256,
}

pub async fn fetch_erc20(token_address: H160) -> Result<String, String>  {
    let endpoint = "wss://rinkeby.infura.io/ws/v3/88371c5dbe284f97bb2789cf7f9ca6f1";
    let provider = Provider::new(Ws::connect(endpoint).await.unwrap());
    let client = Arc::new(provider);    
    let token_contract = IERC20::new(token_address, Arc::clone(&client));
    match token_contract.symbol().call().await {
        Ok(symbol) => Ok(symbol),
        Err(_err) => Err("No symbol found".to_string())
    }
}


pub async fn fetch_erc20_information(token_address: &str, user_address: String) -> Result<ERC20Information, String>  {
    let endpoint = "wss://rinkeby.infura.io/ws/v3/88371c5dbe284f97bb2789cf7f9ca6f1";
    let provider = Provider::new(Ws::connect(endpoint).await.unwrap());
    let client = Arc::new(provider);
//   let client = Provider::<Http>::try_from("https://rpc.gnosischain.com");//?
    let address = token_address.parse::<Address>();
    let token_contract = IERC20::new(address.unwrap(), Arc::clone(&client));
    let user_address_parsed = user_address.parse::<Address>().unwrap();
    let symbol = token_contract.symbol().call().await.unwrap();
    let decimals = token_contract.decimals().call().await.unwrap().into();
    let balance = token_contract.balance_of(user_address_parsed).call().await.unwrap();
    Ok(
        ERC20Information {
            symbol,
            decimals,
            balance,
        }  
    )
}
