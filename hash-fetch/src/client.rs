// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

//! Hash-addressed content resolver & fetcher.

use std::{io, fs};
use std::sync::Arc;
use std::path::PathBuf;

use fetch::{Fetch, Error as FetchError, Client as FetchClient};
use futures::Future;
use parity_reactor::Remote;
use urlhint::{ContractClient, URLHintContract, URLHint, URLHintResult};
use util::{H256, sha3};

/// API for fetching by hash.
pub trait HashFetch: Send + Sync + 'static {
	/// Fetch hash-addressed content.
	/// Parameters:
	/// 1. `hash` - content hash
	/// 2. `on_done` - callback function invoked when the content is ready (or there was error during fetch)
	///
	/// This function may fail immediately when fetch cannot be initialized or content cannot be resolved.
	fn fetch(&self, hash: H256, on_done: Box<Fn(Result<PathBuf, Error>) + Send>);
}

/// Hash-fetching error.
#[derive(Debug)]
pub enum Error {
	/// Hash could not be resolved to a valid content address.
	NoResolution,
	/// Downloaded content hash does not match.
	HashMismatch {
		/// Expected hash
		expected: H256,
		/// Computed hash
		got: H256,
	},
	/// IO Error while validating hash.
	IO(io::Error),
	/// Error during fetch.
	Fetch(FetchError),
}

impl From<FetchError> for Error {
	fn from(error: FetchError) -> Self {
		Error::Fetch(error)
	}
}

impl From<io::Error> for Error {
	fn from(error: io::Error) -> Self {
		Error::IO(error)
	}
}

/// Default Hash-fetching client using on-chain contract to resolve hashes to URLs.
pub struct Client<F: Fetch + 'static = FetchClient> {
	contract: URLHintContract,
	fetch: F,
	remote: Remote,
}

impl Client {
	/// Creates new instance of the `Client` given on-chain contract client and task runner.
	pub fn new(contract: Arc<ContractClient>, remote: Remote) -> Self {
		Client::with_fetch(contract, FetchClient::new().unwrap(), remote)
	}
}

impl<F: Fetch + 'static> Client<F> {

	/// Creates new instance of the `Client` given on-chain contract client, fetch service and task runner.
	pub fn with_fetch(contract: Arc<ContractClient>, fetch: F, remote: Remote) -> Self {
		Client {
			contract: URLHintContract::new(contract),
			fetch: fetch,
			remote: remote,
		}
	}
}

impl<F: Fetch + 'static> HashFetch for Client<F> {
	fn fetch(&self, hash: H256, on_done: Box<Fn(Result<PathBuf, Error>) + Send>) {
		debug!(target: "fetch", "Fetching: {:?}", hash);

		let url = self.contract.resolve(hash.to_vec()).map(|content| match content {
				URLHintResult::Dapp(dapp) => {
					dapp.url()
				},
				URLHintResult::Content(content) => {
					content.url
				},
		}).ok_or_else(|| Error::NoResolution);

		debug!(target: "fetch", "Resolved {:?} to {:?}. Fetching...", hash, url);

		match url {
			Err(err) => on_done(Err(err)),
			Ok(url) => {
				let future = self.fetch.fetch_to_file(&url, &F::temp_filename(), Default::default()).then(move |result| {
					fn validate_hash(hash: H256, result: Result<PathBuf, FetchError>) -> Result<PathBuf, Error> {
						let path = try!(result);
						let mut file_reader = io::BufReader::new(try!(fs::File::open(&path)));
						let content_hash = try!(sha3(&mut file_reader));

						if content_hash != hash {
							Err(Error::HashMismatch{ got: content_hash, expected: hash })
						} else {
							Ok(path)
						}
					}

					debug!(target: "fetch", "Content fetched, validating hash ({:?})", hash);
					on_done(validate_hash(hash, result.map(|x| x.0)));
					Ok(()) as Result<(), ()>
				});
				self.remote.spawn(future);
			},
		}
	}
}
