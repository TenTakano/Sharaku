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
    if (cmd === "validate_template") return undefined;
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

    it("はじめるボタンでモード選択ステップに進む", async () => {
      const user = userEvent.setup();
      render(SetupView, createProps());

      await user.click(screen.getByText("はじめる"));

      expect(
        screen.getByText("リソース管理モードを選択してください"),
      ).toBeInTheDocument();
    });
  });

  describe("モード選択ステップ", () => {
    it("モード選択が表示される", () => {
      render(SetupView, createProps({ initialStep: "mode-select" }));
      expect(screen.getByText("すべて管理")).toBeInTheDocument();
      expect(screen.getByText("メタデータのみ管理")).toBeInTheDocument();
    });

    it("すべて管理を選択して次へでフォルダ選択ステップに進む", async () => {
      const user = userEvent.setup();
      render(SetupView, createProps({ initialStep: "mode-select" }));

      await user.click(screen.getByText("次へ"));

      expect(
        screen.getByText("ライブラリのディレクトリを選択してください"),
      ).toBeInTheDocument();
    });

    it("メタデータのみ管理を選択して次へでライブラリ名ステップに進む", async () => {
      const user = userEvent.setup();
      const { container } = render(
        SetupView,
        createProps({ initialStep: "mode-select" }),
      );

      const radios = container.querySelectorAll<HTMLInputElement>(
        'input[name="setup-resource-mode"]',
      );
      radios[1].dispatchEvent(new Event("change", { bubbles: true }));

      await user.click(screen.getByText("次へ"));

      await waitFor(() => {
        expect(
          screen.getByText("ライブラリ名を入力してください"),
        ).toBeInTheDocument();
      });
    });

    it("onCancel がある場合にキャンセルボタンが表示される", () => {
      render(
        SetupView,
        createProps({ initialStep: "mode-select", onCancel: vi.fn() }),
      );
      expect(screen.getByText("キャンセル")).toBeInTheDocument();
    });
  });

  describe("ライブラリ名ステップ（メタデータのみ）", () => {
    async function goToNameOnlyStep() {
      const user = userEvent.setup();
      const { container } = render(
        SetupView,
        createProps({ initialStep: "mode-select" }),
      );

      const radios = container.querySelectorAll<HTMLInputElement>(
        'input[name="setup-resource-mode"]',
      );
      radios[1].dispatchEvent(new Event("change", { bubbles: true }));
      await user.click(screen.getByText("次へ"));

      await waitFor(() => {
        expect(
          screen.getByText("ライブラリ名を入力してください"),
        ).toBeInTheDocument();
      });
      return user;
    }

    it("ライブラリ名入力フィールドが表示される", async () => {
      await goToNameOnlyStep();
      expect(screen.getByPlaceholderText("マイライブラリ")).toBeInTheDocument();
    });

    it("ライブラリ名が空の場合は作成ボタンが無効", async () => {
      await goToNameOnlyStep();
      expect(screen.getByText("作成")).toBeDisabled();
    });

    it("作成ボタンで create_library が正しい引数で呼ばれ onComplete が発火する", async () => {
      const createLibrarySpy = vi.fn(() => ({
        id: "lib-1",
        name: "テストライブラリ",
        path: "/path",
      }));
      cleanup();
      mockIPC((cmd: string, args: Record<string, unknown>) => {
        if (cmd === "validate_template") return undefined;
        if (cmd === "create_library") return createLibrarySpy(args);
        if (cmd === "plugin:dialog|open") return "/selected/path";
      });

      const user = userEvent.setup();
      const props = createProps({ initialStep: "mode-select" });
      const { container } = render(SetupView, props);
      const radios = container.querySelectorAll<HTMLInputElement>(
        'input[name="setup-resource-mode"]',
      );
      radios[1].dispatchEvent(new Event("change", { bubbles: true }));
      await user.click(screen.getByText("次へ"));

      await waitFor(() => {
        expect(
          screen.getByPlaceholderText("マイライブラリ"),
        ).toBeInTheDocument();
      });

      await user.type(screen.getByPlaceholderText("マイライブラリ"), "テスト");
      await user.click(screen.getByText("作成"));

      await waitFor(() => {
        expect(createLibrarySpy).toHaveBeenCalledWith(
          expect.objectContaining({ name: "テスト" }),
        );
        expect(props.onComplete).toHaveBeenCalled();
      });
    });
  });
});
