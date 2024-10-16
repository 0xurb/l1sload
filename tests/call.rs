//! Call L1Sload precompile test

use std::sync::Arc;

use alloy::{
    network::Ethereum,
    providers::{builder, RootProvider},
    transports::http::{Client, Http},
};
use l1sload::{L1SloadPrecompile, L1_SLOAD_ADDRESS};
use revm::{
    db::{CacheDB, EmptyDB},
    precompile::u64_to_address,
    ContextPrecompiles, Evm,
};
use tokio::runtime::Handle;

fn l1_client() -> Arc<RootProvider<Http<Client>>> {
    let rpc_url = "https://mainnet.infura.io/v3/c60b0bb42f8a4c6481ecd229eddaca27"
        .parse()
        .unwrap();
    Arc::new(builder::<Ethereum>().on_http(rpc_url))
}

#[tokio::test]
async fn should_call() -> eyre::Result<()> {
    let cache_db = CacheDB::new(EmptyDB::default());

    let evm: Evm<_, _> = Evm::builder()
        .with_db(cache_db)
        // add additional precompiles
        .append_handler_register(|handler| {
            let spec_id = handler.cfg.spec_id;

            // install the precompiles
            handler.pre_execution.load_precompiles = Arc::new(move || {
                let mut ctx_precompiles = ContextPrecompiles::new(
                    revm::precompile::PrecompileSpecId::from_spec_id(spec_id),
                );
                ctx_precompiles.extend(std::iter::once((
                    u64_to_address(L1_SLOAD_ADDRESS),
                    L1SloadPrecompile::<_, Http<Client>>::new(Handle::current(), l1_client()),
                )));
                ctx_precompiles
            });
        })
        .build();

    Ok(())
}
