// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

//! Fat database.
use std::collections::HashMap;
use std::sync::RwLock;
use std::path::Path;
use bloomchain::Number;
use bloomchain::group::{BloomGroupDatabase, BloomGroupChain, GroupPosition, BloomGroup};
use blockchain::ImportRoute;
use util::{FixedHash, H256, H264, Database, DBTransaction};
use header::BlockNumber;
use trace::{BlockTraces, LocalizedTrace};
use db::{Key, Writable, Readable, BatchWriter, DatabaseReader, CacheUpdatePolicy};
use super::Config;
use super::trace::{Filter, TraceGroupPosition, BlockTracesBloom, BlockTracesBloomGroup, BlockTracesDetails,
FlatBlockTraces, FlatTransactionTraces, FlatTrace};

#[derive(Debug, Copy, Clone)]
pub enum FatdbIndex {
	/// Block traces index.
	BlockTraces = 0,
	/// Trace bloom group index.
	BlockTracesBloomGroups = 1,
}

fn with_index(hash: &H256, i: FatdbIndex) -> H264 {
	let mut slice = H264::from_slice(hash);
	slice[32] = i as u8;
	slice
}

impl Key<BlockTraces> for H256 {
	fn key(&self) -> H264 {
		with_index(self, FatdbIndex::BlockTraces)
	}
}

impl Key<BlockTracesBloomGroup> for TraceGroupPosition {
	fn key(&self) -> H264 {
		with_index(&self.hash(), FatdbIndex::BlockTracesBloomGroups)
	}
}

/// Fat database.
pub struct Fatdb {
	// cache
	traces: RwLock<HashMap<H256, BlockTraces>>,
	blooms: RwLock<HashMap<TraceGroupPosition, BlockTracesBloomGroup>>,
	// db
	tracesdb: Database,
	// config,
	config: Config,
}

impl BloomGroupDatabase for Fatdb {
	fn blooms_at(&self, position: &GroupPosition) -> Option<BloomGroup> {
		let position = TraceGroupPosition::from(position.clone());
		DatabaseReader::new(&self.tracesdb, &self.blooms)
			.read(&position)
			.map(Into::into)
	}
}

impl Fatdb {
	/// Creates new instance of `Fatdb`.
	pub fn new(mut config: Config, path: &Path) -> Self {
		let mut fatdb_path = path.to_path_buf();
		fatdb_path.push("fatdb");
		let mut tracesdb_path = fatdb_path.clone();
		tracesdb_path.push("traces");
		let tracesdb = Database::open_default(tracesdb_path.to_str().unwrap()).unwrap();

		// check if in previously tracing was enabled
		let tracing_was_enabled = match tracesdb.get(b"enabled").unwrap() {
			Some(ref value) if value as &[u8] == &[0x1] => Some(true),
			Some(ref value) if value as &[u8] == &[0x0] => Some(false),
			Some(_) => { panic!("tracesdb is malformed") },
			None => None,
		};

		// compare it with the current option.
		let tracing = match (tracing_was_enabled, config.tracing.enabled) {
			(Some(true), Some(true)) => true,
			(Some(true), None) => true,
			(Some(true), Some(false)) => false,
			(Some(false), Some(true)) => { panic!("Tracing can't be enabled. Resync required."); },
			(Some(false), None) => false,
			(Some(false), Some(false)) => false,
			(None, Some(true)) => true,
			_ => false,
		};

		config.tracing.enabled = Some(tracing);

		let encoded_tracing= match tracing {
			true => [0x1],
			false => [0x0]
		};

		tracesdb.put(b"enabled", &encoded_tracing).unwrap();

		Fatdb {
			traces: RwLock::new(HashMap::new()),
			blooms: RwLock::new(HashMap::new()),
			tracesdb: tracesdb,
			config: config,
		}
	}

	/// Returns true if trasing is enabled. Otherwise false.
	pub fn is_tracing_enabled(&self) -> bool {
		self.config.tracing.enabled.expect("Auto tracing hasn't been properly configured.")
	}

	/// Imports new block traces and rebuilds the blooms based on the import route.
	///
	/// Traces of all import route's enacted blocks are expected to be already in database
	/// or to be the currenly inserted trace.
	pub fn import_traces(&self, details: BlockTracesDetails, route: &ImportRoute) {
		// fast return if tracing is disabled
		if !self.is_tracing_enabled() {
			return;
		}
		// in real world it's impossible to get import route with retracted blocks
		// and no enacted blocks and this function does not handle this case
		// so let's just panic.
		assert!(! (!route.retracted.is_empty() && route.enacted.is_empty() ), "Invalid import route!");

		let batch = DBTransaction::new();

		// at first, let's insert new block traces
		{
			let mut traces = self.traces.write().unwrap();
			// it's important to use overwrite here,
			// cause this value might be queried by hash later
			BatchWriter::new(&batch, &mut traces).write(details.hash, details.traces, CacheUpdatePolicy::Overwrite);
		}

		// now let's rebuild the blooms
		// do it only if some blocks have been enacted
		if !route.enacted.is_empty() {
			let replaced_range = details.number as Number - route.retracted.len()..details.number as Number;
			let enacted_blooms = route.enacted
				.iter()
				// all traces are expected to be found here. That's why `expect` has been used
				// instead of `filter_map`. If some traces haven't been found, it meens that
				// traces database is malformed or incomplete.
				.map(|block_hash| self.traces(block_hash).expect("Traces database is incomplete."))
				.map(|block_traces| block_traces.bloom())
				.map(BlockTracesBloom::from)
				.map(Into::into)
				.collect();

			let chain = BloomGroupChain::new(self.config.tracing.blooms, self);
			let trace_blooms = chain.replace(&replaced_range, enacted_blooms);
			let blooms_to_insert = trace_blooms.into_iter()
				.map(|p| (From::from(p.0), From::from(p.1)))
				.collect::<HashMap<TraceGroupPosition, BlockTracesBloomGroup>>();

			let mut blooms = self.blooms.write().unwrap();
			BatchWriter::new(&batch, &mut blooms).extend(blooms_to_insert, CacheUpdatePolicy::Remove);
		}

		self.tracesdb.write(batch).unwrap();
	}

	/// Returns traces for block with hash.
	pub fn traces(&self, block_hash: &H256) -> Option<BlockTraces> {
		DatabaseReader::new(&self.tracesdb, &self.traces).read(block_hash)
	}

	/// Returns traces matching given filter.
	pub fn filter_traces<F>(&self, filter: &Filter, block_hash: F) -> Vec<LocalizedTrace> where
	F: Fn(BlockNumber) -> H256 {
		let chain = BloomGroupChain::new(self.config.tracing.blooms, self);
		let numbers = chain.filter(filter);
		numbers.into_iter()
			.flat_map(|n| {
				let number = n as BlockNumber;
				let hash = block_hash(number);
				let traces = self.traces(&hash).expect("Expected to find a trace. Db is probably malformed.");
				let flat_block = FlatBlockTraces::from(traces);
				let tx_traces: Vec<FlatTransactionTraces> = flat_block.into();
				tx_traces.into_iter()
					.enumerate()
					.flat_map(|(tx_number, tx_trace)| {
						let flat_traces: Vec<FlatTrace> = tx_trace.into();
						flat_traces.into_iter()
							.enumerate()
							.filter_map(|(index, trace)| {
								match filter.matches(&trace) {
									true => Some(LocalizedTrace {
										parent: trace.parent,
										children: trace.children,
										depth: trace.depth,
										action: trace.action,
										result: trace.result,
										trace_number: index,
										transaction_number: tx_number,
										block_number: number,
										block_hash: hash.clone()
									}),
									false => None
								}
							})
							.collect::<Vec<_>>()
					})
					.collect::<Vec<_>>()
			})
			.collect::<Vec<_>>()
	}
}

#[cfg(test)]
mod tests {
	use devtools::RandomTempPath;
	use fatdb::Config;
	use super::Fatdb;

	#[test]
	fn test_reopening_db_with_tracing_off() {
		let temp = RandomTempPath::new();
		let mut config = Config::default();

		// set autotracing
		config.tracing.enabled = None;

		{
			let fatdb = Fatdb::new(config.clone(), temp.as_path());
			assert_eq!(fatdb.is_tracing_enabled(), false);
		}

		{
			let fatdb = Fatdb::new(config.clone(), temp.as_path());
			assert_eq!(fatdb.is_tracing_enabled(), false);
		}

		config.tracing.enabled = Some(false);

		{
			let fatdb = Fatdb::new(config.clone(), temp.as_path());
			assert_eq!(fatdb.is_tracing_enabled(), false);
		}
	}

	#[test]
	fn test_reopeining_db_with_tracing_on() {
		let temp = RandomTempPath::new();
		let mut config = Config::default();

		// set tracing on
		config.tracing.enabled = Some(true);

		{
			let fatdb = Fatdb::new(config.clone(), temp.as_path());
			assert_eq!(fatdb.is_tracing_enabled(), true);
		}

		{
			let fatdb = Fatdb::new(config.clone(), temp.as_path());
			assert_eq!(fatdb.is_tracing_enabled(), true);
		}

		config.tracing.enabled = None;

		{
			let fatdb = Fatdb::new(config.clone(), temp.as_path());
			assert_eq!(fatdb.is_tracing_enabled(), true);
		}

		config.tracing.enabled = Some(false);

		{
			let fatdb = Fatdb::new(config.clone(), temp.as_path());
			assert_eq!(fatdb.is_tracing_enabled(), false);
		}
	}

	#[test]
	#[should_panic]
	fn test_invalid_reopeining_db() {
		let temp = RandomTempPath::new();
		let mut config = Config::default();

		// set tracing on
		config.tracing.enabled = Some(false);

		{
			let fatdb = Fatdb::new(config.clone(), temp.as_path());
			assert_eq!(fatdb.is_tracing_enabled(), true);
		}

		config.tracing.enabled = Some(true);
		Fatdb::new(config.clone(), temp.as_path()); // should panic!
	}
}
