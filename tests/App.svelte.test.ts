import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import { mockIPC } from "@tauri-apps/api/mocks";
import App from "../src/App.svelte";
import { getToasts, removeToast } from "../src/lib/stores/toast.svelte";
import type { AppSettings, Library } from "../src/lib/types";

type DragDropPayload =
  | { type: "enter"; paths: string[] }
  | { type: "drop"; paths: string[] }
  | { type: "leave" }
  | { type: "over" };

let dragDropHandler:
  | ((event: { event: string; id: number; payload: DragDropPayload }) => void)
  | undefined;

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({
    onDragDropEvent: (
      handler: (event: {
        event: string;
        id: number;
        payload: DragDropPayload;
      }) => void,
    ) => {
      dragDropHandler = handler;
      return Promise.resolve(() => {});
    },
  }),
}));

globalThis.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof globalThis.ResizeObserver;

Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: () => {},
    removeEventListener: () => {},
    addListener: () => {},
    removeListener: () => {},
    dispatchEvent: () => false,
  }),
});

const MOCK_LIBRARY: Library = {
  id: "lib-1",
  name: "テストライブラリ",
  path: "/path/to/lib",
};

const MOCK_SETTINGS: AppSettings = {
  resourceMode: "full",
  directoryTemplate: "{artist}/{title}",
  typeLabelImage: "Image",
  typeLabelFolder: "Folder",
  deleteFileAction: "ask",
};

function emitDrop(paths: string[]) {
  dragDropHandler?.({
    event: "tauri://drag-drop",
    id: 1,
    payload: { type: "drop", paths },
  });
}

beforeEach(() => {
  cleanup();
  dragDropHandler = undefined;
  for (const toast of getToasts()) {
    removeToast(toast.id);
  }
});

describe("App のドラッグ&ドロップ処理", () => {
  it("複数パスドロップで一括取り込み画面へ遷移する", async () => {
    mockIPC((cmd: string) => {
      if (cmd === "get_active_library") return MOCK_LIBRARY;
      if (cmd === "list_libraries") return [MOCK_LIBRARY];
      if (cmd === "list_tags") return [];
      if (cmd === "list_works") return [];
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "parse_folder_name") return { title: "x", artist: null };
    });

    render(App);

    await waitFor(() => {
      expect(dragDropHandler).toBeDefined();
    });
    await waitFor(() => {
      expect(screen.getByText("テストライブラリ")).toBeInTheDocument();
    });

    emitDrop(["/root/folder1", "/root/folder2"]);

    await waitFor(() => {
      expect(
        screen.getByText("一括取り込み: テストライブラリ"),
      ).toBeInTheDocument();
    });
  });

  it("画像ファイルのドロップで単一取り込み画面にimage kindが渡る", async () => {
    const previewSpy = vi.fn(() => "/library/pictures/photo");
    mockIPC((cmd: string, args: Record<string, unknown>) => {
      if (cmd === "get_active_library") return MOCK_LIBRARY;
      if (cmd === "list_libraries") return [MOCK_LIBRARY];
      if (cmd === "list_tags") return [];
      if (cmd === "list_works") return [];
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "classify_drop_path") return "image";
      if (cmd === "parse_folder_name") return { title: "photo", artist: null };
      if (cmd === "preview_import_path") return previewSpy(args);
    });

    render(App);

    await waitFor(() => {
      expect(dragDropHandler).toBeDefined();
    });
    await waitFor(() => {
      expect(screen.getByText("テストライブラリ")).toBeInTheDocument();
    });

    emitDrop(["/root/photo.jpg"]);

    await waitFor(() => {
      expect(
        screen.getByText("取り込み: テストライブラリ"),
      ).toBeInTheDocument();
    });
    await waitFor(() => {
      expect(previewSpy).toHaveBeenCalledWith(
        expect.objectContaining({ kind: "image" }),
      );
    });
  });

  it("単一ディレクトリドロップかつサブフォルダに画像がある場合は一括取り込み画面(ルートパスモード)へ遷移する", async () => {
    const hasSubfoldersSpy = vi.fn(() => true);
    const discoverFoldersSpy = vi.fn(() => ({
      folders: [],
      images: [],
      skippedFolders: [],
    }));
    mockIPC((cmd: string, args: Record<string, unknown>) => {
      if (cmd === "get_active_library") return MOCK_LIBRARY;
      if (cmd === "list_libraries") return [MOCK_LIBRARY];
      if (cmd === "list_tags") return [];
      if (cmd === "list_works") return [];
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "classify_drop_path") return "folder";
      if (cmd === "has_image_subfolders") return hasSubfoldersSpy(args);
      if (cmd === "discover_folders") return discoverFoldersSpy(args);
    });

    render(App);

    await waitFor(() => {
      expect(dragDropHandler).toBeDefined();
    });
    await waitFor(() => {
      expect(screen.getByText("テストライブラリ")).toBeInTheDocument();
    });

    emitDrop(["/root/folder-with-subfolders"]);

    await waitFor(() => {
      expect(hasSubfoldersSpy).toHaveBeenCalledWith({
        dir: "/root/folder-with-subfolders",
      });
    });
    await waitFor(() => {
      expect(
        screen.getByText("一括取り込み: テストライブラリ"),
      ).toBeInTheDocument();
    });
    await waitFor(() => {
      expect(discoverFoldersSpy).toHaveBeenCalledWith(
        expect.objectContaining({ rootPath: "/root/folder-with-subfolders" }),
      );
    });
  });

  it("単一ディレクトリドロップかつサブフォルダに画像がない場合は単一取り込み画面(folder kind)へ遷移する", async () => {
    const hasSubfoldersSpy = vi.fn(() => false);
    const previewSpy = vi.fn(() => "/library/works/folder-no-subfolders");
    mockIPC((cmd: string, args: Record<string, unknown>) => {
      if (cmd === "get_active_library") return MOCK_LIBRARY;
      if (cmd === "list_libraries") return [MOCK_LIBRARY];
      if (cmd === "list_tags") return [];
      if (cmd === "list_works") return [];
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "classify_drop_path") return "folder";
      if (cmd === "has_image_subfolders") return hasSubfoldersSpy(args);
      if (cmd === "parse_folder_name")
        return { title: "folder-no-subfolders", artist: null };
      if (cmd === "preview_import_path") return previewSpy(args);
    });

    render(App);

    await waitFor(() => {
      expect(dragDropHandler).toBeDefined();
    });
    await waitFor(() => {
      expect(screen.getByText("テストライブラリ")).toBeInTheDocument();
    });

    emitDrop(["/root/folder-no-subfolders"]);

    await waitFor(() => {
      expect(
        screen.getByText("取り込み: テストライブラリ"),
      ).toBeInTheDocument();
    });
    await waitFor(() => {
      expect(previewSpy).toHaveBeenCalledWith(
        expect.objectContaining({ kind: "folder" }),
      );
    });
  });

  it("未対応のファイル形式をドロップするとエラートーストが表示される", async () => {
    mockIPC((cmd: string) => {
      if (cmd === "get_active_library") return MOCK_LIBRARY;
      if (cmd === "list_libraries") return [MOCK_LIBRARY];
      if (cmd === "list_tags") return [];
      if (cmd === "list_works") return [];
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "classify_drop_path") return "other";
    });

    render(App);

    await waitFor(() => {
      expect(dragDropHandler).toBeDefined();
    });
    await waitFor(() => {
      expect(screen.getByText("テストライブラリ")).toBeInTheDocument();
    });

    emitDrop(["/root/note.txt"]);

    await waitFor(() => {
      expect(
        screen.getByText("対応していないファイル形式です"),
      ).toBeInTheDocument();
    });
  });
});
