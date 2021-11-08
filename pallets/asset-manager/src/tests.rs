

// Tests for AssetManager Pallet
use crate::*;
use mock::*;

use frame_support::{assert_noop, assert_ok};

#[test]
fn registering_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
		));

		assert_eq!(
			AssetManager::asset_id_type(1).unwrap(),
			MockAssetType::MockAsset(1)
		);
		expect_events(vec![crate::Event::AssetRegistered(
			1,
			MockAssetType::MockAsset(1),
			0u32,
		)])
	});
}

#[test]
fn test_asset_exists_error() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
		));

		assert_eq!(
			AssetManager::asset_id_type(1).unwrap(),
			MockAssetType::MockAsset(1)
		);
		assert_noop!(
			AssetManager::register_asset(
				Origin::root(),
				MockAssetType::MockAsset(1),
				0u32.into(),
				1u32.into(),
			),
			Error::<Test>::AssetAlreadyExists
		);
	});
}

#[test]
fn test_root_can_change_units_per_second() {
	new_test_ext().execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			Origin::root(),
			MockAssetType::MockAsset(1),
			0u32.into(),
			1u32.into(),
		));

		assert_ok!(AssetManager::set_asset_units_per_second(
			Origin::root(),
			1,
			200u128.into()
		));

		assert_eq!(AssetManager::asset_id_units_per_second(1).unwrap(), 200);

		expect_events(vec![
			crate::Event::AssetRegistered(1, MockAssetType::MockAsset(1), 0),
			crate::Event::UnitsPerSecondChanged(1, 200),
		])
	});
}

#[test]
fn test_asset_id_non_existent_error() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			AssetManager::set_asset_units_per_second(Origin::root(), 1, 200u128.into()),
			Error::<Test>::AssetDoesNotExist
		);
	});
}
