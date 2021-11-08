

pub mod blockscout;
pub mod raw;
pub mod trace_filter;

use crate::proxy::v2::Listener;
#[cfg(feature = "std")]
use serde::Serialize;

#[cfg(feature = "std")]
pub trait TraceResponseBuilder {
	type Listener: Listener;
	type Response: Serialize;

	fn build(listener: Self::Listener) -> Option<Self::Response>;
}
