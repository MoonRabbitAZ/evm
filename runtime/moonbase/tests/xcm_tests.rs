

//! Moonbase Runtime Integration Tests

mod xcm_mock;
use frame_support::assert_ok;
use xcm_mock::jurisdiction;
use xcm_mock::relay_chain;
use xcm_mock::*;

use xcm::v1::{
	Junction::{self, AccountId32, AccountKey20, PalletInstance, Jurisdiction},
	Junctions::*,
	MultiLocation, NetworkId,
};
use xcm_simulator::TestExt;

// Send a relay asset (like DOT) to a jurisdiction A
#[test]
fn receive_relay_asset_from_relay() {
	MockNet::reset();

	let source_location = jurisdiction::AssetType::Xcm(MultiLocation::parent());
	let source_id: jurisdiction::AssetId = source_location.clone().into();
	let asset_metadata = jurisdiction::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};
	// register relay asset in jurisdiction A
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			jurisdiction::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			jurisdiction::Origin::root(),
			source_id,
			0u128
		));
	});

	// Actually send relay asset to jurisdiction
	let dest: MultiLocation = AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Jurisdiction(1).into().into()),
			Box::new(dest.clone().into()),
			Box::new((Here, 123).into()),
			0,
			123,
		));
	});

	// Verify that jurisdiction received the asset
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});
}

// Send relay asset (like DOT) back from Jurisdiction A to relaychain
#[test]
fn send_relay_asset_to_relay() {
	MockNet::reset();

	let source_location = jurisdiction::AssetType::Xcm(MultiLocation::parent());
	let source_id: jurisdiction::AssetId = source_location.clone().into();

	let asset_metadata = jurisdiction::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// First send relay chain asset to Jurisdiction like in previous test
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			jurisdiction::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			jurisdiction::Origin::root(),
			source_id,
			0u128
		));
	});

	let dest: MultiLocation = Junction::AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Jurisdiction(1).into().into()),
			Box::new(dest.clone().into()),
			Box::new((Here, 123).into()),
			0,
			123,
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});

	let mut balance_before_sending = 0;
	Relay::execute_with(|| {
		balance_before_sending = RelayBalances::free_balance(&RELAYALICE);
	});

	let dest = MultiLocation {
		parents: 1,
		interior: X1(AccountId32 {
			network: NetworkId::Any,
			id: RELAYALICE.into(),
		}),
	};

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			jurisdiction::Origin::signed(PARAALICE.into()),
			jurisdiction::CurrencyId::OtherReserve(source_id),
			123,
			Box::new(dest),
			40000
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});

	Relay::execute_with(|| {
		// free execution,x	 full amount received
		assert!(RelayBalances::free_balance(&RELAYALICE) > balance_before_sending);
	});
}

#[test]
fn send_relay_asset_to_para_b() {
	MockNet::reset();

	let source_location = jurisdiction::AssetType::Xcm(MultiLocation::parent());
	let source_id: jurisdiction::AssetId = source_location.clone().into();

	let asset_metadata = jurisdiction::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			jurisdiction::Origin::root(),
			source_location.clone(),
			asset_metadata.clone(),
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			jurisdiction::Origin::root(),
			source_id,
			0u128
		));
	});

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			jurisdiction::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			jurisdiction::Origin::root(),
			source_id,
			0u128
		));
	});

	let dest: MultiLocation = Junction::AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Jurisdiction(1).into().into()),
			Box::new(dest.clone().into()),
			Box::new((Here, 123).into()),
			0,
			123,
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 123);
	});

	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Jurisdiction(2),
			AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE.into(),
			},
		),
	};

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			jurisdiction::Origin::signed(PARAALICE.into()),
			jurisdiction::CurrencyId::OtherReserve(source_id),
			100,
			Box::new(dest),
			40000
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 23);
	});

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});
}

#[test]
fn send_para_a_asset_to_para_b() {
	MockNet::reset();

	let para_a_balances = MultiLocation::new(1, X2(Jurisdiction(1), PalletInstance(1u8)));
	let source_location = jurisdiction::AssetType::Xcm(para_a_balances);
	let source_id: jurisdiction::AssetId = source_location.clone().into();

	let asset_metadata = jurisdiction::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			jurisdiction::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			jurisdiction::Origin::root(),
			source_id,
			0u128
		));
	});

	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Jurisdiction(2),
			AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE.into(),
			},
		),
	};

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			jurisdiction::Origin::signed(PARAALICE.into()),
			jurisdiction::CurrencyId::SelfReserve,
			100,
			Box::new(dest),
			800000
		));
	});
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});
}

#[test]
fn send_para_a_asset_from_para_b_to_para_c() {
	MockNet::reset();

	let para_a_balances = MultiLocation::new(1, X2(Jurisdiction(1), PalletInstance(1u8)));
	let source_location = jurisdiction::AssetType::Xcm(para_a_balances);
	let source_id: jurisdiction::AssetId = source_location.clone().into();

	let asset_metadata = jurisdiction::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			jurisdiction::Origin::root(),
			source_location.clone(),
			asset_metadata.clone(),
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			jurisdiction::Origin::root(),
			source_id,
			0u128
		));
	});

	ParaC::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			jurisdiction::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			jurisdiction::Origin::root(),
			source_id,
			0u128
		));
	});

	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Jurisdiction(2),
			AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE.into(),
			},
		),
	};
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			jurisdiction::Origin::signed(PARAALICE.into()),
			jurisdiction::CurrencyId::SelfReserve,
			100,
			Box::new(dest),
			800000
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});

	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Jurisdiction(3),
			AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE.into(),
			},
		),
	};

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			jurisdiction::Origin::signed(PARAALICE.into()),
			jurisdiction::CurrencyId::OtherReserve(source_id),
			100,
			Box::new(dest),
			800000
		));
	});

	ParaC::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});
}

#[test]
fn send_para_a_asset_to_para_b_and_back_to_para_a() {
	MockNet::reset();

	let para_a_balances = MultiLocation::new(1, X2(Jurisdiction(1), PalletInstance(1u8)));
	let source_location = jurisdiction::AssetType::Xcm(para_a_balances);
	let source_id: jurisdiction::AssetId = source_location.clone().into();

	let asset_metadata = jurisdiction::AssetMetadata {
		name: b"ParaAToken".to_vec(),
		symbol: b"ParaA".to_vec(),
		decimals: 18,
	};

	ParaB::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			jurisdiction::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			jurisdiction::Origin::root(),
			source_id,
			0u128
		));
	});

	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Jurisdiction(2),
			AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE.into(),
			},
		),
	};
	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			jurisdiction::Origin::signed(PARAALICE.into()),
			jurisdiction::CurrencyId::SelfReserve,
			100,
			Box::new(dest),
			4000
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE - 100
		);
	});

	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 100);
	});

	let dest = MultiLocation {
		parents: 1,
		interior: X2(
			Jurisdiction(1),
			AccountKey20 {
				network: NetworkId::Any,
				key: PARAALICE.into(),
			},
		),
	};
	ParaB::execute_with(|| {
		// free execution, full amount received
		assert_ok!(XTokens::transfer(
			jurisdiction::Origin::signed(PARAALICE.into()),
			jurisdiction::CurrencyId::OtherReserve(source_id),
			100,
			Box::new(dest),
			4000
		));
	});

	ParaA::execute_with(|| {
		// free execution, full amount received
		assert_eq!(
			ParaBalances::free_balance(&PARAALICE.into()),
			INITIAL_BALANCE
		);
	});
}

#[test]
fn receive_relay_asset_with_trader() {
	MockNet::reset();

	let source_location = jurisdiction::AssetType::Xcm(MultiLocation::parent());
	let source_id: jurisdiction::AssetId = source_location.clone().into();

	let asset_metadata = jurisdiction::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	// This time we are gonna put a rather high number of units per second
	// we know later we will divide by 1e12
	// Lets put 1e6 as units per second
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			jurisdiction::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			jurisdiction::Origin::root(),
			source_id,
			1_000_000u128
		));
	});

	let dest: MultiLocation = Junction::AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();
	// We are sending 100 tokens from relay.
	// If we set the dest weight to be 1e7, we know the buy_execution will spend 1e7*1e6/1e12 = 10
	// Therefore with no refund, we should receive 10 tokens less
	// Native trader fails for this, and we use the asset trader
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Jurisdiction(1).into().into()),
			Box::new(dest.clone().into()),
			Box::new((Here, 100).into()),
			0,
			10_000_000u64,
		));
	});

	ParaA::execute_with(|| {
		// non-free execution, not full amount received
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 90);
	});
}

#[test]
fn error_when_not_paying_enough() {
	MockNet::reset();

	let source_location = jurisdiction::AssetType::Xcm(MultiLocation::parent());
	let source_id: jurisdiction::AssetId = source_location.clone().into();

	let asset_metadata = jurisdiction::AssetMetadata {
		name: b"RelayToken".to_vec(),
		symbol: b"Relay".to_vec(),
		decimals: 12,
	};

	let dest: MultiLocation = Junction::AccountKey20 {
		network: NetworkId::Any,
		key: PARAALICE,
	}
	.into();
	// This time we are gonna put a rather high number of units per second
	// we know later we will divide by 1e12
	// Lets put 1e6 as units per second
	ParaA::execute_with(|| {
		assert_ok!(AssetManager::register_asset(
			jurisdiction::Origin::root(),
			source_location,
			asset_metadata,
			1u128,
		));
		assert_ok!(AssetManager::set_asset_units_per_second(
			jurisdiction::Origin::root(),
			source_id,
			1_000_000u128
		));
	});

	// We are sending 100 tokens from relay.
	// If we set the dest weight to be 1e7, we know the buy_execution will spend 1e7*1e6/1e12 = 10
	// Therefore with no refund, we should receive 10 tokens less
	Relay::execute_with(|| {
		assert_ok!(RelayChainPalletXcm::reserve_transfer_assets(
			relay_chain::Origin::signed(RELAYALICE),
			Box::new(Jurisdiction(1).into().into()),
			Box::new(dest.clone().into()),
			Box::new((Here, 5).into()),
			0,
			10_000_000u64,
		));
	});

	ParaA::execute_with(|| {
		// amount not received as it is not paying enough
		assert_eq!(Assets::balance(source_id, &PARAALICE.into()), 0);
	});
}
