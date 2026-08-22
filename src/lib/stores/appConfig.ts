import { writable } from 'svelte/store';
import type { AppConfig } from '../types';

export const appConfig = writable<AppConfig>({
  version: 1,
  watchFolders: [],
  autoSwitchAccount: true,
  defaultEditor: 'vscode'
});

export const selectedAccountFilter = writable<string>('ALL');
export const isSettingsModalOpen = writable<boolean>(false);
