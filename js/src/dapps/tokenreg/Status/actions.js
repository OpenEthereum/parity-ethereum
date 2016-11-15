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

import {
  registry as registryAbi,
  githubhint as githubhintAbi
} from '../../../contracts/abi';

import Contracts from '../../../contracts';

import { loadToken, setTokenPending, deleteToken, setTokenData } from '../Tokens/actions';

const { api } = window.parity;

export const SET_LOADING = 'SET_LOADING';
export const setLoading = (isLoading) => ({
  type: SET_LOADING,
  isLoading
});

export const FIND_CONTRACT = 'FIND_CONTRACT';
export const loadContract = () => (dispatch) => {
  dispatch(setLoading(true));

  const { tokenReg } = new Contracts(api);

  api.parity
    .registryAddress()
    .then((registryAddress) => {
      console.log(`registry found at ${registryAddress}`);
      const registry = api.newContract(registryAbi, registryAddress).instance;

      return Promise.all([
        tokenReg.getInstance(),
        registry.getAddress.call({}, [api.util.sha3('githubhint'), 'A'])
      ]);
    })
    .then(([ tokenRegInstance, githubhintAddress ]) => {
      const githubhintContract = api
        .newContract(githubhintAbi, githubhintAddress);

      dispatch(setContractDetails({
        address: tokenRegInstance.address,
        instance: tokenRegInstance
      }));

      dispatch(setGithubhintDetails({
        address: githubhintAddress,
        instance: githubhintContract.instance
      }));

      dispatch(loadContractDetails());
      dispatch(subscribeEvents());
    })
    .catch((error) => {
      throw error;
    });
};

export const LOAD_CONTRACT_DETAILS = 'LOAD_CONTRACT_DETAILS';
export const loadContractDetails = () => (dispatch, getState) => {
  const state = getState();

  const { instance } = state.status.contract;

  Promise
    .all([
      api.eth.accounts(),
      instance.owner.call(),
      instance.fee.call()
    ])
    .then(([accounts, owner, fee]) => {
      console.log(`owner as ${owner}, fee set at ${fee.toFormat()}`);

      const isOwner = accounts.filter(a => a === owner).length > 0;

      dispatch(setContractDetails({
        fee,
        owner,
        isOwner
      }));

      dispatch(setLoading(false));
    })
    .catch((error) => {
      console.error('loadContractDetails error', error);
    });
};

export const SET_CONTRACT_DETAILS = 'SET_CONTRACT_DETAILS';
export const setContractDetails = (details) => ({
  type: SET_CONTRACT_DETAILS,
  details
});

export const SET_GITHUBHINT_CONTRACT = 'SET_GITHUBHINT_CONTRACT';
export const setGithubhintDetails = (details) => ({
  type: SET_GITHUBHINT_CONTRACT,
  details
});

export const subscribeEvents = () => (dispatch, getState) => {
  const state = getState();

  const { instance } = state.status.contract;
  const previousSubscriptionId = state.status.subscriptionId;

  if (previousSubscriptionId) {
    instance.unsubscribe(previousSubscriptionId);
  }

  instance
    .subscribe(null, {
      fromBlock: 'latest',
      toBlock: 'pending',
      limit: 50
    }, (error, logs) => {
      if (error) {
        console.error('setupFilters', error);
        return;
      }

      if (!logs || logs.length === 0) return;

      logs.forEach(log => {
        const event = log.event;
        const type = log.type;
        const params = log.params;

        if (event === 'Registered' && type === 'pending') {
          return dispatch(setTokenData(params.id.toNumber(), {
            tla: '...',
            base: -1,
            address: params.addr.value,
            name: params.name.value,
            isPending: true
          }));
        }

        if (event === 'Registered' && type === 'mined') {
          return dispatch(loadToken(params.id.value.toNumber()));
        }

        if (event === 'Unregistered' && type === 'pending') {
          return dispatch(setTokenPending(params.id.value.toNumber(), true));
        }

        if (event === 'Unregistered' && type === 'mined') {
          return dispatch(deleteToken(params.id.value.toNumber()));
        }

        if (event === 'MetaChanged' && type === 'pending') {
          return dispatch(setTokenData(
            params.id.value.toNumber(),
            { metaPending: true, metaMined: false }
          ));
        }

        if (event === 'MetaChanged' && type === 'mined') {
          setTimeout(() => {
            dispatch(setTokenData(
              params.id.value.toNumber(),
              { metaPending: false, metaMined: false }
            ));
          }, 5000);

          return dispatch(setTokenData(
            params.id.value.toNumber(),
            { metaPending: false, metaMined: true }
          ));
        }

        console.log('new log event', log);
      });
    })
    .then((subscriptionId) => {
      dispatch(setSubscriptionId(subscriptionId));
    });
};

export const SET_SUBSCRIPTION_ID = 'SET_SUBSCRIPTION_ID';
export const setSubscriptionId = subscriptionId => ({
  type: SET_SUBSCRIPTION_ID,
  subscriptionId
});
