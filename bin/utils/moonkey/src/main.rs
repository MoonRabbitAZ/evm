

//! Generate an Ethereum account.

use cli_opt::account_key::GenerateAccountKey;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(author = "moonrabbit")]
struct Opt {
	#[structopt(flatten)]
	cmd: GenerateAccountKey,
}

impl Opt {
	fn run(&self) {
		self.cmd.run()
	}
}

fn main() {
	// Parses the options
	let cmd = Opt::from_args();
	cmd.run();
}
