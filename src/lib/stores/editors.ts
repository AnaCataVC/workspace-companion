import { writable } from 'svelte/store';
import type { EditorInfo, SupportedEditor, TerminalInfo, SupportedTerminal } from '../types';

export const installedEditors = writable<EditorInfo[]>([
  { id: 'vscode', name: 'VS Code', isAvailable: true, iconName: 'code' },
  { id: 'antigravity', name: 'Antigravity IDE', isAvailable: true, iconName: 'bot' },
  { id: 'cursor', name: 'Cursor', isAvailable: false, iconName: 'sparkles' },
  { id: 'windsurf', name: 'Windsurf', isAvailable: false, iconName: 'wind' },
  { id: 'explorer', name: 'File Explorer', isAvailable: true, iconName: 'folder' }
]);

export const installedTerminals = writable<TerminalInfo[]>([
  { id: 'wt', name: 'Windows Terminal (wt)', isAvailable: true, iconName: 'terminal' },
  { id: 'powershell', name: 'PowerShell', isAvailable: true, iconName: 'terminal' },
  { id: 'cmd', name: 'Command Prompt (CMD)', isAvailable: true, iconName: 'terminal' },
  { id: 'git-bash', name: 'Git Bash', isAvailable: true, iconName: 'terminal' },
  { id: 'agy', name: 'AGY CLI (Antigravity)', isAvailable: true, iconName: 'bot' },
  { id: 'none', name: 'Disabled (No Terminal Button)', isAvailable: true, iconName: 'x' }
]);

export function isEditorAvailable(editors: EditorInfo[], id: SupportedEditor): boolean {
  const match = editors.find(e => e.id === id);
  return match ? match.isAvailable : false;
}
