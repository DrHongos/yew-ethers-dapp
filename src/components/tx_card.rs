use yew::prelude::*;

#[path="../helpers.rs"]
mod helpers;
use helpers::short_address;

#[derive(Properties, PartialEq)]
pub struct TxProps {
    pub hash: String,
    pub status: Option<bool>,
}

#[function_component(TxCard)]
pub fn tx_box(props: &TxProps) -> Html {
    html! {
        <div>
            <header>
                <a                        
                    href={format!("https://rinkeby.etherscan.io/tx/{}",props.hash.clone())} 
                    target="_blank"
                >

                    {"Tx: "}{short_address(&props.hash)}
                </a>

                if let Some(status) = props.status {
                    if status {
                        <span>{"Success"}</span>
                    } else {
                        <span>{"Error"}</span>
                    }                    
                } else {
                    <span>{"Awaiting confirmation"}</span>
                }
            </header>
        </div>        
    }
}