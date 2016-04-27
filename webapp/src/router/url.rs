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

//! HTTP/HTTPS URL type. Based on URL type from Iron library.

use url::Host;
use url::{self};

/// HTTP/HTTPS URL type for Iron.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Url {
	/// The lower-cased scheme of the URL, typically "http" or "https".
	pub scheme: String,

	/// The host field of the URL, probably a domain.
	pub host: Host,

	/// The connection port.
	pub port: u16,

	/// The URL path, the resource to be accessed.
	///
	/// A *non-empty* vector encoding the parts of the URL path.
	/// Empty entries of `""` correspond to trailing slashes.
	pub path: Vec<String>,

	/// The URL username field, from the userinfo section of the URL.
	///
	/// `None` if the `@` character was not part of the input OR
	/// if a blank username was provided.
	/// Otherwise, a non-empty string.
	pub username: Option<String>,

	/// The URL password field, from the userinfo section of the URL.
	///
	/// `None` if the `@` character was not part of the input OR
	/// if a blank password was provided.
	/// Otherwise, a non-empty string.
	pub password: Option<String>,

	/// The URL query string.
	///
	/// `None` if the `?` character was not part of the input.
	/// Otherwise, a possibly empty, percent encoded string.
	pub query: Option<String>,

	/// The URL fragment.
	///
	/// `None` if the `#` character was not part of the input.
	/// Otherwise, a possibly empty, percent encoded string.
	pub fragment: Option<String>
}

impl Url {
	/// Create a URL from a string.
	///
	/// The input must be a valid URL with a special scheme for this to succeed.
	///
	/// HTTP and HTTPS are special schemes.
	///
	/// See: http://url.spec.whatwg.org/#special-scheme
	pub fn parse(input: &str) -> Result<Url, String> {
		// Parse the string using rust-url, then convert.
		match url::Url::parse(input) {
			Ok(raw_url) => Url::from_generic_url(raw_url),
			Err(e) => Err(format!("{}", e))
		}
	}

	/// Create a `Url` from a `rust-url` `Url`.
	pub fn from_generic_url(raw_url: url::Url) -> Result<Url, String> {
		// Extract the port as a 16-bit unsigned integer.
		let port: u16 = match raw_url.port_or_known_default() {
			Some(port) => port,
			None => {
				return Err(format!("Unknown port for scheme: `{}`", raw_url.scheme()))
			}
		};

		// Map empty usernames to None.
		let username = match &*raw_url.username() {
			"" => None,
			_ => Some(raw_url.username().to_owned())
		};

		// Map empty passwords to None.
		let password = match raw_url.password() {
			None => None,
			Some(ref x) if x.is_empty() => None,
			Some(password) => Some(password.to_owned())
		};
		let as_string = |x: &str| x.to_owned();

		Ok(Url {
			scheme: raw_url.scheme().to_owned(),
			host: raw_url.host().expect("Valid host, because only data:, mailto: protocols does not have host.").to_owned(),
			port: port,
			path: raw_url.path_segments().expect("Valid path segments. In HTTP we won't get cannot-be-a-base URLs").map(&as_string).collect(),
			username: username,
			password: password,
			query: raw_url.query().map(&as_string),
			fragment: raw_url.fragment().map(&as_string),
		})
	}
}

#[cfg(test)]
mod test {
    use super::Url;

    #[test]
    fn test_default_port() {
        assert_eq!(Url::parse("http://example.com/wow").unwrap().port, 80u16);
        assert_eq!(Url::parse("https://example.com/wow").unwrap().port, 443u16);
    }

    #[test]
    fn test_explicit_port() {
        assert_eq!(Url::parse("http://localhost:3097").unwrap().port, 3097u16);
    }

    #[test]
    fn test_empty_username() {
        assert!(Url::parse("http://@example.com").unwrap().username.is_none());
        assert!(Url::parse("http://:password@example.com").unwrap().username.is_none());
    }

    #[test]
    fn test_not_empty_username() {
        let user = Url::parse("http://john:pass@example.com").unwrap().username;
        assert_eq!(user.unwrap(), "john");

        let user = Url::parse("http://john:@example.com").unwrap().username;
        assert_eq!(user.unwrap(), "john");
    }

    #[test]
    fn test_empty_password() {
        assert!(Url::parse("http://michael@example.com").unwrap().password.is_none());
        assert!(Url::parse("http://:@example.com").unwrap().password.is_none());
    }

    #[test]
    fn test_not_empty_password() {
        let pass = Url::parse("http://michael:pass@example.com").unwrap().password;
        assert_eq!(pass.unwrap(), "pass");

        let pass = Url::parse("http://:pass@example.com").unwrap().password;
        assert_eq!(pass.unwrap(), "pass");
    }
}
