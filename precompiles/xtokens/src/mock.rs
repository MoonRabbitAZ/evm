

//! Test utilities
use super::*;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	traits::{EnsureOrigin, Everything, OriginTrait, PalletInfo as PalletInfoTrait},
	weights::Weight,
};

use frame_support::{construct_runtime, parameter_types};

use pallet_evm::{AddressMapping, EnsureAddressNever, EnsureAddressRoot, PrecompileSet};
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_io;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use xcm::v1::{
	Error as XcmError,
	Junction::{AccountKey20, GeneralIndex, PalletInstance, Jurisdiction},
	Junctions, MultiAsset, MultiLocation, NetworkId, Result as XcmResult, SendXcm, Xcm,
};

use xcm_builder::{AllowUnpaidExecutionFrom, FixedWeightBounds};

use xcm_executor::{
	traits::{InvertLocation, TransactAsset, WeightTrader},
	Assets, XcmExecutor,
};

pub type AccountId = TestAccount;
pub type Balance = u128;
pub type BlockNumber = u64;
pub const PRECOMPILE_ADDRESS: u64 = 1;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Evm: pallet_evm::{Pallet, Call, Storage, Event<T>},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Xtokens: orml_xtokens::{Pallet, Call, Storage, Event<T>},
		moonrabbitXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
	}
);

// FRom https://github.com/moonrabbit/ mrevm/pull/518. Merge to common once is merged
#[derive(
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Clone,
	Copy,
	Encode,
	Decode,
	Debug,
	MaxEncodedLen,
	Serialize,
	Deserialize,
	derive_more::Display,
)]
pub enum TestAccount {
	Alice,
	Bob,
	Charlie,
	AssetId(u128),
	SelfReserve,
	Bogus,
	Precompile,
}

impl Default for TestAccount {
	fn default() -> Self {
		Self::Bogus
	}
}

impl AddressMapping<TestAccount> for TestAccount {
	fn into_account_id(h160_account: H160) -> TestAccount {
		match h160_account {
			a if a == H160::repeat_byte(0xAA) => Self::Alice,
			a if a == H160::repeat_byte(0xBB) => Self::Bob,
			a if a == H160::repeat_byte(0xCC) => Self::Charlie,
			a if a == H160::repeat_byte(0xDD) => Self::SelfReserve,
			a if a == H160::from_low_u64_be(PRECOMPILE_ADDRESS) => Self::Precompile,
			_ => {
				let mut data = [0u8; 16];
				let (prefix_part, id_part) = h160_account.as_fixed_bytes().split_at(4);
				if prefix_part == &[255u8; 4] {
					data.copy_from_slice(id_part);

					return Self::AssetId(u128::from_be_bytes(data));
				}
				Self::Bogus
			}
		}
	}
}

impl From<H160> for TestAccount {
	fn from(x: H160) -> TestAccount {
		TestAccount::into_account_id(x)
	}
}

impl From<TestAccount> for H160 {
	fn from(value: TestAccount) -> H160 {
		match value {
			TestAccount::Alice => H160::repeat_byte(0xAA),
			TestAccount::Bob => H160::repeat_byte(0xBB),
			TestAccount::Charlie => H160::repeat_byte(0xCC),
			TestAccount::Precompile => H160::from_low_u64_be(PRECOMPILE_ADDRESS),
			TestAccount::SelfReserve => H160::repeat_byte(0xDD),
			TestAccount::AssetId(asset_id) => {
				let mut data = [0u8; 20];
				let id_as_bytes = asset_id.to_be_bytes();
				data[0..4].copy_from_slice(&[255u8; 4]);
				data[4..20].copy_from_slice(&id_as_bytes);
				H160::from_slice(&data)
			}
			TestAccount::Bogus => Default::default(),
		}
	}
}

pub type AssetId = u128;

parameter_types! {
	pub JurisdictionId: cumulus_primitives_core::ParaId = 100.into();
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}
impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type DbWeight = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = TestAccount;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type BlockWeights = ();
	type BlockLength = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}
parameter_types! {
	pub const ExistentialDeposit: u128 = 0;
}
impl pallet_balances::Config for Test {
	type MaxReserves = ();
	type ReserveIdentifier = ();
	type MaxLocks = ();
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

// These parameters dont matter much as this will only be called by root with the forced arguments
// No deposit is substracted with those methods
parameter_types! {
	pub const AssetDeposit: Balance = 0;
	pub const ApprovalDeposit: Balance = 0;
	pub const AssetsStringLimit: u32 = 50;
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
}

pub struct TestPrecompiles<R>(PhantomData<R>);

impl<R> PrecompileSet for TestPrecompiles<R>
where
	R: orml_xtokens::Config,
	R: pallet_evm::Config,
	R::AccountId: From<H160>,
	R::Call: Dispatchable<PostInfo = PostDispatchInfo> + GetDispatchInfo,
	R::Call: From<orml_xtokens::Call<R>>,
	<R::Call as Dispatchable>::Origin: From<Option<R::AccountId>>,
	XBalanceOf<R>: TryFrom<U256> + Into<U256> + EvmData,
	R: AccountIdToCurrencyId<R::AccountId, CurrencyIdOf<R>>,
{
	fn execute(
		address: H160,
		input: &[u8],
		target_gas: Option<u64>,
		context: &Context,
	) -> Option<Result<PrecompileOutput, ExitError>> {
		match address {
			a if a == precompile_address() => {
				Some(XtokensWrapper::<R>::execute(input, target_gas, context))
			}
			_ => None,
		}
	}
}

pub fn precompile_address() -> H160 {
	H160::from_low_u64_be(1)
}

pub type Precompiles = TestPrecompiles<Test>;

impl pallet_evm::Config for Test {
	type FeeCalculator = ();
	type GasWeightMapping = ();
	type CallOrigin = EnsureAddressRoot<TestAccount>;
	type WithdrawOrigin = EnsureAddressNever<TestAccount>;
	type AddressMapping = TestAccount;
	type Currency = Balances;
	type Event = Event;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type Precompiles = Precompiles;
	type ChainId = ();
	type OnChargeTransaction = ();
	type BlockGasLimit = ();
	type BlockHashMapping = pallet_evm::SubstrateBlockHashMapping<Self>;
	type FindAuthor = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}
impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}
pub struct ConvertOriginToLocal;
impl<Origin: OriginTrait> EnsureOrigin<Origin> for ConvertOriginToLocal {
	type Success = MultiLocation;

	fn try_origin(_: Origin) -> Result<MultiLocation, Origin> {
		Ok(MultiLocation::here())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn successful_origin() -> Origin {
		Origin::root()
	}
}

pub struct DoNothingRouter;
impl SendXcm for DoNothingRouter {
	fn send_xcm(_dest: MultiLocation, _msg: Xcm<()>) -> XcmResult {
		Ok(())
	}
}

pub type Barrier = AllowUnpaidExecutionFrom<Everything>;

pub struct DummyAssetTransactor;
impl TransactAsset for DummyAssetTransactor {
	fn deposit_asset(_what: &MultiAsset, _who: &MultiLocation) -> XcmResult {
		Ok(())
	}

	fn withdraw_asset(_what: &MultiAsset, _who: &MultiLocation) -> Result<Assets, XcmError> {
		Ok(Assets::default())
	}
}

pub struct DummyWeightTrader;
impl WeightTrader for DummyWeightTrader {
	fn new() -> Self {
		DummyWeightTrader
	}

	fn buy_weight(&mut self, _weight: Weight, _payment: Assets) -> Result<Assets, XcmError> {
		Ok(Assets::default())
	}
}

pub struct InvertNothing;
impl InvertLocation for InvertNothing {
	fn invert_location(_: &MultiLocation) -> MultiLocation {
		MultiLocation::here()
	}
}

impl pallet_xcm::Config for Test {
	// The config types here are entirely configurable, since the only one that is sorely needed
	// is `XcmExecutor`, which will be used in unit tests located in xcm-executor.
	type Event = Event;
	type ExecuteXcmOrigin = ConvertOriginToLocal;
	type LocationInverter = InvertNothing;
	type SendXcmOrigin = ConvertOriginToLocal;
	type Weigher = xcm_builder::FixedWeightBounds<BaseXcmWeight, Call>;
	type XcmRouter = DoNothingRouter;
	type XcmExecuteFilter = frame_support::traits::Everything;
	type XcmExecutor = xcm_executor::XcmExecutor<XcmConfig>;
	type XcmTeleportFilter = frame_support::traits::Everything;
	type XcmReserveTransferFilter = frame_support::traits::Everything;
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type Call = Call;
	type XcmSender = DoNothingRouter;
	type AssetTransactor = DummyAssetTransactor;
	type OriginConverter = pallet_xcm::XcmPassthrough<Origin>;
	type IsReserve = ();
	type IsTeleporter = ();
	type LocationInverter = InvertNothing;
	type Barrier = Barrier;
	type Weigher = FixedWeightBounds<BaseXcmWeight, Call>;
	type Trader = DummyWeightTrader;
	type ResponseHandler = ();
	type SubscriptionService = ();
}
#[derive(Clone, Eq, Debug, PartialEq, Ord, PartialOrd, Encode, Decode)]
pub enum CurrencyId {
	SelfReserve,
	OtherReserve(AssetId),
}

impl Into<Option<CurrencyId>> for TestAccount {
	fn into(self) -> Option<CurrencyId> {
		match self {
			TestAccount::SelfReserve => Some(CurrencyId::SelfReserve),
			TestAccount::AssetId(asset_id) => Some(CurrencyId::OtherReserve(asset_id)),
			_ => None,
		}
	}
}

// Implement the trait, where we convert AccountId to AssetID
impl AccountIdToCurrencyId<AccountId, CurrencyId> for Test {
	/// The way to convert an account to assetId is by ensuring that the prefix is 0XFFFFFFFF
	/// and by taking the lowest 128 bits as the assetId
	fn account_to_currency_id(account: AccountId) -> Option<CurrencyId> {
		match account {
			TestAccount::AssetId(asset_id) => Some(CurrencyId::OtherReserve(asset_id)),
			TestAccount::SelfReserve => Some(CurrencyId::SelfReserve),
			_ => None,
		}
	}
}

pub struct CurrencyIdToMultiLocation;

impl sp_runtime::traits::Convert<CurrencyId, Option<MultiLocation>> for CurrencyIdToMultiLocation {
	fn convert(currency: CurrencyId) -> Option<MultiLocation> {
		match currency {
			CurrencyId::SelfReserve => {
				let multi: MultiLocation = SelfReserve::get();
				Some(multi)
			}
			// To distinguish between relay and others, specially for reserve asset
			CurrencyId::OtherReserve(asset) => {
				if asset == 0 {
					Some(MultiLocation::parent())
				} else {
					Some(MultiLocation::new(
						1,
						Junctions::X2(Jurisdiction(2), GeneralIndex(asset)),
					))
				}
			}
		}
	}
}

pub struct AccountIdToMultiLocation;
impl sp_runtime::traits::Convert<TestAccount, MultiLocation> for AccountIdToMultiLocation {
	fn convert(account: TestAccount) -> MultiLocation {
		let as_h160: H160 = account.into();
		MultiLocation::new(
			1,
			Junctions::X1(AccountKey20 {
				network: NetworkId::Any,
				key: as_h160.as_fixed_bytes().clone(),
			}),
		)
	}
}

parameter_types! {
	pub Ancestry: MultiLocation = Jurisdiction(JurisdictionId::get().into()).into();

	pub const BaseXcmWeight: Weight = 1000;
	pub const RelayNetwork: NetworkId = NetworkId::moonrabbit;

	pub SelfLocation: MultiLocation = (1, Junctions::X1(Jurisdiction(JurisdictionId::get().into()))).into();

	pub SelfReserve: MultiLocation = (
		1,
		Junctions::X2(
			Jurisdiction(JurisdictionId::get().into()),
			PalletInstance(<Test as frame_system::Config>::PalletInfo::index::<Balances>().unwrap() as u8)
		)).into();
}

impl orml_xtokens::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type CurrencyId = CurrencyId;
	type AccountIdToMultiLocation = AccountIdToMultiLocation;
	type CurrencyIdConvert = CurrencyIdToMultiLocation;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type SelfLocation = SelfLocation;
	type Weigher = xcm_builder::FixedWeightBounds<BaseXcmWeight, Call>;
	type BaseXcmWeight = BaseXcmWeight;
	type LocationInverter = InvertNothing;
}

pub(crate) struct ExtBuilder {
	// endowed accounts with balances
	balances: Vec<(AccountId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> ExtBuilder {
		ExtBuilder { balances: vec![] }
	}
}

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}
	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.expect("Frame system builds valid default genesis config");

		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances,
		}
		.assimilate_storage(&mut t)
		.expect("Pallet balances storage can be assimilated");

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub(crate) fn events() -> Vec<Event> {
	System::events()
		.into_iter()
		.map(|r| r.event)
		.collect::<Vec<_>>()
}

// Helper function to give a simple evm context suitable for tests.
// We can remove this once https://github.com/rust-blockchain/evm/pull/35
// is in our dependency graph.
pub fn evm_test_context() -> evm::Context {
	evm::Context {
		address: Default::default(),
		caller: Default::default(),
		apparent_value: From::from(0),
	}
}
