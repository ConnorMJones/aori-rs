use serde_json::json;

use websockets::WebSocket;

use std::sync::Arc;

use eyre::Context;

use ethers::{
    prelude::{k256::ecdsa::SigningKey, LocalWallet, Wallet, Ws},
    providers::{Middleware, Provider},
    signers::Signer,
    types::{Signature, H256},
};

use alloy_sol_types::SolStruct;

use alloy_primitives::FixedBytes;

use aori_types::{
    constants::{MARKET_FEED_URL, REQUEST_URL},
    seaport::{OrderComponents, SEAPORT_DOMAIN},
};

pub struct AoriProvider {
    pub request_conn: WebSocket,
    pub feed_conn: WebSocket,
    pub wallet: Wallet<SigningKey>,
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

        let wallet = key.parse::<LocalWallet>()?.with_chain_id(chain_id);
        let sig: Signature = wallet.sign_message(address.as_str()).await?;
        let request_conn = WebSocket::connect(REQUEST_URL).await?;
        let feed_conn = WebSocket::connect(MARKET_FEED_URL).await?;

        Ok(Self {
            request_conn,
            feed_conn,
            wallet,
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

    pub async fn check_auth(&mut self, jwt: &str) -> eyre::Result<()> {
        self.last_id += 1;
        let auth = json!({
            "id": self.last_id,
            "jsonrpc": "2.0",
            "method": "aori_checkAuth",
            "params": [{
                "auth": jwt
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

    pub async fn make_order(&mut self, order_params: OrderComponents) -> eyre::Result<()> {
        self.last_id += 1;
        let sig: FixedBytes<32> = order_params.eip712_signing_hash(&SEAPORT_DOMAIN);
        let signed_sig: Signature = self.wallet.sign_hash(H256::from_slice(sig.as_slice()))?;
        let order = json!({
            "id": self.last_id,
            "jsonrpc": "2.0",
            "method": "aori_makeOrder",
            "params": [{
                "order": {
                    "signature": format!("0x{}", signed_sig),
                    "parameters": order_params.to_json()
                },
                "isPublic": true,
                "chainId": self.chain_id
            }]
        });
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, Address, U256};
    use aori_types::constants::{DEFAULT_CONDUIT_KEY, DEFAULT_ORDER_ADDRESS, DEFAULT_ZONE_HASH};
    use aori_types::seaport::{ConsiderationItem, ItemType, OfferItem, OrderComponents, OrderType};
    use tokio::time::{sleep, Duration};
    use websockets::Frame;

    #[tokio::test]
    async fn generate_order_sig() {
        dotenv::dotenv().ok();
        let apv = AoriProvider::new_from_env()
            .await
            .expect("Failed to create Aori Provider");
        let offer_item = OfferItem {
            itemType: ItemType::ERC20 as u8,
            token: Address::ZERO,
            identifierOrCriteria: U256::from(0),
            startAmount: U256::from(0),
            endAmount: U256::from(0),
        };
        let consider_item = ConsiderationItem {
            itemType: ItemType::ERC20 as u8,
            token: Address::ZERO,
            identifierOrCriteria: U256::from(0),
            startAmount: U256::from(0),
            endAmount: U256::from(0),
            recipient: Address::ZERO,
        };
        let order_components = OrderComponents {
            offerer: Address::ZERO,
            zone: DEFAULT_ORDER_ADDRESS,
            offer: vec![offer_item],
            consideration: vec![consider_item],
            orderType: OrderType::PARTIAL_RESTRICTED as u8,
            startTime: U256::from(1697240202),
            endTime: U256::from(1697240202),
            zoneHash: DEFAULT_ZONE_HASH.into(),
            salt: U256::from(0),
            conduitKey: DEFAULT_CONDUIT_KEY.into(),
            counter: U256::from(0),
        };

        let params_sig = order_components.eip712_signing_hash(&*SEAPORT_DOMAIN);

        /*
        https://docs.rs/ethers/latest/ethers/signers/struct.Wallet.html#method.sign_typed_data
            async fn sign_typed_data<T: Eip712 + Send + Sync>(
                &self,
                payload: &T,
            ) -> Result<Signature, Self::Error> {
                let encoded =
                    payload.encode_eip712().map_err(|e| Self::Error::Eip712Error(e.to_string()))?;

                self.sign_hash(H256::from(encoded))
            }
        https://github.com/ProjectOpenSea/seaport-js/blob/c7552e1f77528f648b1208f04d4eac910171d48c/src/constants.ts#L10
        for the type you're signing
        */

        let signed_bytes: Signature = apv.wallet.sign_message(params_sig).await.unwrap();
        let signed_slice: Signature = apv
            .wallet
            .sign_hash(H256::from_slice(params_sig.as_slice()))
            .unwrap();
        println!("0x{}", signed_bytes);
        println!("0x{}", signed_slice);
    }

    #[tokio::test]
    async fn test_connection() {
        dotenv::dotenv().ok();
        let mut apv = AoriProvider::new_from_env()
            .await
            .expect("Failed to create Aori Provider");
        apv.ping().await.unwrap();
        let response = format!("{:#?}", apv.request_conn.receive().await.unwrap());
        println!("{response:}");
    }

    #[tokio::test]
    async fn test_auth() {
        dotenv::dotenv().ok();
        let mut apv = AoriProvider::new_from_env()
            .await
            .expect("Failed to create Aori Provider");
        apv.auth_wallet().await.unwrap();
        let frame: Frame = apv.request_conn.receive().await.unwrap();

        let payload: String = match frame {
            Frame::Text { payload, .. } => Some(payload),
            _ => None,
        }
        .unwrap();
        let resp_value: serde_json::Value = serde_json::from_str(&payload).unwrap();
        println!("{:#?}", resp_value);
        let jwt = resp_value.pointer("/result/auth").unwrap().to_string();
        apv.check_auth(jwt.as_str()).await.unwrap();
        sleep(Duration::from_millis(100)).await;
        let check = format!("{:#?}", apv.request_conn.receive().await.unwrap());
        println!("jwt > {}", jwt);
        println!(" check > {check:}");
    }

    #[tokio::test]
    async fn test_make_order() {
        dotenv::dotenv().ok();
        let wallet = std::env::var("WALLET_ADDRESS")
            .context("missing WALLET_ADDRESS")
            .unwrap();
        let start_time = chrono::Utc::now().timestamp_millis();
        let end_time = chrono::Utc::now().timestamp_millis() + 1000 * 60 * 60 * 24;
        let mut apv = AoriProvider::new_from_env()
            .await
            .expect("Failed to create Aori Provider");
        let offer_item = OfferItem {
            itemType: ItemType::ERC20 as u8,
            token: address!("2715Ccea428F8c7694f7e78B2C89cb454c5F7294"),
            identifierOrCriteria: U256::from(0),
            startAmount: U256::from(1000000000000000_u128),
            endAmount: U256::from(1000000000000000_u128),
        };
        let consider_item = ConsiderationItem {
            itemType: ItemType::ERC20 as u8,
            token: address!("D3664B5e72B46eaba722aB6f43c22dBF40181954"),
            identifierOrCriteria: U256::from(0),
            startAmount: U256::from(1500000),
            endAmount: U256::from(1500000),
            recipient: Address::parse_checksummed(&wallet, None).unwrap(),
        };
        let order_params = OrderComponents {
            offerer: Address::parse_checksummed(&wallet, None).unwrap(),
            zone: DEFAULT_ORDER_ADDRESS,
            offer: vec![offer_item.clone()],
            consideration: vec![consider_item.clone()],
            orderType: OrderType::PARTIAL_RESTRICTED as u8,
            startTime: U256::from(start_time),
            endTime: U256::from(end_time),
            zoneHash: DEFAULT_ZONE_HASH.into(),
            salt: U256::from(0),
            conduitKey: DEFAULT_CONDUIT_KEY.into(),
            // totalOriginalConsiderationItems: U256::from(1),
            counter: U256::from(0),
        };

        apv.make_order(order_params).await.unwrap();

        let response = format!("{:#?}", apv.request_conn.receive().await.unwrap());
        println!("{response:}");
    }
}
