

use crate::proxy::v2::call_list::Listener;
use crate::single::TransactionTrace;

pub struct Response;

#[cfg(feature = "std")]
impl super::TraceResponseBuilder for Response {
	type Listener = Listener;
	type Response = TransactionTrace;

	fn build(listener: Listener) -> Option<TransactionTrace> {
		if let Some(entry) = listener.entries.last() {
			return Some(TransactionTrace::CallList(
				entry.into_iter().map(|(_, value)| value.clone()).collect(),
			));
		}
		None
	}
}
