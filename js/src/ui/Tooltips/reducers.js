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

const LS_KEY = 'tooltips';

let currentId = -1;
let maxId = 0;

function closeTooltips (state, action) {
  window.localStorage.setItem(LS_KEY, '{"state":"off"}');

  currentId = -1;

  return Object.assign({}, state, {
    currentId, hide: true
  });
}

function newTooltip (state, action) {
  const { newId } = action;

  maxId = Math.max(newId, maxId);

  return Object.assign({}, state, {
    currentId,
    maxId
  });
}

function nextTooltip (state, action) {
  const hideTips = window.localStorage.getItem(LS_KEY);

  currentId = hideTips
    ? -1
    : currentId + 1;

  return Object.assign({}, state, {
    currentId
  });
}

function loadTooltips (state) {
  const hideTips = window.localStorage.getItem(LS_KEY);
  return hideTips ? { ...state, hide: true } : state;
}

export default function tooltipReducer (state = {}, action) {
  switch (action.type) {
    case 'newTooltip':
      return newTooltip(state, action);

    case 'nextTooltip':
      return nextTooltip(state, action);

    case 'closeTooltips':
      return closeTooltips(state, action);

    case 'loadTooltips':
      return loadTooltips(state);

    default:
      return state;
  }
}
