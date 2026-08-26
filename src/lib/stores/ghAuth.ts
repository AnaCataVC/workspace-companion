import { writable } from 'svelte/store';
import type { GhAccount } from '../types';

export const ghAccounts = writable<GhAccount[]>([]);
export const activeGhAccount = writable<string | null>(null);
export const isGhLoading = writable<boolean>(false);
