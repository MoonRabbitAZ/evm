

//!  mrevm EVM tracing Integration Tests

mod common;

#[cfg(test)]
#[cfg(feature = "evm-tracing")]
mod tests {
	use super::common::*;

	use pallet_evm::AddressMapping;
	use sha3::{Digest, Keccak256};
	use sp_core::{H160, H256};

	use  mrevm_rpc_primitives_debug::runtime_decl_for_DebugRuntimeApi::DebugRuntimeApi;
	use std::str::FromStr;

	#[test]
	fn debug_runtime_api_trace_transaction() {
		let alith = <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(
			H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
				.expect("internal H160 is valid; qed"),
		);
		ExtBuilder::default()
			.with_balances(vec![
				(alith, 2_000 * AAA),
				(AccountId::from(ALICE), 2_000 * AAA),
				(AccountId::from(BOB), 1_000 * AAA),
			])
			.build()
			.execute_with(|| {
				let non_eth_uxt = UncheckedExtrinsic::new_unsigned(
					pallet_balances::Call::<Runtime>::transfer(AccountId::from(BOB), 1 * AAA)
						.into(),
				);
				let transaction = ethereum_transaction(VALID_ETH_TX);
				let eth_uxt = unchecked_eth_tx(VALID_ETH_TX);
				assert!(Runtime::trace_transaction(
					vec![non_eth_uxt.clone(), eth_uxt, non_eth_uxt.clone()],
					&transaction
				)
				.is_ok());
			});
	}

	#[test]
	fn debug_runtime_api_trace_block() {
		let alith = <Runtime as pallet_evm::Config>::AddressMapping::into_account_id(
			H160::from_str("6be02d1d3665660d22ff9624b7be0551ee1ac91b")
				.expect("internal H160 is valid; qed"),
		);
		ExtBuilder::default()
			.with_balances(vec![
				(alith, 2_000 * AAA),
				(AccountId::from(ALICE), 2_000 * AAA),
				(AccountId::from(BOB), 1_000 * AAA),
			])
			.build()
			.execute_with(|| {
				let non_eth_uxt = UncheckedExtrinsic::new_unsigned(
					pallet_balances::Call::<Runtime>::transfer(AccountId::from(BOB), 1 * AAA)
						.into(),
				);
				let eth_uxt = unchecked_eth_tx(VALID_ETH_TX);
				let eth_tx = ethereum_transaction(VALID_ETH_TX);
				let eth_extrinsic_hash =
					H256::from_slice(Keccak256::digest(&rlp::encode(&eth_tx)).as_slice());
				assert!(Runtime::trace_block(
					vec![non_eth_uxt.clone(), eth_uxt.clone(), non_eth_uxt, eth_uxt],
					vec![eth_extrinsic_hash, eth_extrinsic_hash]
				)
				.is_ok());
			});
	}
}
