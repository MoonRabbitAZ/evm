

//! Substrate EVM tracing.
//!
//! The purpose of this crate is enable tracing the EVM opcode execution and will be used by
//! both Dapp developers - to get a granular view on their transactions - and indexers to access
//! the EVM callstack (internal transactions).
//!
//! Proxies EVM messages to the host functions.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use  mrevm_rpc_primitives_debug::proxy::types::{EvmEvent, GasometerEvent, RuntimeEvent};

use evm::tracing::{using as evm_using, EventListener as EvmListener};
use evm_gasometer::tracing::{using as gasometer_using, EventListener as GasometerListener};
use evm_runtime::tracing::{using as runtime_using, EventListener as RuntimeListener};
use sp_std::{cell::RefCell, rc::Rc};

struct ListenerProxy<T>(pub Rc<RefCell<T>>);
impl<T: GasometerListener> GasometerListener for ListenerProxy<T> {
	fn event(&mut self, event: evm_gasometer::tracing::Event) {
		self.0.borrow_mut().event(event);
	}
}

impl<T: RuntimeListener> RuntimeListener for ListenerProxy<T> {
	fn event(&mut self, event: evm_runtime::tracing::Event) {
		self.0.borrow_mut().event(event);
	}
}

impl<T: EvmListener> EvmListener for ListenerProxy<T> {
	fn event(&mut self, event: evm::tracing::Event) {
		self.0.borrow_mut().event(event);
	}
}

pub struct EvmTracer;
impl EvmTracer {
	pub fn new() -> Self {
		Self
	}
	/// Setup event listeners and execute provided closure.
	///
	/// Consume the tracer and return it alongside the return value of
	/// the closure.
	pub fn trace<R, F: FnOnce() -> R>(self, f: F) {
		evm::tracing::enable_tracing(true);
		evm_gasometer::tracing::enable_tracing(true);
		evm_runtime::tracing::enable_tracing(true);

		let wrapped = Rc::new(RefCell::new(self));

		let mut gasometer = ListenerProxy(Rc::clone(&wrapped));
		let mut runtime = ListenerProxy(Rc::clone(&wrapped));
		let mut evm = ListenerProxy(Rc::clone(&wrapped));

		// Each line wraps the previous `f` into a `using` call.
		// Listening to new events results in adding one new line.
		// Order is irrelevant when registering listeners.
		let f = || runtime_using(&mut runtime, f);
		let f = || gasometer_using(&mut gasometer, f);
		let f = || evm_using(&mut evm, f);
		f();

		evm::tracing::enable_tracing(false);
		evm_gasometer::tracing::enable_tracing(false);
		evm_runtime::tracing::enable_tracing(false);
	}

	pub fn emit_new() {
		 mrevm_primitives_ext:: mrevm_ext::call_list_new();
	}
}

impl EvmListener for EvmTracer {
	/// Proxies `evm::tracing::Event` to the host.
	fn event(&mut self, event: evm::tracing::Event) {
		let event: EvmEvent = event.into();
		let message = event.encode();
		 mrevm_primitives_ext:: mrevm_ext::evm_event(message);
	}
}

impl GasometerListener for EvmTracer {
	/// Proxies `evm_gasometer::tracing::Event` to the host.
	fn event(&mut self, event: evm_gasometer::tracing::Event) {
		let event: GasometerEvent = event.into();
		let message = event.encode();
		 mrevm_primitives_ext:: mrevm_ext::gasometer_event(message);
	}
}

impl RuntimeListener for EvmTracer {
	/// Proxies `evm_runtime::tracing::Event` to the host.
	fn event(&mut self, event: evm_runtime::tracing::Event) {
		let event: RuntimeEvent = event.into();
		let message = event.encode();
		 mrevm_primitives_ext:: mrevm_ext::runtime_event(message);
	}
}
