

#![cfg_attr(not(feature = "std"), no_std)]
// These clippy lints are disabled because the macro-generated code triggers them.
#![allow(clippy::unnecessary_mut_passed)]
#![allow(clippy::too_many_arguments)]

use codec::{Decode, Encode};
use ethereum::Transaction;
use sp_runtime::traits::Block as BlockT;
use sp_std::vec::Vec;

#[derive(Eq, PartialEq, Clone, Encode, Decode, sp_runtime::RuntimeDebug)]
pub struct TxPoolResponse {
	pub ready: Vec<Transaction>,
	pub future: Vec<Transaction>,
}

sp_api::decl_runtime_apis! {
	pub trait TxPoolRuntimeApi {
		fn extrinsic_filter(
			xt_ready: Vec<<Block as BlockT>::Extrinsic>,
			xt_future: Vec<<Block as BlockT>::Extrinsic>,
		) -> TxPoolResponse;
	}
}
