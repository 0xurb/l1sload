use std::{marker::PhantomData, sync::Arc};

use revm::{precompile::PrecompileResult, primitives::{Bytes, U256}, ContextPrecompile, ContextStatefulPrecompileMut, EvmWiring, InnerEvmContext};
use tokio::runtime::Handle;

mod constants;
use constants::L1_SLOAD_MAX_NUM_STORAGE_SLOTS;

type StorageSlots = smallvec::SmallVec<[U256; L1_SLOAD_MAX_NUM_STORAGE_SLOTS]>;

/// L1SLOAD context stateful precompile
#[non_exhaustive]
#[derive(Clone)]
struct L1SloadPrecompile<P, T> {
    rt_handle: Handle,
    slots: StorageSlots,
    l1_client: Arc<P>,
    _transport: PhantomData<T>,
}

impl<P, T, W> ContextStatefulPrecompileMut<W> for L1SloadPrecompile<P, T>
where
	P: Clone + Send + Sync + 'static,
	T: Clone + Send + Sync + 'static,
	W: EvmWiring,
{
    fn call_mut(
        &mut self,
        bytes: &Bytes,
        gas_limit: u64,
        evmctx: &mut InnerEvmContext<W>,
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
    fn new<W: EvmWiring>(rt_handle: Handle, l1_client: Arc<P>) -> ContextPrecompile<W> {
        let this = Self {
            rt_handle,
            slots: StorageSlots::from_buf([U256::ZERO; 5]),
            l1_client,
            _transport: PhantomData,
        };
        ContextPrecompile::ContextStatefulMut(Box::new(this))
    }
}
