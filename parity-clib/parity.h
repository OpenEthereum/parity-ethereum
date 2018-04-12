// Copyright 2018 Parity Technologies (UK) Ltd.
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

#ifndef _PARITY_H_INCLUDED_
#define _PARITY_H_INCLUDED_

#include <cstddef>

/// Parameters to pass to `parity_start`.
struct ParityParams {
	/// Configuration object, as handled by the `parity_config_*` functions.
	/// Note that calling `parity_start` will destroy the configuration object (even on failure).
	void *configuration;

	/// Callback function to call when the client receives an RPC request to change its chain spec.
	///
	/// Will only be called if you enable the `--can-restart` flag.
	///
	/// The first parameter of the callback is the value of `on_client_restart_cb_custom`.
	/// The second and third parameters of the callback are the string pointer and length.
	void (*on_client_restart_cb)(void *, const char *, size_t);

	/// Custom parameter passed to the `on_client_restart_cb` callback as first parameter.
	void *on_client_restart_cb_custom;
};

#ifdef __cplusplus
extern "C" {
#endif

/// Builds a new configuration object by parsing a list of CLI arguments.
///
/// The first two parameters are string pointers and string lengths. They must have a length equal
/// to `len`. The strings don't need to be zero-terminated.
///
/// Returns 0 on success, and non-zero on error.
///
/// # Example
///
/// ```no_run
/// void* cfg;
/// const char *args[] = {"--light"};
/// size_t str_lens[] = {7};
/// if (parity_config_from_cli(args, str_lens, 1, &cfg) != 0) {
///     return 1;
/// }
/// ```
///
int parity_config_from_cli(char const *const *args, size_t const *arg_lens, size_t len, void **out);

/// Destroys a configuration object created earlier.
///
/// **Important**: You probably don't need to call this function. Calling `parity_start` destroys
/// 				the configuration object as well (even on failure).
void parity_config_destroy(void *);

/// Starts the parity client in background threads. Returns a pointer to a struct that represents
/// the running client. Can also return NULL if the execution completes instantly.
///
/// **Important**: The configuration object passed inside `cfg` is destroyed when you
/// 				call `parity_start` (even on failure).
///
/// Returns 0 on success, and non-zero on error.
int parity_start(const ParityParams *, void **);

/// Destroys the parity client created with `parity_start`.
///
/// **Warning**: `parity_start` can return NULL if execution finished instantly, in which case you
///					must not call this function.
void parity_destroy(void *);

#ifdef __cplusplus
}
#endif

#endif // include guard
