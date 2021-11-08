

use bip39::{Language, Mnemonic, MnemonicType, Seed};
use primitive_types::{H160, H256};
use secp256k1::{PublicKey, SecretKey};
use sp_runtime::traits::IdentifyAccount;
use structopt::StructOpt;
use tiny_hderive::bip32::ExtendedPrivKey;

#[derive(Debug, StructOpt)]
pub struct GenerateAccountKey {
	/// Generate 12 words mnemonic instead of 24
	#[structopt(long, short = "w")]
	w12: bool,

	/// Specify the mnemonic
	#[structopt(long, short = "m")]
	mnemonic: Option<String>,

	/// The account index to use in the derivation path
	#[structopt(long = "account-index", short = "a")]
	account_index: Option<u32>,
}

impl GenerateAccountKey {
	pub fn run(&self) {
		// Retrieve the mnemonic from the args or generate random ones
		let mnemonic = if let Some(phrase) = &self.mnemonic {
			Mnemonic::from_phrase(phrase, Language::English).unwrap()
		} else {
			match self.w12 {
				true => Mnemonic::new(MnemonicType::Words12, Language::English),
				false => Mnemonic::new(MnemonicType::Words24, Language::English),
			}
		};

		// Retrieves the seed from the mnemonic
		let seed = Seed::new(&mnemonic, "");

		// Generate the derivation path from the account-index
		let derivation_path = format!("m/44'/60'/0'/0/{}", self.account_index.unwrap_or(0));

		// Derives the private key from
		let ext = ExtendedPrivKey::derive(seed.as_bytes(), derivation_path.as_str()).unwrap();
		let private_key = SecretKey::parse_slice(&ext.secret()).unwrap();

		// Retrieves the public key
		let public_key = PublicKey::from_secret_key(&private_key);

		// Convert into H160 address.
		let signer: account::EthereumSigner = public_key.into();
		let address: H160 = signer.into_account();

		println!("Address:      {:?}", address);
		println!("Mnemonic:     {}", mnemonic.phrase());
		println!("Private Key:  {:?}", H256::from(private_key.serialize()));
		println!("Path:         {}", derivation_path);
	}
}
