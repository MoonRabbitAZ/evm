

//! Unit testing
use crate::mock::{events, Call as OuterCall, ExtBuilder, Origin, Test};
use crate::{Call, Error, Event};
use frame_support::{assert_noop, assert_ok, dispatch::Dispatchable};
use sp_runtime::DispatchError;

#[test]
fn can_remark_during_normal_operation() {
	ExtBuilder::default().build().execute_with(|| {
		let call: OuterCall = frame_system::Call::remark(vec![]).into();
		assert_ok!(call.dispatch(Origin::signed(1)));
	})
}

#[test]
fn cannot_remark_during_maintenance_mode() {
	ExtBuilder::default()
		.with_maintenance_mode(true)
		.build()
		.execute_with(|| {
			let call: OuterCall = frame_system::Call::remark(vec![]).into();
			assert_noop!(call.dispatch(Origin::signed(1)), DispatchError::BadOrigin);
		})
}

#[test]
fn can_enter_maintenance_mode() {
	ExtBuilder::default().build().execute_with(|| {
		let call: OuterCall = Call::enter_maintenance_mode().into();
		assert_ok!(call.dispatch(Origin::root()));

		assert_eq!(events(), vec![Event::EnteredMaintenanceMode,]);
	})
}

#[test]
fn cannot_enter_maintenance_mode_from_wrong_origin() {
	ExtBuilder::default()
		.with_maintenance_mode(true)
		.build()
		.execute_with(|| {
			let call: OuterCall = Call::enter_maintenance_mode().into();
			assert_noop!(call.dispatch(Origin::signed(1)), DispatchError::BadOrigin);
		})
}

#[test]
fn cannot_enter_maintenance_mode_when_already_in_it() {
	ExtBuilder::default()
		.with_maintenance_mode(true)
		.build()
		.execute_with(|| {
			let call: OuterCall = Call::enter_maintenance_mode().into();
			assert_noop!(
				call.dispatch(Origin::root()),
				Error::<Test>::AlreadyInMaintenanceMode
			);
		})
}

#[test]
fn can_resume_normal_operation() {
	ExtBuilder::default()
		.with_maintenance_mode(true)
		.build()
		.execute_with(|| {
			let call: OuterCall = Call::resume_normal_operation().into();
			assert_ok!(call.dispatch(Origin::root()));

			assert_eq!(events(), vec![Event::NormalOperationResumed,]);
		})
}

#[test]
fn cannot_resume_normal_operation_from_wrong_origin() {
	ExtBuilder::default()
		.with_maintenance_mode(true)
		.build()
		.execute_with(|| {
			let call: OuterCall = Call::resume_normal_operation().into();
			assert_noop!(call.dispatch(Origin::signed(1)), DispatchError::BadOrigin);
		})
}

#[test]
fn cannot_resume_normal_operation_while_already_operating_normally() {
	ExtBuilder::default().build().execute_with(|| {
		let call: OuterCall = Call::resume_normal_operation().into();
		assert_noop!(
			call.dispatch(Origin::root()),
			Error::<Test>::NotInMaintenanceMode
		);
	})
}
