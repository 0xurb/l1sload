extern crate alloc;

use alloc::sync::Arc;
use core::marker::PhantomData;

use revm::{
    precompile::PrecompileResult,
    primitives::{Bytes, U256},
    ContextPrecompile, ContextStatefulPrecompileMut, Database, InnerEvmContext,
};
use tokio::runtime::Handle;

mod constants;
pub use constants::L1_SLOAD_ADDRESS;
use constants::L1_SLOAD_MAX_NUM_STORAGE_SLOTS;

type StorageSlots = smallvec::SmallVec<[U256; L1_SLOAD_MAX_NUM_STORAGE_SLOTS]>;

/// L1SLOAD context stateful precompile
#[non_exhaustive]
#[derive(Clone)]
pub struct L1SloadPrecompile<P, T> {
    rt_handle: Handle,
    slots: StorageSlots,
    l1_client: Arc<P>,
    _transport: PhantomData<T>,
}

impl<P, T, DB> ContextStatefulPrecompileMut<DB> for L1SloadPrecompile<P, T>
where
    P: Clone + Send + Sync + 'static,
    T: Clone + Send + Sync + 'static,
    DB: Database,
{
    fn call_mut(
        &mut self,
        bytes: &Bytes,
        gas_limit: u64,
        evmctx: &mut InnerEvmContext<DB>,
    ) -> PrecompileResult {
        todo!()
    }
}

impl<P, T> L1SloadPrecompile<P, T>
where
    P: Clone + Send + Sync + 'static,
    T: Clone + Send + Sync + 'static,
{
    /// Creates a new stateful precompile for l1sload.
    pub fn new<DB: Database>(rt_handle: Handle, l1_client: Arc<P>) -> ContextPrecompile<DB> {
        let this = Self {
            rt_handle,
            slots: StorageSlots::from_buf([U256::ZERO; 5]),
            l1_client,
            _transport: PhantomData,
        };
        ContextPrecompile::ContextStatefulMut(Box::new(this))
    }
}
