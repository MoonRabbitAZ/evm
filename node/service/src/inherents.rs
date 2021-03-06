

//! This module houses a MOCK inherent data provider for the validataion
//! data inherent. This mock provider provides stub data that does not represent anything "real"
//! about the external world, but can pass the runtime's checks. This is useful in testing
//! for example, running the --dev service without a relay chain backbone.

use cumulus_primitives_core::PersistedValidationData;
use cumulus_primitives_jurisdiction_inherent::{JurisdictionInherentData, INHERENT_IDENTIFIER};
use sp_inherents::{InherentData, InherentDataProvider};

use cumulus_test_relay_sproof_builder::RelayStateSproofBuilder;

/// Inherent data provider that supplies mocked validation data.
///
/// This is useful when running a node that is not actually backed by any relay chain.
/// For example when running a local node, or running integration tests.
///
/// We mock a relay chain block number as follows:
/// relay_block_number = offset + relay_blocks_per_para_block * current_para_block
/// To simulate a jurisdiction that starts in relay block 1000 and gets a block in every other relay
/// block, use 1000 and 2
pub struct MockValidationDataInherentDataProvider {
	/// The current block number of the local block chain (the jurisdiction)
	pub current_para_block: u32,
	/// The relay block in which this jurisdiction appeared to start. This will be the relay block
	/// number in para block #P1
	pub relay_offset: u32,
	/// The number of relay blocks that elapses between each parablock. Probably set this to 1 or 2
	/// to simulate optimistic or realistic relay chain behavior.
	pub relay_blocks_per_para_block: u32,
}

#[async_trait::async_trait]
impl InherentDataProvider for MockValidationDataInherentDataProvider {
	fn provide_inherent_data(
		&self,
		inherent_data: &mut InherentData,
	) -> Result<(), sp_inherents::Error> {
		// Use the "sproof" (spoof proof) builder to build valid mock state root and proof.
		let (relay_storage_root, proof) =
			RelayStateSproofBuilder::default().into_state_root_and_proof();

		// Calculate the mocked relay block based on the current para block
		let relay_parent_number =
			self.relay_offset + self.relay_blocks_per_para_block * self.current_para_block;

		let data = JurisdictionInherentData {
			validation_data: PersistedValidationData {
				parent_head: Default::default(),
				relay_parent_storage_root: relay_storage_root,
				relay_parent_number,
				max_pov_size: Default::default(),
			},
			downward_messages: Default::default(),
			horizontal_messages: Default::default(),
			relay_chain_state: proof,
		};

		inherent_data.put_data(INHERENT_IDENTIFIER, &data)
	}

	// Copied from the real implementation
	async fn try_handle_error(
		&self,
		_: &sp_inherents::InherentIdentifier,
		_: &[u8],
	) -> Option<Result<(), sp_inherents::Error>> {
		None
	}
}
