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

import styles from './modal.css';

export default class Modal extends Component {
  static propTypes = {
    buttons: PropTypes.node.isRequired,
    children: PropTypes.node.isRequired,
    header: PropTypes.string.isRequired,
    visible: PropTypes.bool.isRequired
  }

  render () {
    const { children, buttons, header, visible } = this.props;

    if (!visible) {
      return null;
    }

    return (
      <div className={ styles.modal }>
        <div className={ styles.overlay } />
        <div className={ styles.body }>
          <div className={ styles.dialog }>
            <div className={ styles.header }>
              { header }
            </div>
            <div className={ styles.content }>
              { children }
            </div>
            <div className={ styles.footer }>
              { buttons }
            </div>
          </div>
        </div>
      </div>
    );
  }
}
