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

import { shallow } from 'enzyme';
import React from 'react';
import sinon from 'sinon';

import ParityBar from './';

let component;
let instance;
let store;

function createRedux (state = {}) {
  store = {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => Object.assign({ signer: { pending: [] } }, state)
  };

  return store;
}

function render (props = {}, state = {}) {
  component = shallow(
    <ParityBar { ...props } />,
    { context: { store: createRedux(state) } }
  ).find('ParityBar').shallow();
  instance = component.instance();

  return component;
}

describe('views/ParityBar', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  describe('renderBar', () => {
    let bar;

    beforeEach(() => {
      render({ dapp: true });
      bar = shallow(instance.renderBar());
    });

    it('renders nothing when not overlaying a dapp', () => {
      render({ dapp: false });
      expect(instance.renderBar()).to.be.null;
    });

    it('renders when overlaying a dapp', () => {
      expect(bar.find('div')).not.to.have.length(0);
    });

    it('includes the ParityBackground', () => {
      expect(bar.find('Connect(ParityBackground)')).to.have.length(1);
    });

    it('renders the Parity button', () => {
      const label = shallow(bar.find('Button').first().props().label);
      expect(label.find('FormattedMessage').props().id).to.equal('parityBar.label.parity');
    });

    it('renders the Signer button', () => {
      const label = shallow(bar.find('Button').last().props().label);

      expect(label.find('FormattedMessage').props().id).to.equal('parityBar.label.signer');
    });
  });

  describe('renderExpanded', () => {
    let expanded;

    beforeEach(() => {
      render();
      expanded = shallow(instance.renderExpanded());
    });

    it('includes the ParityBackground', () => {
      expect(expanded.find('Connect(ParityBackground)')).to.have.length(1);
    });

    it('includes the Signer', () => {
      expect(expanded.find('Connect(Embedded)')).to.have.length(1);
    });
  });

  describe('renderLabel', () => {
    beforeEach(() => {
      render();
    });

    it('renders the label name', () => {
      expect(shallow(instance.renderLabel('testing', null)).text()).to.equal('testing');
    });

    it('renders name and bubble', () => {
      expect(shallow(instance.renderLabel('testing', '(bubble)')).text()).to.equal('testing(bubble)');
    });
  });

  describe('renderSignerLabel', () => {
    let label;

    beforeEach(() => {
      render();
      label = shallow(instance.renderSignerLabel());
    });

    it('renders the signer label', () => {
      expect(label.find('FormattedMessage').props().id).to.equal('parityBar.label.signer');
    });

    it('does not render a badge when no pending requests', () => {
      expect(label.find('Badge')).to.have.length(0);
    });

    it('renders a badge when pending requests', () => {
      render({}, { signer: { pending: ['123', '456'] } });
      expect(shallow(instance.renderSignerLabel()).find('Badge').props().value).to.equal(2);
    });
  });

  describe('opened state', () => {
    beforeEach(() => {
      render({ dapp: true });

      sinon.spy(instance, 'renderBar');
      sinon.spy(instance, 'renderExpanded');
    });

    afterEach(() => {
      instance.renderBar.restore();
      instance.renderExpanded.restore();
    });

    it('renders the bar on with opened === false', () => {
      expect(component.find('Link[to="/apps"]')).to.have.length(1);
    });

    it('renders expanded with opened === true', () => {
      expect(instance.renderExpanded).not.to.have.been.called;
      instance.setState({ opened: true });
      expect(instance.renderExpanded).to.have.been.called;
    });
  });
});
