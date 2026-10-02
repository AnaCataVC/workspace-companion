import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type {
  QuickActionPreset,
  QuickActionScope,
  QuickActionPreview,
  QuickActionResult
} from '../types';
import { toErrorMessage } from '../utils/errors';

export const isQuickActionsModalOpen = writable<boolean>(false);
export const activeQuickActionPreset = writable<QuickActionPreset>('discardUncommitted');
export const quickActionScope = writable<QuickActionScope>('allWatchedRepos');
export const quickActionPreview = writable<QuickActionPreview | null>(null);
export const isLoadingQuickActionPreview = writable<boolean>(false);
export const quickActionPreviewError = writable<string | null>(null);
export const isExecutingQuickAction = writable<boolean>(false);
export const quickActionResult = writable<QuickActionResult | null>(null);
export const quickActionError = writable<string | null>(null);

/**
 * Fetches pre-flight impact preview for the selected preset and scope.
 */
export async function fetchQuickActionPreview(
  preset: QuickActionPreset,
  scope: QuickActionScope
): Promise<void> {
  isLoadingQuickActionPreview.set(true);
  quickActionPreviewError.set(null);

  try {
    const preview = await invoke<QuickActionPreview>('get_quick_action_preview', {
      preset,
      scope
    });
    quickActionPreview.set(preview);
  } catch (err: unknown) {
    quickActionPreviewError.set(toErrorMessage(err, 'Failed to compute action preview'));
  } finally {
    isLoadingQuickActionPreview.set(false);
  }
}

/**
 * Executes the quick action bulk cleanup pipeline.
 */
export async function runQuickAction(
  preset: QuickActionPreset,
  scope: QuickActionScope
): Promise<QuickActionResult | null> {
  isExecutingQuickAction.set(true);
  quickActionError.set(null);
  quickActionResult.set(null);

  try {
    const result = await invoke<QuickActionResult>('execute_quick_action', {
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
  quickActionPreview.set(null);
  quickActionPreviewError.set(null);
  quickActionResult.set(null);
  quickActionError.set(null);
}
