/**
 * Safely converts an unknown error value (from a try/catch block or promise rejection)
 * into a descriptive, user-presentable error string.
 */
export function toErrorMessage(err: unknown, fallback: string = 'An unexpected error occurred'): string {
  if (typeof err === 'string') {
    const trimmed = err.trim();
    return trimmed || fallback;
  }

  if (err instanceof Error) {
    const trimmed = err.message.trim();
    return trimmed || fallback;
  }

  if (err && typeof err === 'object') {
    if ('message' in err && typeof (err as { message: unknown }).message === 'string') {
      const msg = (err as { message: string }).message.trim();
      if (msg) return msg;
    }
    try {
      const str = String(err);
      if (str && str !== '[object Object]') {
        return str;
      }
    } catch {
      // Fallback below
    }
  }

  return fallback;
}
