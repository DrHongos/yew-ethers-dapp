use yew::prelude::*;
use ethers::core::{types::{U256, Address}, utils::format_units};
use wasm_bindgen::prelude::*;
use js_sys::Reflect;
use web_sys::HtmlInputElement;

mod components;
use crate::{ components::{token_card::TokenCard}};

mod helpers;
use helpers::short_address;

mod lib;
use lib::{get_native_balance, fetch_erc20};

enum Msg {
    ConnectMetamask,
    ConnectRinkeby,
    SignMessage,
    SearchERC20,
    AddToken(String),
    FetchBalance(String),
    SetBalance(U256),
    SetClient(JsValue),
    MessagesUser(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct WalletContext {
    pub client: Option<JsValue>, // JsValue = provider on JS side
    pub address: Option<String>,
    pub chain_id: Option<String>,
}
// metamask.js contains wasm-bindgen function shims to call the browser from js side.
#[wasm_bindgen(module = "/src/js/metamask.js")]
extern "C" {
    #[wasm_bindgen(js_name = "getProviderData")]
    #[wasm_bindgen(catch)]
    pub async fn getProviderData() -> Result<JsValue, JsValue>;
   
    #[wasm_bindgen(js_name = "signMessage")]
    #[wasm_bindgen(catch)]
    pub async fn signMessage() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = "setRinkeby")]
    #[wasm_bindgen(catch)]
    pub async fn setRinkeby() -> Result<JsValue, JsValue>;

}

struct Model {
    msgs: Option<String>,
    balance_native: Option<U256>,
    wallet_context: WalletContext,
    erc20_added: Vec<String>,
    input: NodeRef,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // this ensure persistence of the connected wallet (and launches automatically)
        ctx.link().send_future(async move {
            Msg::ConnectMetamask
        });
        Self {
            msgs: None,
            balance_native: None,
            erc20_added: Vec::new(),
            input: NodeRef::default(),
            wallet_context: WalletContext {
                client: None,
                chain_id: None,
                address: None,
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {            
            Msg::ConnectMetamask => {
                log::info!("Connecting!");
                ctx.link().send_future(async move {
                    match getProviderData().await {
                        Ok(accs) => {
                            Msg::SetClient(accs)
                        },
                        Err(err) => {
                            log::error!("Error {:?}", err);
                            Msg::MessagesUser("No metamask!".to_owned())
                        },
                    }
                });
                false
            }
            Msg::ConnectRinkeby => {
                ctx.link().send_future(async {
                    match setRinkeby().await {
                        Ok(_) => Msg::ConnectMetamask, // not updating the state
                        Err(err) => {
                            log::error!("Error {:?}", err);
                            Msg::MessagesUser("Error on change".to_owned())
                        }
                    }
                });
                false
            }
            Msg::SignMessage => {
                ctx.link().send_future(async {
                    match signMessage().await {
                        Ok(msg) => {
                            log::info!("Message signed {:?}", msg);
                            Msg::MessagesUser("Correctly signed".to_string())
                        },
                        Err(err) => {
                            log::error!("Error signing {:?}", err);
                            Msg::MessagesUser("Error while signing".to_string())
                        }
                    }
                });
                true
            }
            Msg::FetchBalance(address) => {
                ctx.link().send_future(async move {
                    match get_native_balance(address).await {
                        Ok(bal) => Msg::SetBalance(bal),
                        Err(err) => Msg::MessagesUser(err)
                    }   
                });
                false
            }
            Msg::SetBalance(bal) => {
                self.balance_native = Some(bal);
                true
            }
            Msg::SetClient(provider) => {                
                self.wallet_context.client = Some(provider);
                self.wallet_context.address = self.get_address();
                self.wallet_context.chain_id = self.get_chain_id();
                let user_address = self.get_address().unwrap();
                ctx.link().send_message(Msg::FetchBalance(user_address));
                true   
            }
            Msg::SearchERC20 => {
                let poss_address = self.input.cast::<HtmlInputElement>().unwrap().value();
                let address = poss_address.parse::<Address>();
                match address {
                    Ok(address_parsed) => {
                        log::info!("searching {:?}", poss_address);
                        ctx.link().send_future(async move {
                            match fetch_erc20(address_parsed).await {
                                Ok(symbol) => {                        
                                    Msg::MessagesUser(format!("Found {}!", symbol));
                                    Msg::AddToken(poss_address)
                                },
                                Err(_err) => {
                                    Msg::MessagesUser("Rejected!".to_string())
                                }
                            }    
                        });                        
                    },
                    Err(err) => {
                        log::info!("Error on input {:?}", err);
                        Msg::MessagesUser("Error on input".to_string());
                    }
                }
                true
            }
            Msg::AddToken(address) => {
                self.erc20_added.push(address);
                true
            }
            Msg::MessagesUser(msg) => {
                log::info!("{:?}", msg);
                self.msgs = Some(msg);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Self {msgs, wallet_context, balance_native, erc20_added, input:_} = self;
        let link = ctx.link();
        html! {
            <div>
                <h3>{ "Metamask connection" }</h3>
                if let Some(address) = &self.get_address() {
                    <>{short_address(&address)}</>
                } else {
                    <button 
                        onclick={link.callback(|_| {
                            Msg::ConnectMetamask
                        })}
                    >                            
                        {"Connect"}
                    </button>
                }
                if let Some(chain) =  &self.get_chain_id() {
                    if chain != "0x4" {
                        <div>                            
                            {" connected to chain "}{chain}
                            <button
                                onclick={link.callback(|_| {
                                    Msg::ConnectRinkeby
                                })}
                            >{"Change to Rinkeby"}</button>
                        </div>
                    } else {
                        if let Some(balance) = balance_native {
                            <p>{"Balance: "} { format_units(balance, 18).unwrap() }{" rEth"}</p>
                        }
                        <button onclick={ctx.link().callback(|_| Msg::SignMessage)}>
                            {"Sign a message"}
                        </button>
                    }
                }

                <h3>{"ERC20 contracts"}</h3>
                // as an example, rDAI
                if let Some(user_address) = &wallet_context.address {
                    <TokenCard
                        token_address = {"0xc7AD46e0b8a400Bb3C915120d284AafbA8fc4735".to_string()}
                        user_address = {user_address.clone()}
                    />
                }
                <h3>{"Add ERC20"}</h3>
                <input
                    type="text"
                    ref={&self.input}
                    placeholder="Rinkeby ERC20 address"
                    onchange={ctx.link().callback(|_| Msg::SearchERC20)}
                />

                if erc20_added.len() > 0 {
                    <div>
                        {self.list_of_added_erc20()}                    
                    </div>
                }
                <h3>{"Messages: "}</h3>
                if let Some(msg) = msgs {
                    <h1> { msg } </h1>
                }

            </div>
        }
    }
}

impl Model {
    fn get_chain_id(&self) -> Option<String> {
        if let Some(client) = &self.wallet_context.client {
            match Reflect::get(
                client.as_ref(), 
                &JsValue::from("chainId")
            ) {
                Ok(val) => {
                    match val.as_string() {
                        Some(chain) => Some(chain),
                        None => None
                    }
                }
                Err(_err) => None
            }
        } else {
            None
        }
    }
    fn get_address(&self) -> Option<String> {
        if let Some(client) = &self.wallet_context.client {
            match Reflect::get(
                client.as_ref(), 
                &JsValue::from("selectedAddress")
            ) {
                Ok(val) => {
                    match val.as_string() {
                        Some(address) => Some(address),
                        None => None
                    }
                },
                Err(_err) => None
            }
        } else {
            None
        }
    }
    fn list_of_added_erc20(&self) -> Html {
        let erc20_tokens = self.erc20_added
            .iter()
            .map(|data| 
        {
            html! {
                <div>
                    <div>
                        <TokenCard
                            token_address = {data.clone()}
                            user_address = {self.wallet_context.address.as_ref().unwrap().clone()}
                        />
                    </div>
                </div>
            }
        });
        html! {
            <div>
                { for erc20_tokens }
            </div>
        }

    }
}


fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    yew::start_app::<Model>();
}