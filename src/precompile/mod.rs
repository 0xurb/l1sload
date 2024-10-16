extern crate alloc;

use alloc::sync::Arc;
use alloy::{primitives::U64, providers::Provider, sol_types::SolValue, transports::Transport};
use core::marker::PhantomData;
use futures::{StreamExt, TryStreamExt};

use revm::{
    precompile::{PrecompileError, PrecompileErrors, PrecompileResult},
    primitives::{Address, Bytes, PrecompileOutput, U256},
    ContextPrecompile, ContextStatefulPrecompileMut, Database, InnerEvmContext, L1_BLOCK_CONTRACT,
};
use tokio::runtime::Handle;

pub mod constants;
use constants::{L1_SLOAD_BASE, L1_SLOAD_MAX_NUM_STORAGE_SLOTS, L1_SLOAD_PER_LOAD_BASE};

type StorageSlots = smallvec::SmallVec<[U256; L1_SLOAD_MAX_NUM_STORAGE_SLOTS]>;

/// L1SLOAD context stateful precompile
#[non_exhaustive]
#[derive(Debug)]
pub struct L1SloadPrecompile<P, T> {
    rt_handle: Handle,
    slots: StorageSlots,
    l1_client: Arc<P>,
    _transport: PhantomData<T>,
}

impl<P, T> Clone for L1SloadPrecompile<P, T> {
    fn clone(&self) -> Self {
        Self {
            rt_handle: self.rt_handle.clone(),
            slots: self.slots.clone(),
            l1_client: Arc::clone(&self.l1_client),
            _transport: PhantomData,
        }
    }
}

impl<P, T, DB> ContextStatefulPrecompileMut<DB> for L1SloadPrecompile<P, T>
where
    P: Provider<T> + 'static,
    T: Transport + Clone,
    DB: Database,
    DB::Error: core::fmt::Debug,
{
    fn call_mut(
        &mut self,
        bytes: &Bytes,
        gas_limit: u64,
        evmctx: &mut InnerEvmContext<DB>,
    ) -> PrecompileResult {
        let input_len = bytes.len();
        let num_storage_slots = input_len / 32;

        // check that input is correct
        if num_storage_slots == 0
            || num_storage_slots > L1_SLOAD_MAX_NUM_STORAGE_SLOTS
            || input_len != 20 + 32 * num_storage_slots
        {
            return Err(format_other_precompile_err("invalid input"));
        }

        // calculate used gas and check limit
        let gas_used = gas_used(num_storage_slots);
        if gas_used > gas_limit {
            return Err(PrecompileError::OutOfGas.into());
        }

        // load the latest L1 block number **known by the L2** system
        // if storage returns error - latest L1 block will be used.
        let at_block = match evmctx.db.storage(L1_BLOCK_CONTRACT, U256::ZERO) {
            Ok(l1_block_slot0) => Some(U64::from_be_slice(
                l1_block_slot0.to_be_bytes::<32>()[U256::BYTES - U64::BYTES..U256::BYTES].as_ref(),
            )),
            Err(err) => {
                tracing::error!(err=?err, "on get `L1_BLOCK_CONTRACT` storage");
                None
            }
        };

        // get L1 address from input
        let l1_address = Address::from_slice(&bytes[..20]);

        // set observed slots to state
        (0..num_storage_slots)
            .for_each(|i| self.slots[i] = U256::from_be_slice(&bytes[20 + 32 * i..52 + 32 * i]));

        // get storage at given slots
        let out = self.get_storages_at(l1_address, at_block)?;

        PrecompileResult::Ok(PrecompileOutput::new(gas_used, out))
    }
}

impl<P, T> L1SloadPrecompile<P, T>
where
    P: Provider<T> + 'static,
    T: Transport + Clone,
{
    pub fn new(rt_handle: Handle, l1_client: Arc<P>) -> Self {
        Self {
            rt_handle,
            slots: StorageSlots::from_buf([U256::ZERO; 5]),
            l1_client,
            _transport: PhantomData,
        }
    }

    /// Creates a new stateful precompile for l1sload.
    pub fn new_precompile<DB: Database>(
        rt_handle: Handle,
        l1_client: Arc<P>,
    ) -> ContextPrecompile<DB>
    where
        DB::Error: core::fmt::Debug,
    {
        let this = Self::new(rt_handle, l1_client);
        ContextPrecompile::ContextStatefulMut(Box::new(this))
    }

    /// Retrieves storage slots by calling `eth_getStorageAt` via L1 RPC
    ///
    /// If `at_block` is [`Option::is_some`], than latest **known by L2** L1 block is used
    /// else latest block is used.
    #[inline]
    #[track_caller]
    fn get_storages_at(
        &mut self,
        l1_address: Address,
        at_block: Option<U64>,
    ) -> Result<Bytes, PrecompileErrors> {
        let mut slots = std::mem::take(&mut self.slots);
        tokio::task::block_in_place(move || {
            self.rt_handle.block_on({
                let l1_client = Arc::clone(&self.l1_client);
                async move {
                    let res: Vec<U256> = futures::stream::iter(slots.drain(..))
                        .map(|key| {
                            let client = Arc::clone(&l1_client);
                            async move {
                                client
                                    .get_storage_at(l1_address, key)
                                    .block_id(at_block.map(Into::into).unwrap_or_default())
                                    .await
                            }
                        })
                        .buffered(L1_SLOAD_MAX_NUM_STORAGE_SLOTS)
                        .try_collect()
                        .await
                        .map_err(format_other_precompile_err)?;

                    Ok::<_, PrecompileErrors>(res.abi_encode().into())
                }
            })
        })
    }
}

/// Calculates used gas
fn gas_used(num_storage_slots: usize) -> u64 {
    L1_SLOAD_BASE + L1_SLOAD_PER_LOAD_BASE * num_storage_slots as u64
}

/// Simple format [`PrecompileError::Other`]
fn format_other_precompile_err(err: impl core::fmt::Display) -> PrecompileErrors {
    PrecompileErrors::Error(PrecompileError::other(err.to_string()))
}
