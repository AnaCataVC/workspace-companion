import { writable } from 'svelte/store';

export type NotificationType = 'error' | 'success' | 'info' | 'warning';

export interface NotificationItem {
  id: string;
  type: NotificationType;
  title: string;
  message?: string;
  durationMs?: number;
}

function createNotificationStore() {
  const { subscribe, update } = writable<NotificationItem[]>([]);

  function add(type: NotificationType, title: string, message?: string, durationMs: number = 4500) {
    const id = Math.random().toString(36).substring(2, 9);
    const item: NotificationItem = { id, type, title, message, durationMs };

    update((items) => [...items, item]);

    if (durationMs > 0) {
      setTimeout(() => {
        dismiss(id);
      }, durationMs);
    }

    return id;
  }

  function dismiss(id: string) {
    update((items) => items.filter((item) => item.id !== id));
  }

  return {
    subscribe,
    error: (title: string, message?: string, durationMs: number = 6000) =>
      add('error', title, message, durationMs),
    success: (title: string, message?: string, durationMs: number = 4000) =>
      add('success', title, message, durationMs),
    info: (title: string, message?: string, durationMs: number = 4500) =>
      add('info', title, message, durationMs),
    warning: (title: string, message?: string, durationMs: number = 5000) =>
      add('warning', title, message, durationMs),
    dismiss
  };
}

export const notifications = createNotificationStore();
