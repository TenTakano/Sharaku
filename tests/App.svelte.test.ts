import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  render,
  screen,
  cleanup,
  waitFor,
  fireEvent,
} from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { getToasts, removeToast } from "../src/lib/stores/toast.svelte";
import type { AppSettings, Library, WorkDetail } from "../src/lib/types";

// jsdom cannot measure element size, so virtua's real VList never renders its
// items; swap in a mock that renders every item unconditionally (same
// approach as WorkGrid.svelte.test.ts) so library work cards are queryable.
vi.mock("virtua/svelte", async () => {
  const VListMock = (await import("./__mocks__/VListMock.svelte")).default;
  return { VList: VListMock };
});

const { default: App } = await import("../src/App.svelte");

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

const MOCK_WORK: WorkDetail = {
  id: 1,
  title: "作品A",
  path: "/path/to/work",
  workType: "image",
  pageCount: 3,
  createdAt: "2025-01-01T00:00:00Z",
  artist: null,
  year: null,
  genre: null,
  circle: null,
  origin: null,
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
    mockIPC((cmd: string) => {
      if (cmd === "get_active_library") return MOCK_LIBRARY;
      if (cmd === "list_libraries") return [MOCK_LIBRARY];
      if (cmd === "list_tags") return [];
      if (cmd === "list_works") return [];
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "classify_drop_path") return "image";
      if (cmd === "parse_folder_name") return { title: "photo", artist: null };
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
      expect(screen.getByDisplayValue("photo")).toBeInTheDocument();
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
      expect(
        screen.getByDisplayValue("folder-no-subfolders"),
      ).toBeInTheDocument();
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

describe("App のプレイリスト遷移", () => {
  it("プレイリストを選択するとプレイリスト画面に遷移する", async () => {
    const user = userEvent.setup();
    mockIPC((cmd: string) => {
      if (cmd === "get_active_library") return MOCK_LIBRARY;
      if (cmd === "list_libraries") return [MOCK_LIBRARY];
      if (cmd === "list_tags") return [];
      if (cmd === "list_works") return [];
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "list_playlists") return [{ id: 1, name: "お気に入り" }];
      if (cmd === "get_playlist_items") return [];
    });

    render(App);

    await waitFor(() => {
      expect(screen.getByText("お気に入り")).toBeInTheDocument();
    });
    await user.click(screen.getByText("お気に入り"));

    await waitFor(() => {
      expect(
        screen.getByText("お気に入り", { selector: "h1" }),
      ).toBeInTheDocument();
    });
  });

  it("プレイリスト表示中に削除するとライブラリ画面に戻る", async () => {
    const user = userEvent.setup();
    let deleted = false;
    mockIPC((cmd: string) => {
      if (cmd === "get_active_library") return MOCK_LIBRARY;
      if (cmd === "list_libraries") return [MOCK_LIBRARY];
      if (cmd === "list_tags") return [];
      if (cmd === "list_works") return [];
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "list_playlists")
        return deleted ? [] : [{ id: 1, name: "お気に入り" }];
      if (cmd === "get_playlist_items") return [];
      if (cmd === "delete_playlist") {
        deleted = true;
        return undefined;
      }
    });

    render(App);

    await waitFor(() => {
      expect(screen.getByText("お気に入り")).toBeInTheDocument();
    });
    await user.click(screen.getByText("お気に入り"));

    await waitFor(() => {
      expect(
        screen.getByText("お気に入り", { selector: "h1" }),
      ).toBeInTheDocument();
    });

    const sidebarItem = screen.getByText("お気に入り", {
      selector: ".sidebar-playlist-name",
    });
    fireEvent.contextMenu(sidebarItem);

    await waitFor(() => {
      expect(screen.getByText("削除")).toBeInTheDocument();
    });
    await user.click(screen.getByText("削除"));

    await waitFor(() => {
      expect(
        screen.getByText(
          "「お気に入り」を削除しますか？この操作は取り消せません。",
        ),
      ).toBeInTheDocument();
    });
    const confirmButtons = screen.getAllByText("削除");
    await user.click(confirmButtons[confirmButtons.length - 1]);

    await waitFor(() => {
      expect(
        screen.getByText("テストライブラリ", { selector: "h1" }),
      ).toBeInTheDocument();
    });
  });

  it("プレイリストから作品を開き、ビューアーの戻るでプレイリスト画面に戻る", async () => {
    const user = userEvent.setup();
    mockWindows("main");
    mockIPC((cmd: string) => {
      if (cmd === "get_active_library") return MOCK_LIBRARY;
      if (cmd === "list_libraries") return [MOCK_LIBRARY];
      if (cmd === "list_tags") return [];
      if (cmd === "list_works") return [];
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "list_playlists") return [{ id: 1, name: "お気に入り" }];
      if (cmd === "get_playlist_items")
        return [
          {
            workId: 1,
            title: "作品A",
            workType: "image",
            pageCount: 3,
            createdAt: "2025-01-01T00:00:00Z",
          },
        ];
      if (cmd === "get_thumbnail") return [0x52, 0x49, 0x46, 0x46];
      if (cmd === "get_work") return MOCK_WORK;
      if (cmd === "get_tags_for_work") return [];
      if (cmd === "plugin:window|is_fullscreen") return false;
      if (cmd === "plugin:window|set_fullscreen") return undefined;
    });

    render(App);

    await waitFor(() => {
      expect(screen.getByText("お気に入り")).toBeInTheDocument();
    });
    await user.click(screen.getByText("お気に入り"));

    await waitFor(() => {
      expect(
        screen.getByText("お気に入り", { selector: "h1" }),
      ).toBeInTheDocument();
    });
    await waitFor(() => {
      expect(screen.getByText("作品A")).toBeInTheDocument();
    });
    await user.click(screen.getByText("作品A"));

    await waitFor(() => {
      expect(
        screen.getByText("作品A", { selector: ".viewer-title" }),
      ).toBeInTheDocument();
    });

    await user.click(
      screen.getByText(/テストライブラリ/, { selector: "button" }),
    );

    await waitFor(() => {
      expect(
        screen.getByText("お気に入り", { selector: "h1" }),
      ).toBeInTheDocument();
    });
  });

  it("プレイリストからライブラリへ戻った後にライブラリの作品を開くと、ビューアーの戻る先はライブラリになる", async () => {
    const user = userEvent.setup();
    mockWindows("main");
    const LIBRARY_WORK: WorkDetail = { ...MOCK_WORK, id: 2, title: "作品B" };
    mockIPC((cmd: string, args: Record<string, unknown>) => {
      if (cmd === "get_active_library") return MOCK_LIBRARY;
      if (cmd === "list_libraries") return [MOCK_LIBRARY];
      if (cmd === "list_tags") return [];
      if (cmd === "list_works")
        return [
          {
            id: 2,
            title: "作品B",
            workType: "image",
            pageCount: 1,
            createdAt: "2025-01-01T00:00:00Z",
          },
        ];
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "list_playlists") return [{ id: 1, name: "お気に入り" }];
      if (cmd === "get_playlist_items")
        return [
          {
            workId: 1,
            title: "作品A",
            workType: "image",
            pageCount: 3,
            createdAt: "2025-01-01T00:00:00Z",
          },
        ];
      if (cmd === "get_thumbnail") return [0x52, 0x49, 0x46, 0x46];
      if (cmd === "get_work")
        return args.workId === 2 ? LIBRARY_WORK : MOCK_WORK;
      if (cmd === "get_tags_for_work") return [];
      if (cmd === "plugin:window|is_fullscreen") return false;
      if (cmd === "plugin:window|set_fullscreen") return undefined;
    });

    render(App);

    await waitFor(() => {
      expect(screen.getByText("お気に入り")).toBeInTheDocument();
    });
    await user.click(screen.getByText("お気に入り"));
    await waitFor(() => {
      expect(
        screen.getByText("お気に入り", { selector: "h1" }),
      ).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(screen.getByText("作品A")).toBeInTheDocument();
    });
    await user.click(screen.getByText("作品A"));
    await waitFor(() => {
      expect(
        screen.getByText("作品A", { selector: ".viewer-title" }),
      ).toBeInTheDocument();
    });

    await user.click(
      screen.getByText(/テストライブラリ/, { selector: "button" }),
    );
    await waitFor(() => {
      expect(
        screen.getByText("お気に入り", { selector: "h1" }),
      ).toBeInTheDocument();
    });

    await user.click(screen.getByText("←"));
    await waitFor(() => {
      expect(
        screen.getByText("テストライブラリ", { selector: "h1" }),
      ).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(screen.getByText("作品B")).toBeInTheDocument();
    });
    await user.click(screen.getByText("作品B"));
    await waitFor(() => {
      expect(
        screen.getByText("作品B", { selector: ".viewer-title" }),
      ).toBeInTheDocument();
    });

    await user.click(
      screen.getByText(/テストライブラリ/, { selector: "button" }),
    );
    await waitFor(() => {
      expect(
        screen.getByText("テストライブラリ", { selector: "h1" }),
      ).toBeInTheDocument();
    });
    expect(
      screen.queryByText("お気に入り", { selector: "h1" }),
    ).not.toBeInTheDocument();
  });
});

describe("App のプレイリストリネーム反映", () => {
  it("選択中プレイリストをリネームするとヘッダタイトルが更新される", async () => {
    const user = userEvent.setup();
    mockIPC((cmd: string) => {
      if (cmd === "get_active_library") return MOCK_LIBRARY;
      if (cmd === "list_libraries") return [MOCK_LIBRARY];
      if (cmd === "list_tags") return [];
      if (cmd === "list_works") return [];
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "list_playlists") return [{ id: 1, name: "お気に入り" }];
      if (cmd === "get_playlist_items") return [];
      if (cmd === "rename_playlist") return undefined;
    });

    render(App);

    await waitFor(() => {
      expect(screen.getByText("お気に入り")).toBeInTheDocument();
    });
    await user.click(screen.getByText("お気に入り"));

    await waitFor(() => {
      expect(
        screen.getByText("お気に入り", { selector: "h1" }),
      ).toBeInTheDocument();
    });

    const sidebarItem = screen.getByText("お気に入り", {
      selector: ".sidebar-playlist-name",
    });
    fireEvent.contextMenu(sidebarItem);

    await waitFor(() => {
      expect(screen.getByText("名前を変更")).toBeInTheDocument();
    });
    await user.click(screen.getByText("名前を変更"));

    const input = screen.getByDisplayValue("お気に入り");
    await user.clear(input);
    await user.type(input, "新しい名前{Enter}");

    await waitFor(() => {
      expect(
        screen.getByText("新しい名前", { selector: "h1" }),
      ).toBeInTheDocument();
    });
  });

  it("選択中でないプレイリストをリネームしてもヘッダタイトルは変わらない", async () => {
    const user = userEvent.setup();
    let playlists = [
      { id: 1, name: "お気に入り" },
      { id: 2, name: "あとで読む" },
    ];
    mockIPC((cmd: string, args: Record<string, unknown>) => {
      if (cmd === "get_active_library") return MOCK_LIBRARY;
      if (cmd === "list_libraries") return [MOCK_LIBRARY];
      if (cmd === "list_tags") return [];
      if (cmd === "list_works") return [];
      if (cmd === "get_settings") return MOCK_SETTINGS;
      if (cmd === "list_playlists") return playlists;
      if (cmd === "get_playlist_items") return [];
      if (cmd === "rename_playlist") {
        const { id, name } = args as { id: number; name: string };
        playlists = playlists.map((p) => (p.id === id ? { ...p, name } : p));
        return undefined;
      }
    });

    render(App);

    await waitFor(() => {
      expect(screen.getByText("お気に入り")).toBeInTheDocument();
    });
    await user.click(screen.getByText("お気に入り"));

    await waitFor(() => {
      expect(
        screen.getByText("お気に入り", { selector: "h1" }),
      ).toBeInTheDocument();
    });

    const otherItem = screen.getByText("あとで読む", {
      selector: ".sidebar-playlist-name",
    });
    fireEvent.contextMenu(otherItem);

    await waitFor(() => {
      expect(screen.getByText("名前を変更")).toBeInTheDocument();
    });
    await user.click(screen.getByText("名前を変更"));

    const input = screen.getByDisplayValue("あとで読む");
    await user.clear(input);
    await user.type(input, "別名{Enter}");

    await waitFor(() => {
      expect(screen.getByText("別名")).toBeInTheDocument();
    });
    expect(
      screen.getByText("お気に入り", { selector: "h1" }),
    ).toBeInTheDocument();
  });
});
