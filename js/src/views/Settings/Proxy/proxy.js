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

import React, { Component, PropTypes } from 'react';

import { Container, ContainerTitle, Translate } from '../../../ui';

import layout from '../layout.css';
import styles from './proxy.css';

export default class Proxy extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  render () {
    const { dappsUrl } = this.context.api;
    const proxyurl = `${dappsUrl}/proxy/proxy.pac`;
    const title = <Translate id='settings.proxy.label' />;

    return (
      <Container>
        <ContainerTitle title={ title } />
        <div className={ layout.layout }>
          <div className={ layout.overview }>
            <div>
              <Translate
                id='settings.proxy.overview_0'
                default='The proxy setup allows you to access Parity and all associated decentralized applications via memorable addresses.' />
            </div>
          </div>
          <div className={ layout.details }>
            <div className={ styles.details }>
              <div>
                <Translate
                  id='settings.proxy.details_0'
                  default='Instead of accessing Parity via the IP address and port, you will be able to access it via the .parity subdomain, by visiting {homeProxy}. To setup subdomain-based routing, you need to add the relevant proxy entries to your browser,'
                  values={ {
                    homeProxy: <span className={ layout.console }>http://home.parity/</span>
                  } } />
              </div>
              <div className={ layout.center }>
                <a href={ proxyurl } target='_blank'>{ proxyurl }</a>
              </div>
              <div>
                <Translate
                  id='settings.proxy.details_1'
                  default='To learn how to configure the proxy, instructions are provided for {windowsLink}, {macOSLink} or {ubuntuLink}.'
                  values={ {
                    windowsLink: <a href='https://blogs.msdn.microsoft.com/ieinternals/2013/10/11/understanding-web-proxy-configuration/' target='_blank'><Translate id='details_windows' default='Windows' /></a>,
                    macOSLink: <a href='https://support.apple.com/kb/PH18553?locale=en_US' target='_blank'><Translate id='settings.proxy.details_macos' default='macOS' /></a>,
                    ubuntuLink: <a href='http://xmodulo.com/how-to-set-up-proxy-auto-config-on-ubuntu-desktop.html' target='_blank'><Translate id='settings.proxy.details_ubuntu' default='Ubuntu' /></a>
                  } } />
              </div>
            </div>
          </div>
        </div>
      </Container>
    );
  }
}
