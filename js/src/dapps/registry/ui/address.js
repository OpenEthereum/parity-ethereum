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

import React, { PropTypes } from 'react';
import { connect } from 'react-redux';

import Hash from './hash';
import IdentityIcon from '../IdentityIcon';

import styles from './address.css';

const Address = ({ address, accounts, contacts, key, shortenHash }) => {
  let caption;
  if (accounts[address] || contacts[address]) {
    const name = (accounts[address] || contacts[address] || {}).name;
    caption = (
      <abbr title={ address } className={ styles.align }>
        { name || address }
      </abbr>
    );
  } else {
    caption = (
      <code className={ styles.align }>
        { shortenHash ? (<Hash hash={ address } linked />) : address }
      </code>
    );
  }

  return (
    <div key={ key } className={ styles.container }>
      <IdentityIcon address={ address } className={ styles.align } />
      { caption }
    </div>
  );
};

Address.propTypes = {
  address: PropTypes.string.isRequired,
  accounts: PropTypes.object.isRequired,
  contacts: PropTypes.object.isRequired,
  key: PropTypes.string,
  shortenHash: PropTypes.bool
};

Address.defaultProps = {
  key: 'address',
  shortenHash: true
};

export default connect(
  // mapStateToProps
  (state) => ({
    accounts: state.accounts.all,
    contacts: state.contacts
  }),
  // mapDispatchToProps
  null
)(Address);
