// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! State test transaction deserialization.

use uint::Uint;
use bytes::Bytes;
use hash::{Address, H256};
use maybe::MaybeEmpty;

/// State test transaction deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Transaction {
	/// Transaction data.
	pub data: Bytes,
	/// Gas limit.
	#[serde(rename="gasLimit")]
	pub gas_limit: Uint,
	/// Gas price.
	#[serde(rename="gasPrice")]
	pub gas_price: Uint,
	/// Nonce.
	pub nonce: Uint,
	/// Secret key.
	#[serde(rename="secretKey")]
	pub secret: Option<H256>,
	/// To.
	pub to: MaybeEmpty<Address>,
	/// Value.
	pub value: Uint,
	/// IOLITE field for metadata for json spec
	pub metadata: Bytes,
	/// IOLITE field for metadataLimit for json spec
	pub metadataLimit: Uint,
	/// IOLITE field for isOld for json spec
	pub isOld: bool,
}

#[cfg(test)]
mod tests {
	use serde_json;
	use state::Transaction;

	#[test]
	fn transaction_deserialization() {
		let s = r#"{
			"data" : "",
			"gasLimit" : "0x2dc6c0",
			"gasPrice" : "0x01",
			"nonce" : "0x00",
			"secretKey" : "45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8",
			"to" : "1000000000000000000000000000000000000000",
			"value" : "0x00",
			"metadata": "",
			"metadataLimit": "0x00",
			"isOld": "false"
		}"#;
		let _deserialized: Transaction = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}
}
