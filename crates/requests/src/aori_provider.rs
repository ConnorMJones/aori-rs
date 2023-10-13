use serde_json::json;

use websockets::WebSocket;

use std::sync::Arc;

use eyre::Context;

use ethers::{
    prelude::{Ws, LocalWallet},
    signers::Signer,
    providers::{Provider, Middleware},
    types::Signature,
};

use alloy_sol_types::SolStruct;

use alloy_primitives::FixedBytes;

use aori_types::{
    constants::{REQUEST_URL, MARKET_FEED_URL},
    seaport::{OrderParameters, SEAPORT_DOMAIN},
};

pub struct AoriProvider {
    pub request_conn: WebSocket,
    pub feed_conn: WebSocket,
    pub chain_id: u64,
    pub last_id: u64,
    pub wallet_addr: Arc<str>,
    pub wallet_sig: Arc<str>,
}

impl AoriProvider {
    pub async fn new_from_env() -> eyre::Result<Self> {
        let key = std::env::var("PRIVATE_KEY").context("missing PRIVATE_KEY")?;
        let address = std::env::var("WALLET_ADDRESS").context("missing WALLET_ADDRESS")?;
        let node = std::env::var("NODE_URL").context("missing NODE_URL")?;

        let pv = Provider::<Ws>::connect(&node).await?;
        let chain_id = pv.get_chainid().await?.low_u64();

        let wallet = key
                .parse::<LocalWallet>()?
                .with_chain_id(chain_id);
        let sig: Signature = wallet.sign_message(&address.as_str()).await?;
        let request_conn = WebSocket::connect(REQUEST_URL).await?;
        let feed_conn = WebSocket::connect(MARKET_FEED_URL).await?;
        Ok(Self {
            request_conn,
            feed_conn,
            chain_id,
            last_id: 0,
            wallet_addr: address.into(),
            wallet_sig: format!("0x{}", sig).into(),
        })
    }
    pub async fn ping(&mut self) -> eyre::Result<()> {
        self.last_id += 1;
        let ping = json!({
            "id": self.last_id,
            "jsonrpc": "2.0",
            "method": "aori_ping",
            "params": []
        });
        self.request_conn.send_text(ping.to_string()).await?;
        Ok(())
    }

    pub async fn auth_wallet(&mut self) -> eyre::Result<()> {
        self.last_id += 1;
        let auth = json!({
            "id": self.last_id,
            "jsonrpc": "2.0",
            "method": "aori_authWallet",
            "params": [{
                "address": *self.wallet_addr,
                "signature": *self.wallet_sig
            }]
        });
        self.request_conn.send_text(auth.to_string()).await?;
        Ok(())
    }

    pub async fn check_auth(&mut self) -> eyre::Result<()> {
        self.last_id += 1;
        let auth = json!({
            "id": self.last_id,
            "jsonrpc": "2.0",
            "method": "aori_checkAuth",
            "params": [{
                "auth": *self.wallet_sig
            }]
        });
        self.request_conn.send_text(auth.to_string()).await?;
        Ok(())
    }

    pub async fn view_orderbook(&mut self, base: &str, quote: &str) -> eyre::Result<()> {
        self.last_id += 1;
        let req = json!({
            "id": self.last_id,
            "jsonrpc": "2.0",
            "method": "aori_viewOrderbook",
            "params": [{
                "chainId": self.chain_id,
                "query": {
                    "base": base,
                    "quote": quote,
                }
            }]
        });
        self.request_conn.send_text(req.to_string()).await?;
        Ok(())
    }

    pub async fn make_order(&mut self, order_params: OrderParameters) -> eyre::Result<()> {
        self.last_id += 1;
        let sig: FixedBytes<32> = order_params.eip712_signing_hash(&*SEAPORT_DOMAIN);
        let order = json!({
            "id": self.last_id,
            "jsonrpc": "2.0",
            "method": "aori_makeOrder",
            "params": [{
                "order": {
                    "signature": format!("{}", sig),
                    "parameters": {
                        "offerer": format!("{}", order_params.offerer),
                        "zone": format!("{}", order_params.zone),
                        "zoneHash": format!("{}", order_params.zoneHash),
                        "startTime": format!("{}", order_params.startTime),
                        "endTime": format!("{}", order_params.endTime),
                        "orderType": order_params.orderType as u8,
                        "offer": [{
                            "itemType": order_params.offer[0].itemType as u8,
                            "token": format!("{}", order_params.offer[0].token),
                            "identifierOrCriteria": order_params.offer[0].identifierOrCriteria.to::<i16>(),
                            "startAmount": order_params.offer[0].startAmount.to::<u128>(),
                            "endAmount": order_params.offer[0].endAmount.to::<u128>()
                        }],
                        "consideration": [{
                            "itemType": order_params.consideration[0].itemType as u8,
                            "token": format!("{}", order_params.consideration[0].token),
                            "identifierOrCriteria": order_params.consideration[0].identifierOrCriteria.to::<i16>(),
                            "startAmount": order_params.consideration[0].startAmount.to::<u128>(),
                            "endAmount": order_params.consideration[0].endAmount.to::<u128>(),
                            "recipient": format!("{}", order_params.consideration[0].recipient),
                        }],
                        "totalOriginalConsiderationItems": order_params.totalOriginalConsiderationItems.to::<i16>(),
                        "salt": format!("{}", order_params.salt),
                        "conduitKey": format!("{}", order_params.conduitKey),
                        "counter": "0"
                    }
                },
                "isPublic": true,
                "chainId": self.chain_id
            }]
        });
        println!("Order > \n {:#?}", &order);
        self.request_conn.send_text(order.to_string()).await?;
        Ok(())
    }

    pub async fn subscribe_orderbook(&mut self) -> eyre::Result<()> {
        self.last_id += 1;
        let sub_req = json!({
            "id": self.last_id,
            "jsonrpc": "2.0",
            "method": "aori_subscribeOrderbook",
            "params": []
        });
        self.feed_conn.send_text(sub_req.to_string()).await?;
        Ok(())
    }
}
