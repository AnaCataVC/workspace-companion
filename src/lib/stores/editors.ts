import { writable } from 'svelte/store';
import type { EditorInfo, SupportedEditor } from '../types';

export const installedEditors = writable<EditorInfo[]>([
  { id: 'antigravity', name: 'Antigravity', isAvailable: true, iconName: 'sparkles' },
  { id: 'vscode', name: 'VS Code', isAvailable: true, iconName: 'code' },
  { id: 'cursor', name: 'Cursor', isAvailable: false, iconName: 'sparkles' },
  { id: 'windsurf', name: 'Windsurf', isAvailable: false, iconName: 'wind' },
  { id: 'wt', name: 'Windows Terminal', isAvailable: true, iconName: 'terminal' },
  { id: 'explorer', name: 'File Explorer', isAvailable: true, iconName: 'folder' }
]);

export function isEditorAvailable(editors: EditorInfo[], id: SupportedEditor): boolean {
  const match = editors.find(e => e.id === id);
  return match ? match.isAvailable : false;
}
