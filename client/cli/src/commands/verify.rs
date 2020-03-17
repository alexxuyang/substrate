// Copyright 2018-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! implementation of the `verify` subcommand

use crate::{RuntimeAdapter, read_message, decode_hex, read_uri, error, SharedParams, VersionInfo};
use sp_core::{Pair, Public, crypto::Ss58Codec};
use structopt::StructOpt;
use sc_service::{Configuration, ChainSpec};

/// The `verify` command
#[derive(Debug, StructOpt, Clone)]
#[structopt(
	name = "verify",
	about = "Verify a signature for a message, provided on STDIN, with a given (public or secret) key"
)]
pub struct VerifyCmd {
	/// Signature, hex-encoded.
	#[structopt(long)]
	sig: String,

	/// The public or secret key URI.
	/// If the value is a file, the file content is used as URI.
	/// If not given, you will be prompted for the URI.
	#[structopt(long)]
	uri: Option<String>,

	/// Message to verify, if not provided you will be prompted to
	/// pass the message via STDIN
	#[structopt(long)]
	message: Option<String>,

	/// The message on STDIN is hex-encoded data
	#[structopt(long)]
	hex: bool,

	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub shared_params: SharedParams,
}

impl VerifyCmd {
	/// Run the command
	pub fn run<RA: RuntimeAdapter>(self) -> error::Result<()> {
		let message = read_message(self.message, self.hex)?;
		let mut signature = <RA::Pair as Pair>::Signature::default();
		let sig_data = decode_hex(self.sig)?;

		if sig_data.len() != signature.as_ref().len() {
			return Err(error::Error::Other(format!(
				"signature has an invalid length. read {} bytes, expected {} bytes",
				sig_data.len(),
				signature.as_ref().len(),
			)));
		}

		signature.as_mut().copy_from_slice(&sig_data);
		let uri = read_uri(self.uri)?;
		let uri = if uri.starts_with("0x") {
			&uri[2..]
		} else {
			&uri
		};

		let pubkey = if let Ok(pubkey_vec) = hex::decode(uri) {
			RA::Public::from_slice(pubkey_vec.as_slice())
		} else {
			RA::Public::from_string(uri)
				.ok()
				.expect("Invalid URI; expecting either a secret URI or a public URI.")
		};

		if <RA::Pair as Pair>::verify(&signature, &message, &pubkey) {
			println!("Signature verifies correctly.");
		} else {
			return Err(error::Error::Other("Signature invalid.".into()))
		}

		Ok(())
	}

	/// Update and prepare a `Configuration` with command line parameters
	pub fn update_config<F>(
		&self,
		mut config: &mut Configuration,
		spec_factory: F,
		version: &VersionInfo,
	) -> error::Result<()> where
		F: FnOnce(&str) -> Result<Box<dyn ChainSpec>, String>,
	{
		self.shared_params.update_config(&mut config, spec_factory, version)?;
		config.use_in_memory_keystore()?;

		Ok(())
	}
}