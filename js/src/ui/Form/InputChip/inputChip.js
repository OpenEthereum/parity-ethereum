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

import React, { Component, PropTypes } from 'react';
import ChipInput from 'material-ui-chip-input';
import { uniq } from 'lodash';

import { nodeOrStringProptype } from '@parity/shared/util/proptypes';

import Chip from './Chip';

export default class InputChip extends Component {
  static propTypes = {
    addOnBlur: PropTypes.bool,
    clearOnBlur: PropTypes.bool,
    className: PropTypes.string,
    hint: nodeOrStringProptype(),
    label: nodeOrStringProptype(),
    onTokensChange: PropTypes.func,
    onInputChange: PropTypes.func,
    onBlur: PropTypes.func,
    tokens: PropTypes.oneOfType([
      PropTypes.array,
      PropTypes.object
    ]).isRequired
  }

  static defaultProps = {
    clearOnBlur: false,
    addOnBlur: false
  }

  render () {
    const { clearOnBlur, className, hint, label, tokens } = this.props;

    return (
      <ChipInput
        className={ className }
        ref='chipInput'

        value={ tokens }
        clearOnBlur={ clearOnBlur }
        floatingLabelText={ label }
        hintText={ hint }

        chipRenderer={ this.chipRenderer }

        onBlur={ this.handleBlur }
        onRequestAdd={ this.handleTokenAdd }
        onRequestDelete={ this.handleTokenDelete }
        onUpdateInput={ this.handleInputChange }

        floatingLabelFixed
        fullWidth

        hintStyle={ {
          bottom: 13,
          left: 0,
          transition: 'none'
        } }
        inputStyle={ {
          marginBottom: 18,
          width: 'initial'
        } }
        underlineStyle={ {
          borderWidth: 2
        } }
      />
    );
  }

  chipRenderer = (state, key) => {
    const { isDisabled, isFocused, handleClick, handleRequestDelete, value } = state;

    return (
      <Chip
        isDisabled={ isDisabled }
        isFocused={ isFocused }
        key={ key }
        label={ value }
        onClick={ handleClick }
        onDelete={ handleRequestDelete }
      />
    );
  }

  handleBlur = () => {
    const { onBlur, addOnBlur } = this.props;

    if (addOnBlur) {
      const { inputValue } = this.refs.chipInput.state;

      this.handleTokenAdd(inputValue);
    }

    if (typeof onBlur === 'function') {
      onBlur();
    }
  }

  handleTokenAdd = (value) => {
    const { tokens, onInputChange } = this.props;
    const newTokens = uniq([].concat(tokens, value));

    this.handleTokensChange(newTokens);

    if (value === this.refs.chipInput.state.inputValue) {
      if (typeof onInputChange === 'function') {
        onInputChange('');
      }
      this.refs.chipInput.setState({ inputValue: '' });
    }
  }

  handleTokenDelete = (value) => {
    const { tokens } = this.props;

    const newTokens = uniq([]
      .concat(tokens)
      .filter(v => v !== value));

    this.handleTokensChange(newTokens);
    this.focus();
  }

  focus = () => {
    this.refs.chipInput.focus();
  }

  handleInputChange = (value) => {
    const { onInputChange } = this.props;
    const splitTokens = value.split(/[\s,;]/);
    const inputValue = (splitTokens.length <= 1)
      ? value
      : splitTokens.slice(-1)[0].trim();

    this.refs.chipInput.setState({ inputValue });

    if (splitTokens.length > 1) {
      const tokensToAdd = splitTokens.slice(0, -1);

      tokensToAdd.forEach(token => this.handleTokenAdd(token));
    }

    if (typeof onInputChange === 'function') {
      onInputChange(inputValue);
    }
  }

  handleTokensChange = (tokens) => {
    const { onTokensChange } = this.props;

    onTokensChange(tokens.filter(token => token && token.length > 0));
  }
}
