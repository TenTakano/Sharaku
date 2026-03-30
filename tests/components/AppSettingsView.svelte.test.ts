import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import { mockIPC } from "@tauri-apps/api/mocks";
import AppSettingsView from "../../src/lib/components/AppSettingsView.svelte";
import { getToasts, removeToast } from "../../src/lib/stores/toast.svelte";

beforeEach(() => {
  cleanup();
  for (const toast of getToasts()) {
    removeToast(toast.id);
  }
  mockIPC((cmd: string) => {
    if (cmd === "set_theme") return undefined;
    if (cmd === "set_banner_auto_close") return undefined;
  });
});

describe("AppSettingsView コンポーネント", () => {
  it("テーマセクションが表示される", () => {
    render(AppSettingsView);
    expect(screen.getByText("テーマ")).toBeInTheDocument();
    expect(screen.getByText("OS設定に従う")).toBeInTheDocument();
    expect(screen.getByText("ライト")).toBeInTheDocument();
    expect(screen.getByText("ダーク")).toBeInTheDocument();
  });

  it("バナー設定セクションが表示される", () => {
    render(AppSettingsView);
    expect(screen.getByText("取り込みバナー")).toBeInTheDocument();
    expect(screen.getByText("1秒")).toBeInTheDocument();
    expect(screen.getByText("3秒")).toBeInTheDocument();
    expect(screen.getByText("5秒")).toBeInTheDocument();
    expect(screen.getByText("手動で閉じる")).toBeInTheDocument();
  });

  it("テーマ変更でラジオボタンが切り替わる", async () => {
    const { container } = render(AppSettingsView);

    const radios = container.querySelectorAll<HTMLInputElement>(
      'input[name="theme-mode"]',
    );
    const lightRadio = radios[1];

    lightRadio.dispatchEvent(new Event("change", { bubbles: true }));

    await waitFor(() => {
      expect(lightRadio.checked).toBe(true);
    });
  });

  it("テーマ変更失敗時にエラートーストが表示される", async () => {
    mockIPC((cmd: string) => {
      if (cmd === "set_theme") throw new Error("backend error");
    });
    const { container } = render(AppSettingsView);

    const radios = container.querySelectorAll<HTMLInputElement>(
      'input[name="theme-mode"]',
    );
    radios[1].dispatchEvent(new Event("change", { bubbles: true }));

    await waitFor(() => {
      const toasts = getToasts();
      expect(
        toasts.some((t) => t.message.includes("テーマの変更に失敗しました")),
      ).toBe(true);
    });
  });

  it("バナー自動閉じ設定が選択できる", async () => {
    const { container } = render(AppSettingsView);

    const radios = container.querySelectorAll<HTMLInputElement>(
      'input[name="banner-auto-close"]',
    );
    const fiveSecRadio = radios[2];

    fiveSecRadio.dispatchEvent(new Event("change", { bubbles: true }));

    await waitFor(() => {
      expect(fiveSecRadio.checked).toBe(true);
    });
  });
});
