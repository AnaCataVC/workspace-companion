/** Case- and separator-insensitive form of a Windows path, without a trailing separator. */
export function normalizePath(path: string): string {
  return path.trim().replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase();
}

/**
 * Whether `path` is `root` itself or lies inside it. Matches on a separator boundary, so
 * `C:/Repos/app-v2` is not considered inside `C:/Repos/app`.
 */
export function isPathWithin(path: string, root: string): boolean {
  const normalizedPath = normalizePath(path);
  const normalizedRoot = normalizePath(root);
  if (!normalizedRoot) return false;
  return normalizedPath === normalizedRoot || normalizedPath.startsWith(`${normalizedRoot}/`);
}

/**
 * Returns the short display branch name, stripping the 'refs/heads/' prefix if present,
 * or '(detached HEAD)' if the branch is empty or null.
 */
export function getShortBranch(fullBranch: string | null | undefined): string {
  if (!fullBranch) return '(detached HEAD)';
  return fullBranch.replace(/^refs\/heads\//, '');
}

/**
 * Returns the directory name (the last path segment) for a given POSIX or Windows path.
 */
export function getDirectoryName(fullPath: string): string {
  if (!fullPath) return '';
  const parts = fullPath.replace(/\\/g, '/').replace(/\/+$/, '').split('/');
  return parts[parts.length - 1] || fullPath;
}
