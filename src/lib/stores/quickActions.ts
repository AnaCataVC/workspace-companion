import { writable } from 'svelte/store';
import { invokeTauri } from '../utils/ipc';
import type {
  QuickActionPreset,
  QuickActionScope,
  QuickActionPreview,
  QuickActionResult
} from '../types';
import { toErrorMessage } from '../utils/errors';

export const isQuickActionsModalOpen = writable<boolean>(false);
export const activeQuickActionPreset = writable<QuickActionPreset>('syncDefaultBranch');
export const quickActionScope = writable<QuickActionScope>('allWatchedRepos');
export const quickActionPreview = writable<QuickActionPreview | null>(null);
export const isLoadingQuickActionPreview = writable<boolean>(false);
export const quickActionPreviewError = writable<string | null>(null);
export const isExecutingQuickAction = writable<boolean>(false);
export const quickActionResult = writable<QuickActionResult | null>(null);
export const quickActionError = writable<string | null>(null);

let previewRequestSeq = 0;

/**
 * Fetches pre-flight impact preview for the selected preset and scope, discarding stale responses
 * when the user switches preset or scope rapidly.
 */
export async function fetchQuickActionPreview(
  preset: QuickActionPreset,
  scope: QuickActionScope
): Promise<void> {
  const reqId = ++previewRequestSeq;
  isLoadingQuickActionPreview.set(true);
  quickActionPreviewError.set(null);

  try {
    const preview = await invokeTauri<QuickActionPreview>('get_quick_action_preview', {
      preset,
      scope
    });
    if (reqId !== previewRequestSeq) return;
    quickActionPreview.set(preview);
  } catch (err: unknown) {
    if (reqId !== previewRequestSeq) return;
    quickActionPreviewError.set(toErrorMessage(err, 'Failed to compute action preview'));
  } finally {
    if (reqId === previewRequestSeq) {
      isLoadingQuickActionPreview.set(false);
    }
  }
}

/**
 * Executes the quick action pipeline (constructive sync or bulk cleanup).
 */
export async function runQuickAction(
  preset: QuickActionPreset,
  scope: QuickActionScope
): Promise<QuickActionResult | null> {
  isExecutingQuickAction.set(true);
  quickActionError.set(null);
  quickActionResult.set(null);

  try {
    const result = await invokeTauri<QuickActionResult>('execute_quick_action', {
      preset,
      scope
    });
    quickActionResult.set(result);
    return result;
  } catch (err: unknown) {
    quickActionError.set(toErrorMessage(err, 'Failed to execute quick action'));
    return null;
  } finally {
    isExecutingQuickAction.set(false);
  }
}

export function resetQuickActionsState(): void {
  previewRequestSeq++;
  isLoadingQuickActionPreview.set(false);
  quickActionPreview.set(null);
  quickActionPreviewError.set(null);
  quickActionResult.set(null);
  quickActionError.set(null);
}

export interface OpenQuickActionsOptions {
  repoPath?: string;
  filteredRepoPaths?: string[];
  preset?: QuickActionPreset;
}

/**
 * Opens QuickActionsModal scoped to a single repository, the active filtered repository subset,
 * or all watched repositories.
 */
export function openQuickActionsModal(options?: string | OpenQuickActionsOptions): void {
  resetQuickActionsState();

  const normalized: OpenQuickActionsOptions =
    typeof options === 'string' ? { repoPath: options } : options ?? {};

  if (normalized.preset) {
    activeQuickActionPreset.set(normalized.preset);
  } else {
    activeQuickActionPreset.set('syncDefaultBranch');
  }

  if (normalized.repoPath) {
    quickActionScope.set({ singleRepo: { repoPath: normalized.repoPath } });
  } else if (normalized.filteredRepoPaths && normalized.filteredRepoPaths.length > 0) {
    quickActionScope.set({ repoList: { repoPaths: normalized.filteredRepoPaths } });
  } else {
    quickActionScope.set('allWatchedRepos');
  }

  isQuickActionsModalOpen.set(true);
}
