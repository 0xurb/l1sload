use revm::interpreter::gas::COLD_SLOAD_COST;

/// Precompile address (tentative).
pub(crate) const L1_SLOAD_ADDRESS: u64 = 0x101;
/// Base gas fee for l1sload operation (tentative).
pub(crate) const L1_SLOAD_BASE: u64 = 2_000;
/// `eth_getStorageAt` credits per call in value of gas (tentative).
pub(crate) const ETH_GET_STORAGE_AT_RPC_CALL_COST: u64 = 50;
/// Per load gas fee for l1sload operation.
pub(crate) const L1_SLOAD_PER_LOAD_BASE: u64 = COLD_SLOAD_COST + ETH_GET_STORAGE_AT_RPC_CALL_COST;
/// Max number of storage slots for l1sload operation (tentative).
pub(crate) const L1_SLOAD_MAX_NUM_STORAGE_SLOTS: usize = 5;
