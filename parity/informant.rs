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

extern crate ansi_term;
use self::ansi_term::Colour::{White, Yellow, Green, Cyan, Blue};
use self::ansi_term::Style;

use std::time::{Instant, Duration};
use std::ops::{Deref, DerefMut};
use isatty::{stdout_isatty};
use ethsync::{SyncStatus, NetworkConfiguration};
use util::{Uint, RwLock};
use ethcore::client::*;
use number_prefix::{binary_prefix, Standalone, Prefixed};

pub struct Informant {
	chain_info: RwLock<Option<BlockChainInfo>>,
	cache_info: RwLock<Option<BlockChainCacheSize>>,
	report: RwLock<Option<ClientReport>>,
	last_tick: RwLock<Instant>,
	with_color: bool,
}

impl Default for Informant {
	fn default() -> Self {
		Informant {
			chain_info: RwLock::new(None),
			cache_info: RwLock::new(None),
			report: RwLock::new(None),
			last_tick: RwLock::new(Instant::now()),
			with_color: true,
		}
	}
}

trait MillisecondDuration {
	fn as_milliseconds(&self) -> u64;
}

impl MillisecondDuration for Duration {
	fn as_milliseconds(&self) -> u64 {
		self.as_secs() * 1000 + self.subsec_nanos() as u64 / 1000000
	}
}

impl Informant {
	/// Make a new instance potentially `with_color` output.
	pub fn new(with_color: bool) -> Self {
		Informant {
			chain_info: RwLock::new(None),
			cache_info: RwLock::new(None),
			report: RwLock::new(None),
			last_tick: RwLock::new(Instant::now()),
			with_color: with_color,
		}
	}

	fn format_bytes(b: usize) -> String {
		match binary_prefix(b as f64) {
			Standalone(bytes)   => format!("{} bytes", bytes),
			Prefixed(prefix, n) => format!("{:.0} {}B", n, prefix),
		}
	}


	#[cfg_attr(feature="dev", allow(match_bool))]
	pub fn tick(&self, client: &Client, maybe_status: Option<(SyncStatus, NetworkConfiguration)>) {
		let elapsed = self.last_tick.read().elapsed();
		if elapsed < Duration::from_secs(5) {
			return;
		}

		let chain_info = client.chain_info();
		let queue_info = client.queue_info();
		let cache_info = client.blockchain_cache_info();
		let report = client.report();

		let last_report = match self.report.read().deref() { &Some(ref last_report) => last_report.clone(), _ => ClientReport::default() };

		let importing = queue_info.unverified_queue_size + queue_info.verified_queue_size > 3 || ((report.blocks_imported - last_report.blocks_imported) * 1000000) as u64 / elapsed.as_milliseconds() > 1000;
		if !importing && elapsed < Duration::from_secs(30) {
			return;
		}

		*self.last_tick.write() = Instant::now();

		let paint = |c: Style, t: String| match self.with_color && stdout_isatty() {
			true => format!("{}", c.paint(t)),
			false => t,
		};

		info!("{}   {}   {}",
			match importing {
				true => format!("{} {}   {}   {}+{} Qed", 
					paint(White.bold(), format!("{:>8}", format!("#{}", chain_info.best_block_number))),
					paint(White.bold(), format!("{}", chain_info.best_block_hash)),
					format!("{} blk/s {} tx/s {} Mgas/s",  
						paint(Yellow.bold(), format!("{:4}", ((report.blocks_imported - last_report.blocks_imported) * 1000) as u64 / elapsed.as_milliseconds())),
						paint(Yellow.bold(), format!("{:4}", ((report.transactions_applied - last_report.transactions_applied) * 1000) as u64 / elapsed.as_milliseconds())),
						paint(Yellow.bold(), format!("{:3}", ((report.gas_processed - last_report.gas_processed) / From::from(elapsed.as_milliseconds() * 1000)).low_u64()))
					),
					paint(Green.bold(), format!("{:5}", queue_info.unverified_queue_size)),
					paint(Green.bold(), format!("{:5}", queue_info.verified_queue_size))
				),
				false => String::new(),
			},
			match maybe_status {
				Some((ref sync_info, ref net_config)) => format!("{}{}/{}/{} peers",
					match importing {
						true => format!("{}   ", paint(Green.bold(), format!("{:>8}", format!("#{}", sync_info.last_imported_block_number.unwrap_or(chain_info.best_block_number))))),
						false => String::new(),
					},
					paint(Cyan.bold(), format!("{:2}", sync_info.num_active_peers)),
					paint(Cyan.bold(), format!("{:2}", sync_info.num_peers)),
					paint(Cyan.bold(), format!("{:2}", net_config.ideal_peers))
				),
				None => String::new(),
			},
			format!("{} db {} chain {} queue{}", 
				paint(Blue.bold(), format!("{:>8}", Informant::format_bytes(report.state_db_mem))),
				paint(Blue.bold(), format!("{:>8}", Informant::format_bytes(cache_info.total()))),
				paint(Blue.bold(), format!("{:>8}", Informant::format_bytes(queue_info.mem_used))),
				match maybe_status {
					Some((ref sync_info, _)) => format!(" {} sync", paint(Blue.bold(), format!("{:>8}", Informant::format_bytes(sync_info.mem_used)))), 
					_ => String::new(),
				}
			)
		);

		*self.chain_info.write().deref_mut() = Some(chain_info);
		*self.cache_info.write().deref_mut() = Some(cache_info);
		*self.report.write().deref_mut() = Some(report);
	}
}

