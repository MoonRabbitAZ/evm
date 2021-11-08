

use crate::proxy::v2::raw::Listener;
use crate::single::TransactionTrace;

pub struct Response;

#[cfg(feature = "std")]
impl super::TraceResponseBuilder for Response {
	type Listener = Listener;
	type Response = TransactionTrace;

	fn build(listener: Listener) -> Option<TransactionTrace> {
		Some(TransactionTrace::Raw {
			step_logs: listener.step_logs,
			gas: listener.final_gas.into(),
			return_value: listener.return_value,
		})
	}
}
