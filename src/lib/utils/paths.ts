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
