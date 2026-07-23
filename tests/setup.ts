import "@testing-library/jest-dom/vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { beforeEach, afterEach } from "vitest";

beforeEach(() => {
  mockIPC(() => undefined);
});

afterEach(() => {
  clearMocks();
});
