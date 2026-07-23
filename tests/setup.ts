import "@testing-library/jest-dom/vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { beforeEach, afterEach } from "vitest";

// jsdom does not implement ResizeObserver; @dnd-kit/dom (used by PlaylistView's
// drag-and-drop) reads it at module-eval time, so a minimal stub is required
// for any test that imports a component tree including PlaylistView/App.
class ResizeObserverStub {
  observe() {}
  unobserve() {}
  disconnect() {}
}
// @ts-expect-error jsdom has no ResizeObserver typing
globalThis.ResizeObserver ??= ResizeObserverStub;

beforeEach(() => {
  mockIPC(() => undefined);
});

afterEach(() => {
  clearMocks();
});
