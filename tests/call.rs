//! Call L1Sload precompile test

use std::sync::Arc;

use alloy::{
    primitives::{aliases::U112, U32},
    sol_types::SolValue,
    transports::http::{Client, Http},
};
use l1sload::{L1SloadPrecompile, L1_SLOAD_ADDRESS};
use revm::{
    db::{CacheDB, EmptyDB},
    precompile::u64_to_address,
    primitives::{address, Bytes, ExecutionResult, Output, TxKind, U256},
    ContextPrecompiles, Evm, L1_BLOCK_CONTRACT,
};
use tokio::runtime::Handle;

mod common;
use common::l1_client;

/// Definition of reserves, observed in that test by slot storage get.
#[derive(Debug, PartialEq)]
struct UniV2Reserves {
    reserve0: U112,
    reserve1: U112,
    block_timestamp_last: U32,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn should_call() -> eyre::Result<()> {
    // === ETH target slot === //
    // choose slot of storage that you would like to transact with
    let slot = U256::from(8);
    // ETH/USDT pair on Uniswap V2
    let pool_address = address!("0d4a11d5EEaaC28EC3F61d100daF4d40471f1852");
    // Input for L1_SLOAD precompile
    let precompile_input: Bytes = [pool_address.as_slice(), slot.to_be_bytes::<32>().as_ref()]
        .concat()
        .into();

    let expected_reserves = UniV2Reserves {
        reserve0: U112::from(25417578056206466070741_u128),
        reserve1: U112::from(66119976721112_u64),
        block_timestamp_last: U32::from(1729037363),
    };

    debug_assert_eq!(precompile_input.len(), 52);

    // === EVM build === //
    let mut cache_db = CacheDB::new(EmptyDB::default());
    // insert same storage slot0 as was on Optimism [`L1_BLOCK_CONTRACT`] at block height 20974450
    cache_db.insert_account_storage(
        L1_BLOCK_CONTRACT,
        U256::ZERO,
        U256::from(31895110171864498523091241842_u128), // at block 20974450
    )?;

    // TODO - make facade for register to simply call
    // `.append_handler_register(L1SloadPrecompile::register)`
    let mut evm: Evm<_, _> = Evm::builder()
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
        .modify_tx_env(|tx| {
            tx.caller = Default::default();
            // account you want to transact with
            tx.transact_to = TxKind::Call(u64_to_address(L1_SLOAD_ADDRESS));
            // data of the transaction
            tx.data = precompile_input;
            // transaction value in wei
            tx.value = U256::from(0);
            // TODO - must be op transaction here, after revm update
            // *enveloped_tx = Some(Bytes::default());
        })
        .build();

    // execute transaction without writing to the DB
    let ref_tx = evm.transact().unwrap();
    // select ExecutionResult struct
    let result = ref_tx.result;

    // unpack output call enum into raw bytes
    let value = match result {
        ExecutionResult::Success {
            output: Output::Call(value),
            ..
        } => value,
        _ => panic!("Execution failed: {result:?}"),
    };

    // decode bytes to reserves + ts via alloy's abi decode
    let data = Vec::<U256>::abi_decode(value.as_ref(), true)?;
    let bytes = Bytes::from(data[0].to_be_bytes_vec());

    let reserve0 = &bytes[18..32];
    let reserve1 = &bytes[4..18];
    let timestamp = &bytes[0..4];

    let got = UniV2Reserves {
        reserve0: U112::from_be_bytes(<[u8; 14]>::try_from(reserve0)?),
        reserve1: U112::from_be_bytes(<[u8; 14]>::try_from(reserve1)?),
        block_timestamp_last: U32::from_be_bytes(<[u8; 4]>::try_from(timestamp)?),
    };
    assert_eq!(got, expected_reserves);
    dbg!(got);

    Ok(())
}
