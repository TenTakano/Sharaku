import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import SettingsView from "../../src/lib/components/LibrarySettingsView.svelte";
import type { AppSettings } from "../../src/lib/types";
import { getToasts, removeToast } from "../../src/lib/stores/toast.svelte";

const MOCK_SETTINGS: AppSettings = {
  resourceMode: "full",
  directoryTemplate: "{artist}/{title}",
  typeLabelImage: "Image",
  typeLabelFolder: "Folder",
  deleteFileAction: "ask",
};

function createProps(overrides = {}) {
  return {
    libraryId: "lib-1",
    libraryName: "テストライブラリ",
    libraryPath: "/path/to/library",
    onNavigate: vi.fn(),
    onImportUnregistered: vi.fn(),
    onDeleteLibrary: vi.fn(),
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
    if (cmd === "set_resource_mode") return undefined;
    if (cmd === "set_delete_file_action") return undefined;
    if (cmd === "validate_template") return { valid: true, error: null };
    if (cmd === "preview_template") return "/library/Artist/Title";
    if (cmd === "set_directory_template") return undefined;
    if (cmd === "set_type_labels") return undefined;
    if (cmd === "remove_library") return undefined;
    if (cmd === "check_integrity")
      return {
        totalWorks: 10,
        orphanWorks: [],
        unregisteredEntries: [],
      };
  });
});

describe("SettingsView コンポーネント", () => {
  it("読み込み中に「読み込み中...」が表示される", () => {
    render(SettingsView, createProps());
    expect(screen.getByText("読み込み中...")).toBeInTheDocument();
  });

  it("設定読み込み後にセクションが表示される", async () => {
    render(SettingsView, createProps());

    await waitFor(() => {
      expect(screen.getByText("ライブラリルート")).toBeInTheDocument();
      expect(screen.getByText("リソース管理モード")).toBeInTheDocument();
    });
  });

  it("ライブラリルートパスが表示される", async () => {
    render(SettingsView, createProps());

    await waitFor(() => {
      expect(screen.getByText("/path/to/library")).toBeInTheDocument();
    });
  });

  it("libraryPath が null の場合「未設定」が表示される", async () => {
    render(SettingsView, createProps({ libraryPath: null }));

    await waitFor(() => {
      expect(screen.getByText("未設定")).toBeInTheDocument();
    });
  });

  it("リソース管理モードが表示される", async () => {
    render(SettingsView, createProps());

    await waitFor(() => {
      expect(screen.getByText("すべて管理")).toBeInTheDocument();
      expect(screen.getByText("メタデータのみ管理")).toBeInTheDocument();
    });
  });

  it("一括取り込みボタンで onNavigate('bulk-import') が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(SettingsView, props);

    await waitFor(() => {
      expect(screen.getByText("一括取り込みを開始")).toBeInTheDocument();
    });

    await user.click(screen.getByText("一括取り込みを開始"));

    expect(props.onNavigate).toHaveBeenCalledWith("bulk-import");
  });

  it("ライブラリ削除ボタンで確認ダイアログが表示される", async () => {
    const user = userEvent.setup();
    render(SettingsView, createProps());

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "ライブラリを削除" }),
      ).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: "ライブラリを削除" }));

    await waitFor(() => {
      expect(screen.getByText(/を削除しますか/)).toBeInTheDocument();
    });
  });

  it("削除確認で remove_library が呼ばれ onDeleteLibrary が発火する", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(SettingsView, props);

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "ライブラリを削除" }),
      ).toBeInTheDocument();
    });

    await user.click(screen.getByRole("button", { name: "ライブラリを削除" }));

    await waitFor(() => {
      expect(screen.getByText("削除する")).toBeInTheDocument();
    });

    await user.click(screen.getByText("削除する"));

    await waitFor(() => {
      expect(props.onDeleteLibrary).toHaveBeenCalled();
    });
  });
});
