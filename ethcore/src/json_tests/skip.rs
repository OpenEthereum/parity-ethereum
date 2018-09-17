// Copyright 2015-2018 Parity Technologies (UK) Ltd.
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

//! State tests to skip.

use ethjson;

lazy_static!{


	pub static ref SKIP_TEST_STATE: ethjson::test::SkipStates = {
		let skip_data = include_bytes!("../../res/ethereum/tests-issues/currents.json");
		ethjson::test::SkipStates::load(&skip_data[..]).expect("No invalid json allowed")
	};

}
