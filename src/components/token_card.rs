use yew::prelude::*;
use wasm_bindgen::prelude::*;
use ethers::core::{types::{U256, H256}, utils::format_units};
use web_sys::HtmlInputElement;
use crate::{components::tx_card::TxCard};

#[path="../lib.rs"]
mod lib;
use lib::{fetch_erc20_information, ERC20Information};

#[path="../helpers.rs"]
mod helpers;
use helpers::{decode_hex, wait_receipt};

#[wasm_bindgen(module = "/src/js/metamask.js")]
extern "C" {
    #[wasm_bindgen(js_name = "transfer")]
    #[wasm_bindgen(catch)]
    pub async fn transfer(
        token_address: String, 
        recipient: String, 
        amount: String, 
        decimals: i32
    ) -> Result<JsValue, JsValue>;
}


#[derive(Clone, Debug, PartialEq, Properties)]
pub struct TokenCardProps {
    pub token_address: String,
    pub user_address: String,
}

pub struct TokenCard {
    symbol: String,
    decimals: i32,
    balance: U256,
    error: Option<String>,
    tx: Option<String>,
    tx_processed: Option<bool>,
    // handle refs for DOM elements
    to: NodeRef, 
    amount: NodeRef,
}

pub enum TokenCardMsg {
    FillERC20(ERC20Information),
    SetError(String),
    SetTx(String),
    SetTxProcessed(bool),    
    EnableListener(JsValue),
    Transfer,
}

impl Component for TokenCard {
    type Message = TokenCardMsg;
    type Properties = TokenCardProps;

    fn create(ctx: &Context<Self>) -> Self {        
        let token_address = ctx.props().token_address.clone();
        let user_address = ctx.props().user_address.clone(); 
        ctx.link().send_future(async move {
            match fetch_erc20_information(&token_address, user_address).await {
                Ok(data) => {
                    TokenCardMsg::FillERC20(data)
                },
                Err(_err) => {
                    TokenCardMsg::SetError("Error fetching collateral contract".to_string())
                },
            }
        });
        Self {
            symbol: String::from("fetching.."),
            decimals: 18i32,
            balance: U256::from(0),
            error: None,
            tx: None,
            tx_processed: None,
            to: NodeRef::default(),
            amount: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TokenCardMsg::FillERC20(res) => {
                self.symbol = res.symbol;
                self.decimals = res.decimals as i32;
                self.balance = res.balance;
                true
            },
            TokenCardMsg::Transfer => {
                let token_address = ctx.props().token_address.clone();
                let to_address = self.to.cast::<HtmlInputElement>().unwrap().value();
                let val = self.amount.cast::<HtmlInputElement>()
                    .unwrap()
                    .value();
                let decimals = self.decimals.clone();
                ctx.link().send_future(async move {
                    match transfer(
                        token_address,                        
              to_address, 
                val,
                        decimals
                    ).await {
                        Ok(tx) => TokenCardMsg::EnableListener(tx),
                        Err(err) => {
                            log::error!("Error during transfer {:?}", err);
                            TokenCardMsg::SetError("Error".to_string())
                        }
                    }
                });
                true
            }
            TokenCardMsg::EnableListener(tx) => {
                ctx.link().send_message(TokenCardMsg::SetTx(tx.as_string().unwrap().clone()));
                ctx.link().send_future(async move {
                    // JsValue(String) has 66 bytes => 0x+base64
                    let substring = &tx.as_string().unwrap()[2..]; // splits "0x"
                    let hash_from_hex = decode_hex(substring).unwrap(); // 64 bytes to 32 bytes
                    let from_hex_h256 = H256::from_slice(&hash_from_hex); // H256                    
                    let tx = wait_receipt(from_hex_h256).await; 
                    match tx.as_ref() {
                        Ok(receipt) => {
                            log::info!("Receipt: {:?}", receipt);
                            TokenCardMsg::SetTxProcessed(true)
                        },
                        Err(err) => {
                            log::error!("Error {:?}", err);
                            TokenCardMsg::SetTxProcessed(false)
                        }
                    }
                }); 
                true
            }
            TokenCardMsg::SetTx(hash) => {
                log::info!("Setting tx: {:?}", hash);
                self.tx = Some(hash);
                true
            }
            TokenCardMsg::SetTxProcessed(status) => {
                log::info!("Setting tx processed: {:?}", status);
                self.tx_processed = Some(status);
                true
            }
            TokenCardMsg::SetError(error_msg) => {
                self.error = Some(error_msg);
                true
            }
        }        
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let Self { symbol, decimals, balance, error, to:_, amount:_, tx:_, tx_processed:_} = self;
        html! {
            <div>
                if let Some(error_msg) = error {
                    <div>
                        { error_msg }
                    </div>
                } else {
                    <a                        
                        href={format!("https://rinkeby.etherscan.io/token/{}",ctx.props().token_address.clone())} 
                        target="_blank"
                    >
                        {"Token: "}
                        {"r"}{ symbol }
                    </a>
                    <p>
                        {"Your balance: "}
                        {format_units(balance, *decimals).unwrap()}
                        <br />
                    </p>
                    <div>
                        <p>{"Transfer function"}</p>
                        <input
                            type="text"
                            placeholder="Recipient"
                            ref={&self.to}
                        />
                        <input
                            type="number"
                            placeholder="Amount"
                            ref={&self.amount}
                        />
                        {
                            match &self.tx {
                                Some(hash) => html! {
                                    <TxCard
                                        hash = {hash.to_string()}
                                        status = {self.tx_processed}
                                    />        
                                },                                
                                None => html! {
                                    <button onclick={ctx.link().callback(|_| TokenCardMsg::Transfer)}>
                                        {"Transfer"}
                                    </button>    
                                }
                            }
                        }
                    </div>
                }
            </div>
        }
    }
}
