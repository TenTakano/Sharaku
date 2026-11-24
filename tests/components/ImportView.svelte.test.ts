import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import ImportView from "../../src/lib/components/ImportView.svelte";
import { getToasts, removeToast } from "../../src/lib/stores/toast.svelte";

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
    if (cmd === "parse_folder_name")
      return { title: "パースされたタイトル", artist: "パースアーティスト" };
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

    it("元の場所に登録される旨の案内が表示される", async () => {
      render(ImportView, createProps({ initialSourcePath: "/path/to/source" }));

      await waitFor(() => {
        expect(screen.getByText("元の場所に登録されます")).toBeInTheDocument();
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

  describe("取り込み対象の種別選択", () => {
    it("初期状態ではフォルダ向けの説明とボタンラベルが表示される", () => {
      render(ImportView, createProps());

      expect(
        screen.getByText("取り込む画像フォルダを選択してください。"),
      ).toBeInTheDocument();
      expect(screen.getByText("フォルダを選択...")).toBeInTheDocument();
    });

    it("画像ファイルを選択すると説明文とボタンラベルが画像向けに切り替わる", async () => {
      const user = userEvent.setup();
      render(ImportView, createProps());

      await user.click(screen.getByText("画像ファイル（1枚）"));

      expect(
        screen.getByText("取り込む画像ファイルを選択してください。"),
      ).toBeInTheDocument();
      expect(screen.getByText("画像を選択...")).toBeInTheDocument();
    });

    it("フォルダ選択時はディレクトリ選択ダイアログが開かれる", async () => {
      const user = userEvent.setup();
      const dialogOpenSpy = vi.fn(() => "/path/to/folder");
      mockIPC((cmd: string, args: Record<string, unknown>) => {
        if (cmd === "plugin:dialog|open") return dialogOpenSpy(args);
      });
      render(ImportView, createProps());

      await user.click(screen.getByText("フォルダを選択..."));

      expect(dialogOpenSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          options: expect.objectContaining({ directory: true }),
        }),
      );
    });

    it("画像選択時は拡張子フィルタ付きの画像ファイル選択ダイアログが開かれる", async () => {
      const user = userEvent.setup();
      const dialogOpenSpy = vi.fn(() => "/path/to/photo.jpg");
      mockIPC((cmd: string, args: Record<string, unknown>) => {
        if (cmd === "parse_folder_name")
          return { title: "パースされたタイトル", artist: null };
        if (cmd === "plugin:dialog|open") return dialogOpenSpy(args);
      });
      render(ImportView, createProps());

      await user.click(screen.getByText("画像ファイル（1枚）"));
      await user.click(screen.getByText("画像を選択..."));

      expect(dialogOpenSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          options: expect.objectContaining({
            directory: false,
            multiple: false,
            filters: [
              {
                name: "画像",
                extensions: ["jpg", "jpeg", "png", "gif", "webp", "bmp"],
              },
            ],
          }),
        }),
      );
    });
  });

  describe("画像インポート時のメタデータ抽出とkind伝播", () => {
    it("basenameOf/stripExtensionでファイル名から拡張子を除いたタイトルが抽出される", async () => {
      const parseFolderNameSpy = vi.fn(() => ({
        title: "my-photo",
        artist: null,
      }));
      mockIPC((cmd: string, args: Record<string, unknown>) => {
        if (cmd === "parse_folder_name") return parseFolderNameSpy(args);
      });

      render(
        ImportView,
        createProps({
          initialSourcePath: "/path/to/my-photo.jpg",
          initialKind: "image",
        }),
      );

      await waitFor(() => {
        expect(parseFolderNameSpy).toHaveBeenCalledWith({
          folderName: "my-photo",
        });
      });
    });

    it("enqueue_importにkind=imageが伝播する", async () => {
      const enqueueSpy = vi.fn(() => ({ jobId: "job-1" }));
      mockIPC((cmd: string, args: Record<string, unknown>) => {
        if (cmd === "parse_folder_name")
          return { title: "my-photo", artist: null };
        if (cmd === "enqueue_import") return enqueueSpy(args);
      });

      const user = userEvent.setup();
      const props = createProps({
        initialSourcePath: "/path/to/my-photo.jpg",
        initialKind: "image",
      });
      render(ImportView, props);

      await waitFor(() => {
        expect(screen.getByText("取り込み実行")).toBeInTheDocument();
      });

      await user.click(screen.getByText("取り込み実行"));

      await waitFor(() => {
        expect(enqueueSpy).toHaveBeenCalledWith({
          requests: [expect.objectContaining({ kind: "image" })],
        });
        expect(props.onBack).toHaveBeenCalled();
      });
    });
  });
});
