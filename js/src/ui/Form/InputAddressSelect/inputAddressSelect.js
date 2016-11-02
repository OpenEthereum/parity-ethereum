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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import AutoComplete from 'material-ui/AutoComplete';
import MenuItem from 'material-ui/MenuItem';
import normalize from 'normalize-for-search';

import IdentityIcon from '../../IdentityIcon';
import util from '../../../api/util';

import styles from './inputAddressSelect.css';

const isAccount = (choice) => choice.data && choice.data.uuid;

const isNotDeleted = (choice) => choice.data && choice.data.meta && !choice.data.meta.deleted;

const sortChoices = (a, b) => {
  // accounts first
  if (isAccount(a) && !isAccount(b)) return -1;
  if (!isAccount(a) && isAccount(b)) return 1;
  // alphabetically
  if (a.tokens < b.tokens) return -1;
  if (a.tokens > b.tokens) return 1;
  // fallback
  return 0;
};

class InputAddressSelect extends Component {
  static propTypes = {
    accounts: PropTypes.object.isRequired,
    contacts: PropTypes.object.isRequired,
    error: PropTypes.string,
    label: PropTypes.string,
    hint: PropTypes.string,
    value: PropTypes.string,
    maxSearchResults: PropTypes.number,
    onChange: PropTypes.func
  };

  static defaultProps = {
    onChange: () => {}
  };

  state = {
    choices: [],
    address: ''
  }

  componentWillMount () {
    this.updateChoices();
  }

  componentWillReceiveProps (nextProps) {
    this.updateChoices(nextProps);
  }

  render () {
    const { label, hint, error, maxSearchResults } = this.props;
    const { choices, address } = this.state;

    // don't show IdentityIcon if user searches by name
    const addressToRender = util.isAddressValid(address)
      ? address : (
        util.isAddressValid('0x' + address)
          ? '0x' + address
          : null
      );

    return (
      <div className={ styles.wrapper }>
        <IdentityIcon
          className={ styles.icon }
          address={ addressToRender }
          inline
        />
        <AutoComplete
          floatingLabelText={ label }
          hintText={ hint || 'enter a name or an address' }
          errorText={ error }
          dataSource={ choices }
          filter={ this.filter }
          onNewRequest={ this.onNewRequest }
          onUpdateInput={ this.onUpdateInput }
          maxSearchResults={ maxSearchResults || 8 }
          fullWidth openOnFocus floatingLabelFixed
        />
      </div>
    );
  }

  renderChoice = (data) => {
    const icon = (<IdentityIcon address={ data.address } inline />);
    // TODO move those styles down there to a better place
    return (
      <MenuItem
        primaryText={ data.name }
        key={ data.address }
        leftIcon={ icon }
        innerDivStyle={ {
          display: 'flex', alignItems: 'center',
          paddingLeft: '1em', paddingRight: '1em'
        } }
      />
    );
  }

  updateChoices = (nextProps) => {
    const { accounts, contacts } = nextProps || this.props;
    this.setState({ choices: this.computeChoices(accounts, contacts) });
  }

  computeChoices = (accounts, contacts) => {
    return Object.values(Object.assign({}, contacts, accounts))
      .map((data) => ({
        tokens: normalize(data.name),
        value: this.renderChoice(data),
        text: data.name, data
      }))
      .filter(isNotDeleted)
      .sort(sortChoices);
  };

  onNewRequest = (choice) => {
    this.setState({ address: choice.data.address });
    this.props.onChange(null, choice.data.address);
  };

  filter = (query, _, choice) => {
    query = query.trim();

    const needle = normalize(query);
    return (choice.tokens.indexOf(needle) >= 0) ||
      (choice.data.address.slice(0, query.length).toLowerCase() === query);
  };

  onUpdateInput = (query, choices) => {
    query = query.trim();
    this.setState({ address: query });

    if (query.slice(0, 2) !== '0x' && util.isAddressValid('0x' + query)) {
      this.props.onChange(null, '0x' + query);
    } else {
      this.props.onChange(null, query);
    }
  };
}

function mapStateToProps (state) {
  const { accounts, contacts } = state.personal;

  return {
    accounts,
    contacts
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(InputAddressSelect);
