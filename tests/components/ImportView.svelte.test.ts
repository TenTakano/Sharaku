import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import ImportView from "../../src/lib/components/ImportView.svelte";
import { getToasts, removeToast } from "../../src/lib/stores/toast.svelte";
import type { AppSettings } from "../../src/lib/types";

const MOCK_SETTINGS: AppSettings = {
  resourceMode: "full",
  directoryTemplate: "{artist}/{title}",
  typeLabelImage: "Image",
  typeLabelFolder: "Folder",
  deleteFileAction: "ask",
};

function createProps(overrides = {}) {
  return {
    onBack: vi.fn(),
    ...overrides,
  };
}

beforeEach(() => {
  cleanup();
  for (const toast of getToasts()) {
    removeToast(toast.id);
  }
  mockIPC((cmd: string) => {
    if (cmd === "get_settings") return MOCK_SETTINGS;
    if (cmd === "parse_folder_name")
      return { title: "パースされたタイトル", artist: "パースアーティスト" };
    if (cmd === "preview_import_path")
      return "/library/パースアーティスト/パースされたタイトル";
    if (cmd === "enqueue_import") return { jobId: "job-1" };
    if (cmd === "plugin:dialog|open") return "/path/to/folder";
  });
});

describe("ImportView コンポーネント", () => {
  describe("選択ステップ", () => {
    it("フォルダ選択ボタンが表示される", () => {
      render(ImportView, createProps());
      expect(screen.getByText("フォルダを選択...")).toBeInTheDocument();
    });

    it("タイトルと説明が表示される", () => {
      render(ImportView, createProps());
      expect(screen.getByText("取り込み対象を選択")).toBeInTheDocument();
      expect(
        screen.getByText("取り込む画像フォルダを選択してください。"),
      ).toBeInTheDocument();
    });
  });

  describe("initialSourcePath でメタデータステップ", () => {
    it("initialSourcePath が指定されている場合、直接メタデータステップに進む", async () => {
      render(ImportView, createProps({ initialSourcePath: "/path/to/source" }));

      await waitFor(() => {
        expect(screen.getByText("メタデータ入力")).toBeInTheDocument();
      });
    });

    it("パース結果がフォーム初期値に反映される", async () => {
      render(ImportView, createProps({ initialSourcePath: "/path/to/source" }));

      await waitFor(() => {
        expect(
          screen.getByDisplayValue("パースされたタイトル"),
        ).toBeInTheDocument();
        expect(
          screen.getByDisplayValue("パースアーティスト"),
        ).toBeInTheDocument();
      });
    });

    it("取り込み元パスが表示される", async () => {
      render(ImportView, createProps({ initialSourcePath: "/path/to/source" }));

      await waitFor(() => {
        expect(screen.getByText("/path/to/source")).toBeInTheDocument();
      });
    });

    it("fullモードで取り込みモード選択が表示される", async () => {
      render(ImportView, createProps({ initialSourcePath: "/path/to/source" }));

      await waitFor(() => {
        expect(screen.getByText("コピー")).toBeInTheDocument();
        expect(screen.getByText("移動")).toBeInTheDocument();
      });
    });

    it("タイトルが空の場合は実行ボタンが無効になる", async () => {
      const user = userEvent.setup();
      render(ImportView, createProps({ initialSourcePath: "/path/to/source" }));

      await waitFor(() => {
        expect(
          screen.getByDisplayValue("パースされたタイトル"),
        ).toBeInTheDocument();
      });

      const titleInput = screen.getByDisplayValue("パースされたタイトル");
      await user.clear(titleInput);

      expect(screen.getByText("取り込み実行")).toBeDisabled();
    });

    it("取り込み実行で enqueue_import が呼ばれ onBack が呼ばれる", async () => {
      const user = userEvent.setup();
      const props = createProps({ initialSourcePath: "/path/to/source" });
      render(ImportView, props);

      await waitFor(() => {
        expect(screen.getByText("取り込み実行")).toBeInTheDocument();
      });

      await user.click(screen.getByText("取り込み実行"));

      await waitFor(() => {
        expect(props.onBack).toHaveBeenCalled();
      });
    });

    it("戻るボタンで選択ステップに戻る", async () => {
      const user = userEvent.setup();
      render(ImportView, createProps({ initialSourcePath: "/path/to/source" }));

      await waitFor(() => {
        expect(screen.getByText("メタデータ入力")).toBeInTheDocument();
      });

      await user.click(screen.getByText("← 戻る"));

      await waitFor(() => {
        expect(screen.getByText("取り込み対象を選択")).toBeInTheDocument();
      });
    });
  });
});
