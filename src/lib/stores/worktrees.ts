import { writable } from 'svelte/store';
import type { RepositoryWorktrees } from '../types';

export const scannedRepos = writable<RepositoryWorktrees[]>([]);
export const isScanning = writable<boolean>(false);
export const searchFilter = writable<string>('');
export const isPinned = writable<boolean>(false);
export const scanError = writable<string | null>(null);
