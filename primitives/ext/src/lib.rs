

//! Environmental-aware externalities for EVM tracing in Wasm runtime. This enables
//! capturing the - potentially large - trace output data in the host and keep
//! a low memory footprint in `--execution=wasm`.
//!
//! - The original trace Runtime Api call is wrapped `using` environmental (thread local).
//! - Arguments are scale-encoded known types in the host.
//! - Host functions will decode the input and emit an event `with` environmental.

#![cfg_attr(not(feature = "std"), no_std)]
use sp_runtime_interface::runtime_interface;

use codec::Decode;
use sp_std::vec::Vec;

use ethereum_types::U256;
use  mrevm_rpc_primitives_debug::{
	proxy::types::{EvmEvent, GasometerEvent, RuntimeEvent},
	proxy::v1::Event as EventV1,
	proxy::v2::Event as EventV2,
	single::{Call, RawStepLog},
};

#[runtime_interface]
pub trait  mrevmExt {
	// Old format to be deprecated.
	fn raw_step(&mut self, data: Vec<u8>) {
		let data: RawStepLog = Decode::decode(&mut &data[..]).unwrap();
		EventV1::RawStep(data).emit();
	}
	fn raw_gas(&mut self, data: Vec<u8>) {
		let data: U256 = Decode::decode(&mut &data[..]).unwrap();
		EventV1::RawGas(data).emit();
	}
	fn raw_return_value(&mut self, data: Vec<u8>) {
		EventV1::RawReturnValue(data).emit();
	}
	fn call_list_entry(&mut self, index: u32, value: Vec<u8>) {
		let value: Call = Decode::decode(&mut &value[..]).unwrap();
		EventV1::CallListEntry((index, value)).emit();
	}
	fn call_list_new(&mut self) {
		EventV1::CallListNew().emit();
	}
	// New design, proxy events.
	/// An `Evm` event proxied by the  mrevm runtime to this host function.
	/// evm ->  mrevm_runtime -> host.
	fn evm_event(&mut self, event: Vec<u8>) {
		let event: EvmEvent = Decode::decode(&mut &event[..]).unwrap();
		EventV2::Evm(event).emit();
	}
	/// A `Gasometer` event proxied by the  mrevm runtime to this host function.
	/// evm_gasometer ->  mrevm_runtime -> host.
	fn gasometer_event(&mut self, event: Vec<u8>) {
		let event: GasometerEvent = Decode::decode(&mut &event[..]).unwrap();
		EventV2::Gasometer(event).emit();
	}
	/// A `Runtime` event proxied by the  mrevm runtime to this host function.
	/// evm_runtime ->  mrevm_runtime -> host.
	fn runtime_event(&mut self, event: Vec<u8>) {
		let event: RuntimeEvent = Decode::decode(&mut &event[..]).unwrap();
		EventV2::Runtime(event).emit();
	}
	/// An event to create a new CallList (currently a new transaction when tracing a block).
	#[version(2)]
	fn call_list_new(&mut self) {
		EventV2::CallListNew().emit();
	}
}
