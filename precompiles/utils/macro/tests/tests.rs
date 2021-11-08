

use sha3::{Digest, Keccak256};

#[precompile_utils_macro::generate_function_selector]
pub enum Action {
	Toto = "toto()",
	Tata = "tata()",
}

#[test]
fn tests() {
	assert_eq!(
		&(Action::Toto as u32).to_be_bytes()[..],
		&Keccak256::digest(b"toto()")[0..4],
	);
	assert_eq!(
		&(Action::Tata as u32).to_be_bytes()[..],
		&Keccak256::digest(b"tata()")[0..4],
	);
	assert_ne!(Action::Toto as u32, Action::Tata as u32);
}
