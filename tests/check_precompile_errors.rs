//! Check expected precompile errors for OOG and invalid input

use l1sload::{L1SloadPrecompile, L1_SLOAD_BASE, L1_SLOAD_PER_LOAD_BASE};
use revm::{
    db::EmptyDB,
    primitives::{Bytes, PrecompileError, PrecompileErrors, PrecompileResult},
    ContextStatefulPrecompileMut, InnerEvmContext,
};
use tokio::runtime::Handle;

mod common;
use common::l1_client;

fn init_and_call(bytes: &revm::primitives::Bytes, gas_limit: u64) -> PrecompileResult {
    let mut evmctx = InnerEvmContext::<EmptyDB>::new(EmptyDB::default());
    let mut precompile = L1SloadPrecompile::new(Handle::current(), l1_client());
    precompile.call_mut(bytes, gas_limit, &mut evmctx)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn check_precompile_errors() {
    assert!(init_and_call(
        &[&[0x00; 20][..], &[0x01; 32], &[0x02; 32]].concat().into(),
        L1_SLOAD_BASE + 2 * L1_SLOAD_PER_LOAD_BASE
    )
    .is_ok());

    assert_eq!(
        init_and_call(
            &[&[0x00; 20][..], &[0x01; 32], &[0x02; 32]].concat().into(),
            (L1_SLOAD_BASE + 2 * L1_SLOAD_PER_LOAD_BASE) - 1
        )
        .err(),
        Some(PrecompileErrors::Error(PrecompileError::OutOfGas)),
        "expects OOG"
    );

    assert_eq!(
        init_and_call(&Bytes::new(), 21_000).err(),
        Some(PrecompileErrors::Error(PrecompileError::Other(
            "invalid input".to_owned()
        ))),
        "expects invalid input"
    );

    assert_eq!(
        init_and_call(
            &[
                &[0x00; 20][..],
                &[0x01; 32],
                &[0x02; 32],
                &[0x03; 32],
                &[0x04; 32],
                &[0x05; 32],
                &[0x06; 32]
            ]
            .concat()
            .into(),
            21_000
        )
        .err(),
        Some(PrecompileErrors::Error(PrecompileError::Other(
            "invalid input".to_owned()
        ))),
        "expects invalid input"
    );
}
