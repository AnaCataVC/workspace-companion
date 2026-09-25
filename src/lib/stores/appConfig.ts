import { writable } from 'svelte/store';
import type { AppConfig, ViewDensity, StatusFilterType } from '../types';

export const appConfig = writable<AppConfig>({
  version: 1,
  watchFolders: [],
  autoSwitchAccount: true,
  defaultEditor: 'vscode',
  defaultTerminal: 'wt',
  showTerminalButton: true,
  isPinned: false
});

export const selectedAccountFilter = writable<string>('ALL');
export const selectedStatusFilter = writable<StatusFilterType>('ALL');

const initialDensity = (typeof localStorage !== 'undefined' && localStorage.getItem('workspace_view_density') as ViewDensity) || 'compact';
export const viewDensity = writable<ViewDensity>(initialDensity);

viewDensity.subscribe((val) => {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('workspace_view_density', val);
  }
});
