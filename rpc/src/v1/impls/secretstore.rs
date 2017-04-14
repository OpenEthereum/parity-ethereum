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

//! SecretStore-specific rpc implementation.

use std::sync::{Arc, Weak};

use crypto::DEFAULT_MAC;
use ethkey::Secret;
use ethcore::account_provider::AccountProvider;

use jsonrpc_core::Error;
use v1::helpers::errors;
use v1::helpers::accounts::unwrap_provider;
use v1::traits::SecretStore;
use v1::types::{H160, H512, Bytes};

/// Parity implementation.
pub struct SecretStoreClient {
	accounts: Option<Weak<AccountProvider>>,
}

impl SecretStoreClient {
	/// Creates new SecretStoreClient
	pub fn new(store: &Option<Arc<AccountProvider>>) -> Self {
		SecretStoreClient {
			accounts: store.as_ref().map(Arc::downgrade),
		}
	}

	/// Attempt to get the `Arc<AccountProvider>`, errors if provider was not
	/// set, or if upgrading the weak reference failed.
	fn account_provider(&self) -> Result<Arc<AccountProvider>, Error> {
		unwrap_provider(&self.accounts)
	}

	/// Decrypt public key using account' private key
	fn decrypt_key(&self, address: H160, key: Bytes) -> Result<Vec<u8>, Error> {
		let store = self.account_provider()?;
		store.decrypt(address.into(), None, &DEFAULT_MAC, &key.0)
			.map_err(|e| errors::account("Could not decrypt key.", e))
	}

	/// Decrypt secret key using account' private key
	fn decrypt_secret(&self, address: H160, key: Bytes) -> Result<Secret, Error> {
		self.decrypt_key(address, key)
			.and_then(|s| Secret::from_slice(&s).map_err(|e| errors::account("invalid secret", e)))
	}
}

impl SecretStore for SecretStoreClient {
	fn encrypt(&self, address: H160, key: Bytes, data: Bytes) -> Result<Bytes, Error> {
		encryption::encrypt_document(self.decrypt_key(address, key)?, data.0)
			.map(Into::into)
	}

	fn decrypt(&self, address: H160, key: Bytes, data: Bytes) -> Result<Bytes, Error> {
		encryption::decrypt_document(self.decrypt_key(address, key)?, data.0)
			.map(Into::into)
	}

	fn shadow_decrypt(&self, address: H160, decrypted_secret: H512, common_point: H512, decrypt_shadows: Vec<Bytes>, data: Bytes) -> Result<Bytes, Error> {
		let mut shadows = Vec::with_capacity(decrypt_shadows.len());
		for decrypt_shadow in decrypt_shadows {
			shadows.push(self.decrypt_secret(address.clone(), decrypt_shadow)?);
		}

		encryption::decrypt_document_with_shadow(decrypted_secret.into(), common_point.into(), shadows, data.0)
			.map(Into::into)
	}
}

#[cfg(not(feature="secretstore"))]
mod encryption {
	use ethkey::{Secret, Public};
	use jsonrpc_core::Error;
	use util::Bytes;
	use v1::helpers::errors;

	pub fn encrypt_document(_key: Vec<u8>, _document: Bytes) -> Result<Bytes, Error> {
		Err(errors::secretstore_disabled())
	}

	pub fn decrypt_document(_key: Vec<u8>, _document: Bytes) -> Result<Bytes, Error> {
		Err(errors::secretstore_disabled())
	}

	pub fn decrypt_document_with_shadow(_decrypted_secret: Public, _common_point: Public, _shadows: Vec<Secret>, _document: Bytes) -> Result<Bytes, Error> {
		Err(errors::secretstore_disabled())
	}
}

#[cfg(feature="secretstore")]
mod encryption {
	use ethkey::{Secret, Public};
	use jsonrpc_core::Error;
	use ethcore_secretstore;
	use util::Bytes;
	use v1::helpers::errors;

	pub fn encrypt_document(key: Vec<u8>, document: Bytes) -> Result<Bytes, Error> {
		ethcore_secretstore::encrypt_document(key, document)
			.map_err(|e| errors::encryption_error(e))
	}

	pub fn decrypt_document(key: Vec<u8>, document: Bytes) -> Result<Bytes, Error> {
		ethcore_secretstore::decrypt_document(key, document)
			.map_err(|e| errors::encryption_error(e))
	}

	pub fn decrypt_document_with_shadow(decrypted_secret: Public, common_point: Public, shadows: Vec<Secret>, document: Bytes) -> Result<Bytes, Error> {
		ethcore_secretstore::decrypt_document_with_shadow(decrypted_secret, common_point, shadows, document)
			.map_err(|e| errors::encryption_error(e))
	}
}
