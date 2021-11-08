

#![cfg_attr(not(feature = "std"), no_std)]

use crowdloan_rewards_precompiles::CrowdloanRewardsWrapper;
use evm::{executor::PrecompileOutput, Context, ExitError};
use frame_support::dispatch::{Dispatchable, GetDispatchInfo, PostDispatchInfo};
use pallet_evm::{Precompile, PrecompileSet};
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_dispatch::Dispatch;
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_sha3fips::Sha3FIPS256;
use pallet_evm_precompile_simple::{ECRecover, ECRecoverPublicKey, Identity, Ripemd160, Sha256};
use jurisdiction_staking_precompiles::JurisdictionStakingWrapper;
use parity_scale_codec::Decode;
use sp_core::H160;
use sp_std::convert::TryFrom;
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;

use frame_support::traits::Currency;
type BalanceOf<Runtime> = <<Runtime as jurisdiction_staking::Config>::Currency as Currency<
	<Runtime as frame_system::Config>::AccountId,
>>::Balance;

type RewardBalanceOf<Runtime> =
	<<Runtime as pallet_crowdloan_rewards::Config>::RewardCurrency as Currency<
		<Runtime as frame_system::Config>::AccountId,
	>>::Balance;

/// The PrecompileSet installed in the  mrevm runtime.
/// We include the nine Istanbul precompiles
/// (https://github.com/ethereum/go-ethereum/blob/3c46f557/core/vm/contracts.go#L69)
/// as well as a special precompile for dispatching Substrate extrinsics
#[derive(Debug, Clone, Copy)]
pub struct  mrevmPrecompiles<R>(PhantomData<R>);

impl<R: frame_system::Config>  mrevmPrecompiles<R>
where
	R::AccountId: From<H160>,
{
	/// Return all addresses that contain precompiles. This can be used to populate dummy code
	/// under the precompile.
	pub fn used_addresses() -> impl Iterator<Item = R::AccountId> {
		sp_std::vec![1, 2, 3, 4, 5, 6, 7, 8, 1024, 1025, 1026, 2048, 2049]
			.into_iter()
			.map(|x| hash(x).into())
	}
}

/// The following distribution has been decided for the precompiles
/// 0-1023: Ethereum Mainnet Precompiles
/// 1024-2047 Precompiles that are not in Ethereum Mainnet but are neither  mrevm specific
/// 2048-4095  mrevm specific precompiles
impl<R> PrecompileSet for  mrevmPrecompiles<R>
where
	R::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo + Decode,
	<R::Call as Dispatchable>::Origin: From<Option<R::AccountId>>,
	R: jurisdiction_staking::Config + pallet_evm::Config + pallet_crowdloan_rewards::Config,
	R::AccountId: From<H160>,
	BalanceOf<R>: Debug + precompile_utils::EvmData,
	RewardBalanceOf<R>: TryFrom<sp_core::U256> + Debug,
	R::Call: From<jurisdiction_staking::Call<R>> + From<pallet_crowdloan_rewards::Call<R>>,
{
	fn execute(
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
	) -> Option<Result<PrecompileOutput, ExitError>> {
		match address {
			// Ethereum precompiles :
			a if a == hash(1) => Some(ECRecover::execute(input, target_gas, context)),
			a if a == hash(2) => Some(Sha256::execute(input, target_gas, context)),
			a if a == hash(3) => Some(Ripemd160::execute(input, target_gas, context)),
			a if a == hash(4) => Some(Identity::execute(input, target_gas, context)),
			a if a == hash(5) => Some(Modexp::execute(input, target_gas, context)),
			a if a == hash(6) => Some(Bn128Add::execute(input, target_gas, context)),
			a if a == hash(7) => Some(Bn128Mul::execute(input, target_gas, context)),
			a if a == hash(8) => Some(Bn128Pairing::execute(input, target_gas, context)),
			// Non- mrevm specific nor Ethereum precompiles :
			a if a == hash(1024) => Some(Sha3FIPS256::execute(input, target_gas, context)),
			a if a == hash(1025) => Some(Dispatch::<R>::execute(input, target_gas, context)),
			a if a == hash(1026) => Some(ECRecoverPublicKey::execute(input, target_gas, context)),
			//  mrevm specific precompiles :
			a if a == hash(2048) => Some(JurisdictionStakingWrapper::<R>::execute(
				input, target_gas, context,
			)),
			a if a == hash(2049) => Some(CrowdloanRewardsWrapper::<R>::execute(
				input, target_gas, context,
			)),
			_ => None,
		}
	}
}

fn hash(a: u64) -> H160 {
	H160::from_low_u64_be(a)
}
