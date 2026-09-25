import type { Action } from 'svelte/action';

const FOCUSABLE_SELECTOR = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled]):not([type="hidden"])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])'
].join(',');

function focusableIn(node: HTMLElement): HTMLElement[] {
  return Array.from(node.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)).filter(
    (el) => el.offsetParent !== null || el === document.activeElement
  );
}

/**
 * Modal focus management: moves focus into the dialog on open (to the element marked
 * `data-autofocus`, else the first focusable one), keeps Tab / Shift+Tab cycling inside it, and
 * restores focus to whatever was focused before the dialog opened when it closes.
 * Escape handling stays in `closeOnEscape`.
 */
export const autofocus: Action<HTMLElement> = (node) => {
  const previouslyFocused = document.activeElement instanceof HTMLElement ? document.activeElement : null;

  // Deferred one frame so conditionally rendered children (and `disabled` states) have settled.
  const frame = requestAnimationFrame(() => {
    const preferred = node.querySelector<HTMLElement>('[data-autofocus]');
    (preferred ?? focusableIn(node)[0] ?? node).focus();
  });

  function handleKeydown(event: KeyboardEvent) {
    if (event.key !== 'Tab') return;
    const focusable = focusableIn(node);
    if (focusable.length === 0) {
      event.preventDefault();
      return;
    }
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    const active = document.activeElement;
    if (event.shiftKey && (active === first || !node.contains(active))) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && (active === last || !node.contains(active))) {
      event.preventDefault();
      first.focus();
    }
  }

  node.addEventListener('keydown', handleKeydown);
  if (!node.hasAttribute('tabindex')) node.setAttribute('tabindex', '-1');

  return {
    destroy() {
      cancelAnimationFrame(frame);
      node.removeEventListener('keydown', handleKeydown);
      if (previouslyFocused && document.contains(previouslyFocused)) previouslyFocused.focus();
    }
  };
};
