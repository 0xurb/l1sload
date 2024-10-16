use std::sync::Arc;

use alloy::{
    network::Ethereum,
    providers::{builder, RootProvider},
    transports::http::{Client, Http},
};

pub fn l1_client() -> Arc<RootProvider<Http<Client>>> {
    let rpc_url = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
        .parse()
        .unwrap();
    Arc::new(builder::<Ethereum>().on_http(rpc_url))
}
