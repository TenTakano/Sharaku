import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import Sidebar from "../../src/lib/components/Sidebar.svelte";
import { getToasts, removeToast } from "../../src/lib/stores/toast.svelte";
import type { Library, Playlist, Tag } from "../../src/lib/types";

const MOCK_LIBRARIES: Library[] = [
  { id: "lib-1", name: "マイライブラリ", path: "/path/to/lib1" },
  { id: "lib-2", name: "サブライブラリ", path: "/path/to/lib2" },
];

const MOCK_TAGS: Tag[] = [
  { id: 1, name: "風景", category: "genre" },
  { id: 2, name: "人物", category: "genre" },
  { id: 3, name: "田中太郎", category: "artist" },
];

const MOCK_PLAYLISTS: Playlist[] = [
  { id: 1, name: "お気に入り" },
  { id: 2, name: "あとで読む" },
];

function createProps(overrides = {}) {
  return {
    activeLibrary: MOCK_LIBRARIES[0],
    currentView: "grid",
    reloadTrigger: 0,
    playlistReloadTrigger: 0,
    onSwitchLibrary: vi.fn(),
    onNavigate: vi.fn(),
    onTagSelect: vi.fn(),
    onNavigateToAppSettings: vi.fn(),
    selectedTagIds: [] as number[],
    selectedPlaylistId: null as number | null,
    onSelectPlaylist: vi.fn(),
    onPlaylistRenamed: vi.fn(),
    onPlaylistDeleted: vi.fn(),
    ...overrides,
  };
}

beforeEach(() => {
  cleanup();
  for (const toast of getToasts()) {
    removeToast(toast.id);
  }
  mockIPC((cmd: string) => {
    if (cmd === "list_libraries") return MOCK_LIBRARIES;
    if (cmd === "list_tags") return MOCK_TAGS;
    if (cmd === "list_playlists") return MOCK_PLAYLISTS;
  });
});

describe("Sidebar コンポーネント", () => {
  it("ロゴ「SHARAKU」が表示される", () => {
    render(Sidebar, createProps());
    expect(screen.getByText("SHARAKU")).toBeInTheDocument();
  });

  it("ライブラリ一覧が表示される", async () => {
    render(Sidebar, createProps());

    await waitFor(() => {
      expect(screen.getByText("マイライブラリ")).toBeInTheDocument();
      expect(screen.getByText("サブライブラリ")).toBeInTheDocument();
    });
  });

  it("アクティブなライブラリがハイライトされる", async () => {
    const { container } = render(Sidebar, createProps());

    await waitFor(() => {
      expect(screen.getByText("マイライブラリ")).toBeInTheDocument();
    });

    const activeItem = container.querySelector(".sidebar-library-item.active");
    expect(activeItem).toBeInTheDocument();
    expect(activeItem?.textContent).toContain("マイライブラリ");
  });

  it("ライブラリクリックで switch_library が呼ばれ onSwitchLibrary が発火する", async () => {
    const user = userEvent.setup();
    const switchSpy = vi.fn();
    mockIPC((cmd: string) => {
      if (cmd === "list_libraries") return MOCK_LIBRARIES;
      if (cmd === "list_tags") return MOCK_TAGS;
      if (cmd === "switch_library") return switchSpy();
    });
    const props = createProps();
    render(Sidebar, props);

    await waitFor(() => {
      expect(screen.getByText("サブライブラリ")).toBeInTheDocument();
    });

    await user.click(screen.getByText("サブライブラリ"));

    await waitFor(() => {
      expect(switchSpy).toHaveBeenCalled();
      expect(props.onSwitchLibrary).toHaveBeenCalledWith(MOCK_LIBRARIES[1]);
    });
  });

  it("アクティブライブラリの設定ボタンクリックで onNavigate('settings') が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    const { container } = render(Sidebar, props);

    await waitFor(() => {
      expect(screen.getByText("マイライブラリ")).toBeInTheDocument();
    });

    const settingsBtn = container.querySelector(".sidebar-library-settings")!;
    await user.click(settingsBtn as HTMLElement);

    expect(props.onNavigate).toHaveBeenCalledWith("settings");
  });

  it("「+ ライブラリを追加」ボタンで onNavigate('add-library') が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(Sidebar, props);

    await user.click(screen.getByText("+ ライブラリを追加"));

    expect(props.onNavigate).toHaveBeenCalledWith("add-library");
  });

  it("タグがある場合、Tags セクションが表示される", async () => {
    render(Sidebar, createProps());

    await waitFor(() => {
      expect(screen.getByText("Tags")).toBeInTheDocument();
    });
  });

  it("タグがない場合、Tags セクションが表示されない", async () => {
    mockIPC((cmd: string) => {
      if (cmd === "list_libraries") return MOCK_LIBRARIES;
      if (cmd === "list_tags") return [];
    });
    render(Sidebar, createProps());

    await waitFor(() => {
      expect(screen.getByText("マイライブラリ")).toBeInTheDocument();
    });

    expect(screen.queryByText("Tags")).not.toBeInTheDocument();
  });

  it("カテゴリヘッダーをクリックするとタグリストが展開される", async () => {
    const user = userEvent.setup();
    render(Sidebar, createProps());

    await waitFor(() => {
      expect(screen.getByText("Tags")).toBeInTheDocument();
    });

    await user.click(screen.getByText(/^genre/));

    await waitFor(() => {
      expect(screen.getByText("風景")).toBeInTheDocument();
      expect(screen.getByText("人物")).toBeInTheDocument();
    });
  });

  it("展開されたカテゴリのタグをクリックすると onTagSelect が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(Sidebar, props);

    await waitFor(() => {
      expect(screen.getByText("Tags")).toBeInTheDocument();
    });

    await user.click(screen.getByText(/^genre/));
    await waitFor(() => {
      expect(screen.getByText("風景")).toBeInTheDocument();
    });

    await user.click(screen.getByText("風景"));
    expect(props.onTagSelect).toHaveBeenCalledWith(MOCK_TAGS[0]);
  });

  it("selectedTagIds に含まれるタグに selected クラスが付与される", async () => {
    const user = userEvent.setup();
    const { container } = render(Sidebar, createProps({ selectedTagIds: [1] }));

    await waitFor(() => {
      expect(screen.getByText("Tags")).toBeInTheDocument();
    });

    await user.click(screen.getByText(/^genre/));

    await waitFor(() => {
      const selectedItem = container.querySelector(
        ".sidebar-tag-item.selected",
      );
      expect(selectedItem).toBeInTheDocument();
      expect(selectedItem?.textContent).toBe("風景");
    });
  });

  it("アプリ設定ボタンで onNavigateToAppSettings が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(Sidebar, props);

    await user.click(screen.getByText(/アプリ設定/));

    expect(props.onNavigateToAppSettings).toHaveBeenCalledOnce();
  });

  describe("Playlists セクション", () => {
    it("プレイリスト一覧が表示される", async () => {
      render(Sidebar, createProps());

      await waitFor(() => {
        expect(screen.getByText("お気に入り")).toBeInTheDocument();
        expect(screen.getByText("あとで読む")).toBeInTheDocument();
      });
    });

    it("プレイリストクリックで onSelectPlaylist が呼ばれる", async () => {
      const user = userEvent.setup();
      const props = createProps();
      render(Sidebar, props);

      await waitFor(() => {
        expect(screen.getByText("お気に入り")).toBeInTheDocument();
      });
      await user.click(screen.getByText("お気に入り"));

      expect(props.onSelectPlaylist).toHaveBeenCalledWith(MOCK_PLAYLISTS[0]);
    });

    it("currentView が playlist かつ selectedPlaylistId が一致する場合 active クラスが付与される", async () => {
      const { container } = render(
        Sidebar,
        createProps({ currentView: "playlist", selectedPlaylistId: 1 }),
      );

      await waitFor(() => {
        const activeItem = container.querySelector(
          ".sidebar-playlist-item.active",
        );
        expect(activeItem).toBeInTheDocument();
        expect(activeItem?.textContent).toBe("お気に入り");
      });
    });

    it("selectedPlaylistId が一致しても currentView が playlist でなければ active クラスが付与されない", async () => {
      const { container } = render(
        Sidebar,
        createProps({ currentView: "library", selectedPlaylistId: 1 }),
      );

      await waitFor(() => {
        expect(screen.getByText("お気に入り")).toBeInTheDocument();
      });
      expect(
        container.querySelector(".sidebar-playlist-item.active"),
      ).not.toBeInTheDocument();
    });

    it("「+ 新規プレイリスト作成」で入力欄が表示され、Enter で create_playlist が呼ばれる", async () => {
      const user = userEvent.setup();
      const createSpy = vi.fn(() => ({ id: 3, name: "新しいプレイリスト" }));
      mockIPC((cmd: string) => {
        if (cmd === "list_libraries") return MOCK_LIBRARIES;
        if (cmd === "list_tags") return MOCK_TAGS;
        if (cmd === "list_playlists") return MOCK_PLAYLISTS;
        if (cmd === "create_playlist") return createSpy();
      });
      const props = createProps();
      render(Sidebar, props);

      await user.click(screen.getByText("+ 新規プレイリスト作成"));
      const input = screen.getByPlaceholderText("プレイリスト名");
      await user.type(input, "新しいプレイリスト{Enter}");

      await waitFor(() => {
        expect(createSpy).toHaveBeenCalledOnce();
        expect(props.onSelectPlaylist).toHaveBeenCalledWith({
          id: 3,
          name: "新しいプレイリスト",
        });
      });
    });

    it("プレイリストを右クリックすると名前変更・削除のコンテキストメニューが表示される", async () => {
      const user = userEvent.setup();
      render(Sidebar, createProps());

      await waitFor(() => {
        expect(screen.getByText("お気に入り")).toBeInTheDocument();
      });
      await user.pointer({
        keys: "[MouseRight]",
        target: screen.getByText("お気に入り"),
      });

      expect(screen.getByText("名前を変更")).toBeInTheDocument();
      expect(screen.getByText("削除")).toBeInTheDocument();
    });

    it("コンテキストメニューの名前を変更→編集→Enter で rename_playlist と onPlaylistRenamed が呼ばれる", async () => {
      const user = userEvent.setup();
      const renameSpy = vi.fn();
      mockIPC((cmd: string) => {
        if (cmd === "list_libraries") return MOCK_LIBRARIES;
        if (cmd === "list_tags") return MOCK_TAGS;
        if (cmd === "list_playlists") return MOCK_PLAYLISTS;
        if (cmd === "rename_playlist") return renameSpy();
      });
      const props = createProps();
      render(Sidebar, props);

      await waitFor(() => {
        expect(screen.getByText("お気に入り")).toBeInTheDocument();
      });
      await user.pointer({
        keys: "[MouseRight]",
        target: screen.getByText("お気に入り"),
      });
      await user.click(screen.getByText("名前を変更"));

      const input = screen.getByDisplayValue("お気に入り");
      await user.clear(input);
      await user.type(input, "新しい名前{Enter}");

      await waitFor(() => {
        expect(renameSpy).toHaveBeenCalledOnce();
        expect(props.onPlaylistRenamed).toHaveBeenCalledWith({
          id: 1,
          name: "新しい名前",
        });
      });
    });

    it("コンテキストメニューの削除→確認で delete_playlist と onPlaylistDeleted が呼ばれる", async () => {
      const user = userEvent.setup();
      const deleteSpy = vi.fn();
      mockIPC((cmd: string) => {
        if (cmd === "list_libraries") return MOCK_LIBRARIES;
        if (cmd === "list_tags") return MOCK_TAGS;
        if (cmd === "list_playlists") return MOCK_PLAYLISTS;
        if (cmd === "delete_playlist") return deleteSpy();
      });
      const props = createProps();
      render(Sidebar, props);

      await waitFor(() => {
        expect(screen.getByText("お気に入り")).toBeInTheDocument();
      });
      await user.pointer({
        keys: "[MouseRight]",
        target: screen.getByText("お気に入り"),
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
        expect(deleteSpy).toHaveBeenCalledOnce();
        expect(props.onPlaylistDeleted).toHaveBeenCalledWith(1);
      });
    });

    it("プレイリスト作成が失敗するとエラートーストが表示され入力欄が残る", async () => {
      const user = userEvent.setup();
      mockIPC((cmd: string) => {
        if (cmd === "list_libraries") return MOCK_LIBRARIES;
        if (cmd === "list_tags") return MOCK_TAGS;
        if (cmd === "list_playlists") return MOCK_PLAYLISTS;
        if (cmd === "create_playlist") throw new Error("db error");
      });
      const props = createProps();
      render(Sidebar, props);

      await user.click(screen.getByText("+ 新規プレイリスト作成"));
      const input = screen.getByPlaceholderText("プレイリスト名");
      await user.type(input, "新しいプレイリスト{Enter}");

      await waitFor(() => {
        expect(getToasts()).toHaveLength(1);
      });
      expect(getToasts()[0].type).toBe("error");
      expect(props.onSelectPlaylist).not.toHaveBeenCalled();
      expect(screen.getByPlaceholderText("プレイリスト名")).toBeInTheDocument();
    });

    it("プレイリスト名変更が失敗するとエラートーストが表示され編集状態が残る", async () => {
      const user = userEvent.setup();
      mockIPC((cmd: string) => {
        if (cmd === "list_libraries") return MOCK_LIBRARIES;
        if (cmd === "list_tags") return MOCK_TAGS;
        if (cmd === "list_playlists") return MOCK_PLAYLISTS;
        if (cmd === "rename_playlist") throw new Error("db error");
      });
      const props = createProps();
      render(Sidebar, props);

      await waitFor(() => {
        expect(screen.getByText("お気に入り")).toBeInTheDocument();
      });
      await user.pointer({
        keys: "[MouseRight]",
        target: screen.getByText("お気に入り"),
      });
      await user.click(screen.getByText("名前を変更"));

      const input = screen.getByDisplayValue("お気に入り");
      await user.clear(input);
      await user.type(input, "新しい名前{Enter}");

      await waitFor(() => {
        expect(getToasts()).toHaveLength(1);
      });
      expect(getToasts()[0].type).toBe("error");
      expect(props.onPlaylistRenamed).not.toHaveBeenCalled();
      expect(screen.getByDisplayValue("新しい名前")).toBeInTheDocument();
    });

    it("プレイリスト削除が失敗するとエラートーストが表示され確認ダイアログが残る", async () => {
      const user = userEvent.setup();
      mockIPC((cmd: string) => {
        if (cmd === "list_libraries") return MOCK_LIBRARIES;
        if (cmd === "list_tags") return MOCK_TAGS;
        if (cmd === "list_playlists") return MOCK_PLAYLISTS;
        if (cmd === "delete_playlist") throw new Error("db error");
      });
      const props = createProps();
      render(Sidebar, props);

      await waitFor(() => {
        expect(screen.getByText("お気に入り")).toBeInTheDocument();
      });
      await user.pointer({
        keys: "[MouseRight]",
        target: screen.getByText("お気に入り"),
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
        expect(getToasts()).toHaveLength(1);
      });
      expect(getToasts()[0].type).toBe("error");
      expect(props.onPlaylistDeleted).not.toHaveBeenCalled();
      expect(
        screen.getByText(
          "「お気に入り」を削除しますか？この操作は取り消せません。",
        ),
      ).toBeInTheDocument();
    });
  });
});
