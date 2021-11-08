
pub use  mrevm_core_primitives::{AccountId, Balance, Block, BlockNumber, Hash, Header, Index};
use sc_client_api::{Backend as BackendT, BlockchainEvents, KeyIterator};
use sp_api::{CallApiAt, NumberFor, ProvideRuntimeApi};
use sp_blockchain::HeaderBackend;
use sp_consensus::BlockStatus;
use sp_runtime::{
	generic::{BlockId, SignedBlock},
	traits::{BlakeTwo256, Block as BlockT},
	Justifications,
};
use sp_storage::{ChildInfo, PrefixedStorageKey, StorageData, StorageKey};
use std::sync::Arc;

/// A set of APIs that moonrabbit-like runtimes must implement.
///
/// This trait has no methods or associated type. It is a concise marker for all the trait bounds
/// that it contains.
pub trait RuntimeApiCollection:
	sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
	+ sp_api::ApiExt<Block>
	+ sp_block_builder::BlockBuilder<Block>
	+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>
	+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
	+ sp_api::Metadata<Block>
	+ sp_offchain::OffchainWorkerApi<Block>
	+ sp_session::SessionKeys<Block>
	+ fp_rpc::EthereumRuntimeRPCApi<Block>
	+  mrevm_rpc_primitives_debug::DebugRuntimeApi<Block>
	+  mrevm_rpc_primitives_txpool::TxPoolRuntimeApi<Block>
	+ nimbus_primitives::AuthorFilterAPI<Block, nimbus_primitives::NimbusId>
	+ cumulus_primitives_core::CollectCollationInfo<Block>
where
	<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

impl<Api> RuntimeApiCollection for Api
where
	Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ sp_api::ApiExt<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>
		+ pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance>
		+ sp_api::Metadata<Block>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_session::SessionKeys<Block>
		+ fp_rpc::EthereumRuntimeRPCApi<Block>
		+  mrevm_rpc_primitives_debug::DebugRuntimeApi<Block>
		+  mrevm_rpc_primitives_txpool::TxPoolRuntimeApi<Block>
		+ nimbus_primitives::AuthorFilterAPI<Block, nimbus_primitives::NimbusId>
		+ cumulus_primitives_core::CollectCollationInfo<Block>,
	<Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

/// Config that abstracts over all available client implementations.
///
/// For a concrete type there exists [`Client`].
pub trait AbstractClient<Block, Backend>:
	BlockchainEvents<Block>
	+ Sized
	+ Send
	+ Sync
	+ ProvideRuntimeApi<Block>
	+ HeaderBackend<Block>
	+ CallApiAt<Block, StateBackend = Backend::State>
where
	Block: BlockT,
	Backend: BackendT<Block>,
	Backend::State: sp_api::StateBackend<BlakeTwo256>,
	Self::Api: RuntimeApiCollection<StateBackend = Backend::State>,
{
}

impl<Block, Backend, Client> AbstractClient<Block, Backend> for Client
where
	Block: BlockT,
	Backend: BackendT<Block>,
	Backend::State: sp_api::StateBackend<BlakeTwo256>,
	Client: BlockchainEvents<Block>
		+ ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ Sized
		+ Send
		+ Sync
		+ CallApiAt<Block, StateBackend = Backend::State>,
	Client::Api: RuntimeApiCollection<StateBackend = Backend::State>,
{
}

/// Execute something with the client instance.
///
/// As there exist multiple chains inside  mrevm, like  mrevm itself, Moonbase,
/// mrevm etc, there can exist different kinds of client types. As these
/// client types differ in the generics that are being used, we can not easily
/// return them from a function. For returning them from a function there exists
/// [`Client`]. However, the problem on how to use this client instance still
/// exists. This trait "solves" it in a dirty way. It requires a type to
/// implement this trait and than the [`execute_with_client`](ExecuteWithClient:
/// :execute_with_client) function can be called with any possible client
/// instance.
///
/// In a perfect world, we could make a closure work in this way.
pub trait ExecuteWithClient {
	/// The return type when calling this instance.
	type Output;

	/// Execute whatever should be executed with the given client instance.
	fn execute_with_client<Client, Api, Backend>(self, client: Arc<Client>) -> Self::Output
	where
		<Api as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
		Backend: sc_client_api::Backend<Block>,
		Backend::State: sp_api::StateBackend<BlakeTwo256>,
		Api: crate::RuntimeApiCollection<StateBackend = Backend::State>,
		Client: AbstractClient<Block, Backend, Api = Api> + 'static;
}

/// A handle to a  mrevm client instance.
///
/// The  mrevm service supports multiple different runtimes (Moonbase,  mrevm
/// itself, etc). As each runtime has a specialized client, we need to hide them
/// behind a trait. This is this trait.
///
/// When wanting to work with the inner client, you need to use `execute_with`.
pub trait ClientHandle {
	/// Execute the given something with the client.
	fn execute_with<T: ExecuteWithClient>(&self, t: T) -> T::Output;
}

/// A client instance of  mrevm.
#[derive(Clone)]
pub enum Client {
	 mrevm(Arc<crate::FullClient< mrevm_runtime::RuntimeApi, crate:: mrevmExecutor>>),
	mrevm(Arc<crate::FullClient<mrevm_runtime::RuntimeApi, crate::mrevmExecutor>>),
	Moonbase(Arc<crate::FullClient<moonbase_runtime::RuntimeApi, crate::MoonbaseExecutor>>),
}

impl ClientHandle for Client {
	fn execute_with<T: ExecuteWithClient>(&self, t: T) -> T::Output {
		match self {
			Self:: mrevm(client) => {
				T::execute_with_client::<_, _, crate::FullBackend>(t, client.clone())
			}
			Self::mrevm(client) => {
				T::execute_with_client::<_, _, crate::FullBackend>(t, client.clone())
			}
			Self::Moonbase(client) => {
				T::execute_with_client::<_, _, crate::FullBackend>(t, client.clone())
			}
		}
	}
}

impl sc_client_api::UsageProvider<Block> for Client {
	fn usage_info(&self) -> sc_client_api::ClientInfo<Block> {
		match self {
			Self:: mrevm(client) => client.usage_info(),
			Self::mrevm(client) => client.usage_info(),
			Self::Moonbase(client) => client.usage_info(),
		}
	}
}

impl sc_client_api::BlockBackend<Block> for Client {
	fn block_body(
		&self,
		id: &BlockId<Block>,
	) -> sp_blockchain::Result<Option<Vec<<Block as BlockT>::Extrinsic>>> {
		match self {
			Self:: mrevm(client) => client.block_body(id),
			Self::mrevm(client) => client.block_body(id),
			Self::Moonbase(client) => client.block_body(id),
		}
	}

	fn block_indexed_body(
		&self,
		id: &BlockId<Block>,
	) -> sp_blockchain::Result<Option<Vec<Vec<u8>>>> {
		match self {
			Self:: mrevm(client) => client.block_indexed_body(id),
			Self::mrevm(client) => client.block_indexed_body(id),
			Self::Moonbase(client) => client.block_indexed_body(id),
		}
	}

	fn block(&self, id: &BlockId<Block>) -> sp_blockchain::Result<Option<SignedBlock<Block>>> {
		match self {
			Self:: mrevm(client) => client.block(id),
			Self::mrevm(client) => client.block(id),
			Self::Moonbase(client) => client.block(id),
		}
	}

	fn block_status(&self, id: &BlockId<Block>) -> sp_blockchain::Result<BlockStatus> {
		match self {
			Self:: mrevm(client) => client.block_status(id),
			Self::mrevm(client) => client.block_status(id),
			Self::Moonbase(client) => client.block_status(id),
		}
	}

	fn justifications(&self, id: &BlockId<Block>) -> sp_blockchain::Result<Option<Justifications>> {
		match self {
			Self:: mrevm(client) => client.justifications(id),
			Self::mrevm(client) => client.justifications(id),
			Self::Moonbase(client) => client.justifications(id),
		}
	}

	fn block_hash(
		&self,
		number: NumberFor<Block>,
	) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
		match self {
			Self:: mrevm(client) => client.block_hash(number),
			Self::mrevm(client) => client.block_hash(number),
			Self::Moonbase(client) => client.block_hash(number),
		}
	}

	fn indexed_transaction(
		&self,
		hash: &<Block as BlockT>::Hash,
	) -> sp_blockchain::Result<Option<Vec<u8>>> {
		match self {
			Self:: mrevm(client) => client.indexed_transaction(hash),
			Self::mrevm(client) => client.indexed_transaction(hash),
			Self::Moonbase(client) => client.indexed_transaction(hash),
		}
	}

	fn has_indexed_transaction(
		&self,
		hash: &<Block as BlockT>::Hash,
	) -> sp_blockchain::Result<bool> {
		match self {
			Self:: mrevm(client) => client.has_indexed_transaction(hash),
			Self::mrevm(client) => client.has_indexed_transaction(hash),
			Self::Moonbase(client) => client.has_indexed_transaction(hash),
		}
	}
}

impl sc_client_api::StorageProvider<Block, crate::FullBackend> for Client {
	fn storage(
		&self,
		id: &BlockId<Block>,
		key: &StorageKey,
	) -> sp_blockchain::Result<Option<StorageData>> {
		match self {
			Self:: mrevm(client) => client.storage(id, key),
			Self::mrevm(client) => client.storage(id, key),
			Self::Moonbase(client) => client.storage(id, key),
		}
	}

	fn storage_keys(
		&self,
		id: &BlockId<Block>,
		key_prefix: &StorageKey,
	) -> sp_blockchain::Result<Vec<StorageKey>> {
		match self {
			Self:: mrevm(client) => client.storage_keys(id, key_prefix),
			Self::mrevm(client) => client.storage_keys(id, key_prefix),
			Self::Moonbase(client) => client.storage_keys(id, key_prefix),
		}
	}

	fn storage_hash(
		&self,
		id: &BlockId<Block>,
		key: &StorageKey,
	) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
		match self {
			Self:: mrevm(client) => client.storage_hash(id, key),
			Self::mrevm(client) => client.storage_hash(id, key),
			Self::Moonbase(client) => client.storage_hash(id, key),
		}
	}

	fn storage_pairs(
		&self,
		id: &BlockId<Block>,
		key_prefix: &StorageKey,
	) -> sp_blockchain::Result<Vec<(StorageKey, StorageData)>> {
		match self {
			Self:: mrevm(client) => client.storage_pairs(id, key_prefix),
			Self::mrevm(client) => client.storage_pairs(id, key_prefix),
			Self::Moonbase(client) => client.storage_pairs(id, key_prefix),
		}
	}

	fn storage_keys_iter<'a>(
		&self,
		id: &BlockId<Block>,
		prefix: Option<&'a StorageKey>,
		start_key: Option<&StorageKey>,
	) -> sp_blockchain::Result<
		KeyIterator<'a, <crate::FullBackend as sc_client_api::Backend<Block>>::State, Block>,
	> {
		match self {
			Self:: mrevm(client) => client.storage_keys_iter(id, prefix, start_key),
			Self::mrevm(client) => client.storage_keys_iter(id, prefix, start_key),
			Self::Moonbase(client) => client.storage_keys_iter(id, prefix, start_key),
		}
	}

	fn child_storage(
		&self,
		id: &BlockId<Block>,
		child_info: &ChildInfo,
		key: &StorageKey,
	) -> sp_blockchain::Result<Option<StorageData>> {
		match self {
			Self:: mrevm(client) => client.child_storage(id, child_info, key),
			Self::mrevm(client) => client.child_storage(id, child_info, key),
			Self::Moonbase(client) => client.child_storage(id, child_info, key),
		}
	}

	fn child_storage_keys(
		&self,
		id: &BlockId<Block>,
		child_info: &ChildInfo,
		key_prefix: &StorageKey,
	) -> sp_blockchain::Result<Vec<StorageKey>> {
		match self {
			Self:: mrevm(client) => client.child_storage_keys(id, child_info, key_prefix),
			Self::mrevm(client) => client.child_storage_keys(id, child_info, key_prefix),
			Self::Moonbase(client) => client.child_storage_keys(id, child_info, key_prefix),
		}
	}

	fn child_storage_hash(
		&self,
		id: &BlockId<Block>,
		child_info: &ChildInfo,
		key: &StorageKey,
	) -> sp_blockchain::Result<Option<<Block as BlockT>::Hash>> {
		match self {
			Self:: mrevm(client) => client.child_storage_hash(id, child_info, key),
			Self::mrevm(client) => client.child_storage_hash(id, child_info, key),
			Self::Moonbase(client) => client.child_storage_hash(id, child_info, key),
		}
	}

	fn max_key_changes_range(
		&self,
		first: NumberFor<Block>,
		last: BlockId<Block>,
	) -> sp_blockchain::Result<Option<(NumberFor<Block>, BlockId<Block>)>> {
		match self {
			Self:: mrevm(client) => client.max_key_changes_range(first, last),
			Self::mrevm(client) => client.max_key_changes_range(first, last),
			Self::Moonbase(client) => client.max_key_changes_range(first, last),
		}
	}

	fn key_changes(
		&self,
		first: NumberFor<Block>,
		last: BlockId<Block>,
		storage_key: Option<&PrefixedStorageKey>,
		key: &StorageKey,
	) -> sp_blockchain::Result<Vec<(NumberFor<Block>, u32)>> {
		match self {
			Self:: mrevm(client) => client.key_changes(first, last, storage_key, key),
			Self::mrevm(client) => client.key_changes(first, last, storage_key, key),
			Self::Moonbase(client) => client.key_changes(first, last, storage_key, key),
		}
	}
}

impl sp_blockchain::HeaderBackend<Block> for Client {
	fn header(&self, id: BlockId<Block>) -> sp_blockchain::Result<Option<Header>> {
		match self {
			Self:: mrevm(client) => client.header(&id),
			Self::mrevm(client) => client.header(&id),
			Self::Moonbase(client) => client.header(&id),
		}
	}

	fn info(&self) -> sp_blockchain::Info<Block> {
		match self {
			Self:: mrevm(client) => client.info(),
			Self::mrevm(client) => client.info(),
			Self::Moonbase(client) => client.info(),
		}
	}

	fn status(&self, id: BlockId<Block>) -> sp_blockchain::Result<sp_blockchain::BlockStatus> {
		match self {
			Self:: mrevm(client) => client.status(id),
			Self::mrevm(client) => client.status(id),
			Self::Moonbase(client) => client.status(id),
		}
	}

	fn number(&self, hash: Hash) -> sp_blockchain::Result<Option<BlockNumber>> {
		match self {
			Self:: mrevm(client) => client.number(hash),
			Self::mrevm(client) => client.number(hash),
			Self::Moonbase(client) => client.number(hash),
		}
	}

	fn hash(&self, number: BlockNumber) -> sp_blockchain::Result<Option<Hash>> {
		match self {
			Self:: mrevm(client) => client.hash(number),
			Self::mrevm(client) => client.hash(number),
			Self::Moonbase(client) => client.hash(number),
		}
	}
}
