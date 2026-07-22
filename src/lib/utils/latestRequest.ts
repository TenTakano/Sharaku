export interface LatestRequestGuard {
  next(): number;
  isLatest(id: number): boolean;
}

/**
 * 非同期リクエストを連投したとき、最後に発行したリクエストの結果だけを
 * 反映するためのガード。next() で取得したIDをレスポンス受信時に
 * isLatest() で照合し、古いレスポンスの反映を防ぐ。
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
