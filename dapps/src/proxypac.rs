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

//! Serving ProxyPac file

use endpoint::{Endpoint, Handler, EndpointPath};
use handlers::ContentHandler;
use apps::DAPPS_DOMAIN;
use signer_address;

pub struct ProxyPac {
	signer_port: Option<u16>,
}

impl ProxyPac {
	pub fn boxed(signer_port: Option<u16>) -> Box<Endpoint> {
		Box::new(ProxyPac {
			signer_port: signer_port
		})
	}
}

impl Endpoint for ProxyPac {
	fn to_handler(&self, path: EndpointPath) -> Box<Handler> {
		let signer = self.signer_port
			.map(signer_address)
			.unwrap_or_else(|| format!("{}:{}", path.host, path.port));

		let content = format!(
r#"
function FindProxyForURL(url, host) {{
	if (shExpMatch(host, "home{0}"))
	{{
		return "PROXY {3}";
	}}

	if (shExpMatch(host, "*{0}"))
	{{
		return "PROXY {1}:{2}";
	}}

	return "DIRECT";
}}
"#,
		DAPPS_DOMAIN, path.host, path.port, signer);

		Box::new(ContentHandler::ok(content, "application/javascript".to_owned()))
	}
}


