import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import SetupView from "../../src/lib/components/SetupView.svelte";

function createProps(overrides = {}) {
  return {
    onComplete: vi.fn(),
    ...overrides,
  };
}

beforeEach(() => {
  cleanup();
  mockIPC((cmd: string) => {
    if (cmd === "create_library")
      return { id: "lib-1", name: "テストライブラリ", path: "/path" };
    if (cmd === "plugin:dialog|open") return "/selected/path";
  });
});

describe("SetupView コンポーネント", () => {
  describe("ウェルカムステップ", () => {
    it("タイトルとサブタイトルが表示される", () => {
      render(SetupView, createProps());
      expect(screen.getByText("Sharaku")).toBeInTheDocument();
      expect(
        screen.getByText("画像ライブラリ管理ツールへようこそ"),
      ).toBeInTheDocument();
    });

    it("はじめるボタンでディレクトリ選択ステップに進む", async () => {
      const user = userEvent.setup();
      render(SetupView, createProps());

      await user.click(screen.getByText("はじめる"));

      expect(
        screen.getByText("ライブラリのディレクトリを選択してください"),
      ).toBeInTheDocument();
    });
  });

  describe("ディレクトリ選択ステップ", () => {
    it("onCancel がある場合にキャンセルボタンが表示される", () => {
      render(
        SetupView,
        createProps({ initialStep: "select", onCancel: vi.fn() }),
      );
      expect(screen.getByText("キャンセル")).toBeInTheDocument();
    });

    it("フォルダを選択するとパスとライブラリ名入力が表示される", async () => {
      const user = userEvent.setup();
      render(SetupView, createProps({ initialStep: "select" }));

      await user.click(screen.getByText("フォルダを選択"));

      await waitFor(() => {
        expect(screen.getByText("/selected/path")).toBeInTheDocument();
        expect(
          screen.getByPlaceholderText("マイライブラリ"),
        ).toBeInTheDocument();
      });
    });

    it("フォルダを設定せずに続けるとライブラリ名入力が表示される", async () => {
      const user = userEvent.setup();
      render(SetupView, createProps({ initialStep: "select" }));

      await user.click(screen.getByText("フォルダを設定せずに続ける"));

      expect(screen.getByPlaceholderText("マイライブラリ")).toBeInTheDocument();
    });

    it("ライブラリ名が空の場合は作成ボタンが無効", async () => {
      const user = userEvent.setup();
      render(SetupView, createProps({ initialStep: "select" }));

      await user.click(screen.getByText("フォルダを設定せずに続ける"));

      expect(screen.getByText("作成")).toBeDisabled();
    });

    it("作成ボタンで create_library が選択したパスで呼ばれ onComplete が発火する", async () => {
      const createLibrarySpy = vi.fn(() => ({
        id: "lib-1",
        name: "テストライブラリ",
        path: "/selected/path",
      }));
      cleanup();
      mockIPC((cmd: string, args: Record<string, unknown>) => {
        if (cmd === "create_library") return createLibrarySpy(args);
        if (cmd === "plugin:dialog|open") return "/selected/path";
      });

      const user = userEvent.setup();
      const props = createProps({ initialStep: "select" });
      render(SetupView, props);

      await user.click(screen.getByText("フォルダを選択"));

      await waitFor(() => {
        expect(
          screen.getByPlaceholderText("マイライブラリ"),
        ).toBeInTheDocument();
      });

      await user.click(screen.getByText("作成"));

      await waitFor(() => {
        expect(createLibrarySpy).toHaveBeenCalledWith(
          expect.objectContaining({ name: "path", path: "/selected/path" }),
        );
        expect(props.onComplete).toHaveBeenCalled();
      });
    });

    it("フォルダを設定せずに作成すると create_library が path: null で呼ばれる", async () => {
      const createLibrarySpy = vi.fn(() => ({
        id: "lib-1",
        name: "テスト",
        path: null,
      }));
      cleanup();
      mockIPC((cmd: string, args: Record<string, unknown>) => {
        if (cmd === "create_library") return createLibrarySpy(args);
        if (cmd === "plugin:dialog|open") return "/selected/path";
      });

      const user = userEvent.setup();
      const props = createProps({ initialStep: "select" });
      render(SetupView, props);

      await user.click(screen.getByText("フォルダを設定せずに続ける"));
      await user.type(screen.getByPlaceholderText("マイライブラリ"), "テスト");
      await user.click(screen.getByText("作成"));

      await waitFor(() => {
        expect(createLibrarySpy).toHaveBeenCalledWith(
          expect.objectContaining({ name: "テスト", path: null }),
        );
        expect(props.onComplete).toHaveBeenCalled();
      });
    });
  });
});
