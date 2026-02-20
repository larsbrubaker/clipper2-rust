const STORAGE_PREFIX = 'clipper2.demo.state.';

function getStorageKey(demoId: string): string {
  return `${STORAGE_PREFIX}${demoId}`;
}

export function loadDemoState<T>(demoId: string, defaults: T): T {
  try {
    const raw = window.localStorage.getItem(getStorageKey(demoId));
    if (!raw) return defaults;
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== 'object') return defaults;
    return { ...defaults, ...parsed };
  } catch {
    return defaults;
  }
}

export function saveDemoState<T>(demoId: string, state: T): void {
  try {
    window.localStorage.setItem(getStorageKey(demoId), JSON.stringify(state));
  } catch {
    // Ignore storage errors (private mode, quota exceeded, etc).
  }
}
