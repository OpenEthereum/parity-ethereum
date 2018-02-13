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

//! Spec deserialization.

use std::io::Read;
use serde_json;

use spec::{Params, Genesis, Engine, State, Error};

/// Spec deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Spec {
	/// Spec name.
	pub name: String,
	/// Special fork name.
	#[serde(rename="dataDir")]
	pub data_dir: Option<String>,
	/// Engine.
	pub engine: Engine,
	/// Spec params.
	pub params: Params,
	/// Genesis header.
	pub genesis: Genesis,
	/// Genesis state.
	pub accounts: State,
	/// Boot nodes.
	pub nodes: Option<Vec<String>>,
}

impl Spec {
	/// Loads test from json.
	pub fn load<R>(reader: R) -> Result<Self, Error> where R: Read {
		let spec: Self = serde_json::from_reader(reader)?;

		Self::validate(&spec)?;

		Ok(spec)
	}

	fn validate(spec: &Self) -> Result<(), Error> {
		let divisor: u64 = spec.params.gas_limit_bound_divisor.into();

		if divisor == 0 {
			return Err(Error::ZeroValueDivisor)
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use serde_json;
	use spec::spec::{Spec, Error};

	#[test]
	fn spec_deserialization() {
		let s = r#"{
	"name": "Morden",
	"dataDir": "morden",
	"engine": {
		"Ethash": {
			"params": {
				"minimumDifficulty": "0x020000",
				"difficultyBoundDivisor": "0x0800",
				"durationLimit": "0x0d",
				"homesteadTransition" : "0x",
				"daoHardforkTransition": "0xffffffffffffffff",
				"daoHardforkBeneficiary": "0x0000000000000000000000000000000000000000",
				"daoHardforkAccounts": []
			}
		}
	},
	"params": {
		"accountStartNonce": "0x0100000",
		"homesteadTransition": "0x789b0",
		"maximumExtraDataSize": "0x20",
		"minGasLimit": "0x1388",
		"networkID" : "0x2",
		"forkBlock": "0xffffffffffffffff",
		"forkCanonHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"gasLimitBoundDivisor": "0x20"
	},
	"genesis": {
		"seal": {
			"ethereum": {
				"mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
				"nonce": "0x00006d6f7264656e"
			}
		},
		"difficulty": "0x20000",
		"author": "0x0000000000000000000000000000000000000000",
		"timestamp": "0x00",
		"parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"extraData": "0x",
		"gasLimit": "0x2fefd8"
	},
	"nodes": [
		"enode://b1217cbaa440e35ed471157123fe468e19e8b5ad5bedb4b1fdbcbdab6fb2f5ed3e95dd9c24a22a79fdb2352204cea207df27d92bfd21bfd41545e8b16f637499@104.44.138.37:30303"
	],
	"accounts": {
		"0000000000000000000000000000000000000001": { "balance": "1", "nonce": "1048576", "builtin": { "name": "ecrecover", "pricing": { "linear": { "base": 3000, "word": 0 } } } },
		"0000000000000000000000000000000000000002": { "balance": "1", "nonce": "1048576", "builtin": { "name": "sha256", "pricing": { "linear": { "base": 60, "word": 12 } } } },
		"0000000000000000000000000000000000000003": { "balance": "1", "nonce": "1048576", "builtin": { "name": "ripemd160", "pricing": { "linear": { "base": 600, "word": 120 } } } },
		"0000000000000000000000000000000000000004": { "balance": "1", "nonce": "1048576", "builtin": { "name": "identity", "pricing": { "linear": { "base": 15, "word": 3 } } } },
		"102e61f5d8f9bc71d0ad4a084df4e65e05ce0e1c": { "balance": "1606938044258990275541962092341162602522202993782792835301376", "nonce": "1048576" }
	}
		}"#;
		let _deserialized: Spec = serde_json::from_str(s).unwrap();
		// TODO: validate all fields
	}

	#[test]
	fn test_zero_valued_divisor() {
		let s = r#"{
	"name": "Morden",
	"dataDir": "morden",
	"engine": {
		"Ethash": {
			"params": {
				"minimumDifficulty": "0x020000",
				"difficultyBoundDivisor": "0x0800",
				"durationLimit": "0x0d",
				"homesteadTransition" : "0x",
				"daoHardforkTransition": "0xffffffffffffffff",
				"daoHardforkBeneficiary": "0x0000000000000000000000000000000000000000",
				"daoHardforkAccounts": []
			}
		}
	},
	"params": {
		"accountStartNonce": "0x0100000",
		"homesteadTransition": "0x789b0",
		"maximumExtraDataSize": "0x20",
		"minGasLimit": "0x1388",
		"networkID" : "0x2",
		"forkBlock": "0xffffffffffffffff",
		"forkCanonHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"gasLimitBoundDivisor": "0x0"
	},
	"genesis": {
		"seal": {
			"ethereum": {
				"mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
				"nonce": "0x00006d6f7264656e"
			}
		},
		"difficulty": "0x20000",
		"author": "0x0000000000000000000000000000000000000000",
		"timestamp": "0x00",
		"parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
		"extraData": "0x",
		"gasLimit": "0x2fefd8"
	},
	"nodes": [
		"enode://b1217cbaa440e35ed471157123fe468e19e8b5ad5bedb4b1fdbcbdab6fb2f5ed3e95dd9c24a22a79fdb2352204cea207df27d92bfd21bfd41545e8b16f637499@104.44.138.37:30303"
	],
	"accounts": {
		"0000000000000000000000000000000000000001": { "balance": "1", "nonce": "1048576", "builtin": { "name": "ecrecover", "pricing": { "linear": { "base": 3000, "word": 0 } } } },
		"0000000000000000000000000000000000000002": { "balance": "1", "nonce": "1048576", "builtin": { "name": "sha256", "pricing": { "linear": { "base": 60, "word": 12 } } } },
		"0000000000000000000000000000000000000003": { "balance": "1", "nonce": "1048576", "builtin": { "name": "ripemd160", "pricing": { "linear": { "base": 600, "word": 120 } } } },
		"0000000000000000000000000000000000000004": { "balance": "1", "nonce": "1048576", "builtin": { "name": "identity", "pricing": { "linear": { "base": 15, "word": 3 } } } },
		"102e61f5d8f9bc71d0ad4a084df4e65e05ce0e1c": { "balance": "1606938044258990275541962092341162602522202993782792835301376", "nonce": "1048576" }
	}
		}"#;

		match Spec::load(s.as_bytes()) {
			Err(e) => {
				match e {
					Error::ZeroValueDivisor => {},
					wrong => panic!("Unexpected error returned: {}", wrong),
				}
			},
			Ok(_) => panic!("Spec should fail to load"),
		}
	}
}
