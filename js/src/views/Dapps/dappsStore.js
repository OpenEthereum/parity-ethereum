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

import { action, computed, observable, transaction } from 'mobx';
import store from 'store';

import Contracts from '~/contracts';
import DappsFetcher from './dappsFetcher';

const LS_KEY_DISPLAY = 'displayApps';
const LS_KEY_EXTERNAL_ACCEPT = 'acceptExternal';

export default class DappsStore {
  @observable apps = [];
  @observable displayApps = {};
  @observable modalOpen = false;
  @observable externalOverlayVisible = true;

  dappsFetcher = null;

  constructor (api) {
    this.loadExternalOverlay();
    this.readDisplayApps();

    this.dappsFetcher = DappsFetcher.get(api);

    Promise
      .all([
        this._fetchBuiltinApps(),
        this._fetchLocalApps(),
        this._fetchRegistryApps()
      ])
      .then(this.writeDisplayApps);
  }

  _fetchBuiltinApps () {
    this.dappsFetcher
      .fetchBuiltinApps()
      .then((apps) => this.addApps(apps));
  }

  _fetchLocalApps () {
    this.dappsFetcher
      .fetchLocalApps()
      .then((apps) => this.addApps(apps));
  }

  _fetchRegistryApps () {
    const { dappReg } = Contracts.get();

    this.dappsFetcher
      .fetchRegistryAppIds()
      .then((appIds) => {
        appIds.forEach((appId) => {
          // Fetch the Dapp and display it ASAP
          this.dappsFetcher
            .fetchRegistryApp(dappReg, appId)
            .then((app) => {
              if (app) {
                this.addApps([ app ]);
              }
            });
        });
      });
  }

  @computed get sortedBuiltin () {
    return this.apps.filter((app) => app.type === 'builtin');
  }

  @computed get sortedLocal () {
    return this.apps.filter((app) => app.type === 'local');
  }

  @computed get sortedNetwork () {
    return this.apps.filter((app) => app.type === 'network');
  }

  @computed get visibleApps () {
    return this.apps.filter((app) => this.displayApps[app.id] && this.displayApps[app.id].visible);
  }

  @computed get visibleBuiltin () {
    return this.visibleApps.filter((app) => app.type === 'builtin');
  }

  @computed get visibleLocal () {
    return this.visibleApps.filter((app) => app.type === 'local');
  }

  @computed get visibleNetwork () {
    return this.visibleApps.filter((app) => app.type === 'network');
  }

  @action openModal = () => {
    this.modalOpen = true;
  }

  @action closeModal = () => {
    this.modalOpen = false;
  }

  @action closeExternalOverlay = () => {
    this.externalOverlayVisible = false;
    store.set(LS_KEY_EXTERNAL_ACCEPT, true);
  }

  @action loadExternalOverlay () {
    this.externalOverlayVisible = !(store.get(LS_KEY_EXTERNAL_ACCEPT) || false);
  }

  @action hideApp = (id) => {
    this.displayApps = Object.assign({}, this.displayApps, { [id]: { visible: false } });
    this.writeDisplayApps();
  }

  @action showApp = (id) => {
    this.displayApps = Object.assign({}, this.displayApps, { [id]: { visible: true } });
    this.writeDisplayApps();
  }

  @action readDisplayApps = () => {
    this.displayApps = store.get(LS_KEY_DISPLAY) || {};
  }

  @action writeDisplayApps = () => {
    store.set(LS_KEY_DISPLAY, this.displayApps);
  }

  @action addApps = (apps) => {
    transaction(() => {
      // Get new apps IDs if available
      const newAppsIds = apps.map((app) => app.id).filter((id) => id);

      this.apps = this.apps
        .filter((app) => !app.id || !newAppsIds.includes(app.id))
        .concat(apps || [])
        .sort((a, b) => a.name.localeCompare(b.name));

      const visibility = {};
      apps.forEach((app) => {
        if (!this.displayApps[app.id]) {
          visibility[app.id] = { visible: app.visible };
        }
      });

      this.displayApps = Object.assign({}, this.displayApps, visibility);
    });
  }
}
