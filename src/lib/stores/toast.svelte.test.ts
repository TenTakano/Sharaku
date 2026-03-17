import { describe, it, expect, vi, beforeEach } from "vitest";
import { addToast, removeToast, getToasts } from "./toast.svelte";

beforeEach(() => {
  for (const toast of getToasts()) {
    removeToast(toast.id);
  }
  vi.restoreAllMocks();
});

describe("toast store", () => {
  it("addToast でトーストが追加される", () => {
    addToast("success", "テストメッセージ");
    const toasts = getToasts();
    expect(toasts).toHaveLength(1);
    expect(toasts[0].type).toBe("success");
    expect(toasts[0].message).toBe("テストメッセージ");
  });

  it("removeToast で指定IDのトーストが削除される", () => {
    addToast("success", "メッセージ1");
    addToast("error", "メッセージ2");
    const id = getToasts()[0].id;
    removeToast(id);
    const toasts = getToasts();
    expect(toasts).toHaveLength(1);
    expect(toasts[0].message).toBe("メッセージ2");
  });

  it("指定 duration 後に自動削除される", () => {
    vi.useFakeTimers();
    addToast("success", "自動削除", 1000);
    expect(getToasts()).toHaveLength(1);
    vi.advanceTimersByTime(1000);
    expect(getToasts()).toHaveLength(0);
    vi.useRealTimers();
  });

  it("複数のトーストが独立して管理される", () => {
    vi.useFakeTimers();
    addToast("success", "短い", 1000);
    addToast("error", "長い", 5000);
    expect(getToasts()).toHaveLength(2);
    vi.advanceTimersByTime(1000);
    expect(getToasts()).toHaveLength(1);
    expect(getToasts()[0].message).toBe("長い");
    vi.advanceTimersByTime(4000);
    expect(getToasts()).toHaveLength(0);
    vi.useRealTimers();
  });

  it("デフォルト duration は 3000ms", () => {
    vi.useFakeTimers();
    addToast("success", "デフォルト");
    expect(getToasts()).toHaveLength(1);
    vi.advanceTimersByTime(2999);
    expect(getToasts()).toHaveLength(1);
    vi.advanceTimersByTime(1);
    expect(getToasts()).toHaveLength(0);
    vi.useRealTimers();
  });
});
