export interface LatestRequestGuard {
  next(): number;
  isLatest(id: number): boolean;
}

/**
 * Guard for applying only the result of the most recently issued request when
 * async requests fire in quick succession. Match the ID obtained from next()
 * against isLatest() upon receiving a response, to prevent stale responses
 * from being applied.
 */
export function createLatestRequestGuard(): LatestRequestGuard {
  let currentId = 0;
  return {
    next(): number {
      return ++currentId;
    },
    isLatest(id: number): boolean {
      return id === currentId;
    },
  };
}
