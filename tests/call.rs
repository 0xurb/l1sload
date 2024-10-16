//! Call L1Sload precompile test

use std::sync::Arc;

use alloy::transports::http::{Client, Http};
use l1sload::{L1SloadPrecompile, L1_SLOAD_ADDRESS};
use revm::{
    db::{CacheDB, EmptyDB},
    precompile::u64_to_address,
    ContextPrecompiles, Evm,
};
use tokio::runtime::Handle;

mod common;
use common::l1_client;

#[tokio::test]
async fn should_call() -> eyre::Result<()> {
    let cache_db = CacheDB::new(EmptyDB::default());

    // TODO - make facade for register to simply call
    // `.append_handler_register(L1SloadPrecompile::register)`
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
                    L1SloadPrecompile::<_, Http<Client>>::new_precompile(
                        Handle::current(),
                        l1_client(),
                    ),
                )));
                ctx_precompiles
            });
        })
        .build();

    Ok(())
}
