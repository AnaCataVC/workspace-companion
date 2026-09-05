import type { Action } from 'svelte/action';

export interface CloseOnEscapeOptions {
  /** Whether Escape should currently trigger a close (e.g. the modal is open). */
  enabled: () => boolean;
  /** Called when Escape is pressed while enabled() returns true. */
  onClose: () => void;
}

/**
 * Action for <svelte:window> that closes a modal on Escape.
 * Centralizes the escape-to-close pattern shared by the app's modal components.
 */
export const closeOnEscape: Action<Window, CloseOnEscapeOptions> = (node, options) => {
  let current = options;

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape' && current.enabled()) {
      current.onClose();
    }
  }

  node.addEventListener('keydown', handleKeydown);

  return {
    update(newOptions: CloseOnEscapeOptions) {
      current = newOptions;
    },
    destroy() {
      node.removeEventListener('keydown', handleKeydown);
    }
  };
};
