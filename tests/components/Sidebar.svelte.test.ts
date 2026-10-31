import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import Sidebar from "../../src/lib/components/Sidebar.svelte";
import { getToasts, removeToast } from "../../src/lib/stores/toast.svelte";
import type { Library, Playlist } from "../../src/lib/types";

const MOCK_LIBRARIES: Library[] = [
  { id: "lib-1", name: "マイライブラリ", path: "/path/to/lib1" },
  { id: "lib-2", name: "サブライブラリ", path: "/path/to/lib2" },
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
    onNavigateToAppSettings: vi.fn(),
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
    if (cmd === "list_playlists") return MOCK_PLAYLISTS;
  });
});

describe("Sidebar コンポーネント", () => {
  it("ロゴ「SHARAKU」が表示される", () => {
    render(Sidebar, createProps());
    expect(screen.getByText("SHARAKU")).toBeInTheDocument();
  });

  async function openLibraryMenu(user: ReturnType<typeof userEvent.setup>) {
    await user.click(screen.getByRole("button", { name: /ライブラリ/ }));
  }

  it("ライブラリ切り替えボタンにアクティブなライブラリ名が表示される", () => {
    render(Sidebar, createProps());
    expect(
      screen.getByRole("button", { name: /マイライブラリ/ }),
    ).toHaveAttribute("aria-expanded", "false");
  });

  it("ライブラリ切り替えボタンを押すとライブラリ一覧が開き、アクティブなものがハイライトされる", async () => {
    const user = userEvent.setup();
    const { container } = render(Sidebar, createProps());

    await openLibraryMenu(user);

    await waitFor(() => {
      expect(screen.getByText("サブライブラリ")).toBeInTheDocument();
    });
    const activeItem = container.querySelector(".sidebar-library-item.active");
    expect(activeItem?.textContent).toContain("マイライブラリ");
  });

  it("ライブラリクリックで switch_library が呼ばれ onSwitchLibrary が発火する", async () => {
    const user = userEvent.setup();
    const switchSpy = vi.fn();
    mockIPC((cmd: string) => {
      if (cmd === "list_libraries") return MOCK_LIBRARIES;
      if (cmd === "switch_library") return switchSpy();
    });
    const props = createProps();
    render(Sidebar, props);

    await openLibraryMenu(user);
    await waitFor(() => {
      expect(screen.getByText("サブライブラリ")).toBeInTheDocument();
    });

    await user.click(screen.getByText("サブライブラリ"));

    await waitFor(() => {
      expect(switchSpy).toHaveBeenCalled();
      expect(props.onSwitchLibrary).toHaveBeenCalledWith(MOCK_LIBRARIES[1]);
    });
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });

  it("ライブラリメニューは Escape とメニューの外のクリックで閉じる", async () => {
    const user = userEvent.setup();
    render(Sidebar, createProps());

    await openLibraryMenu(user);
    expect(screen.getByRole("menu")).toBeInTheDocument();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();

    await openLibraryMenu(user);
    expect(screen.getByRole("menu")).toBeInTheDocument();
    await user.click(screen.getByText("SHARAKU"));
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });

  it("ライブラリメニューは開くとアクティブなライブラリにフォーカスし、矢印キーで移動し、Escape で切り替えボタンに戻る", async () => {
    const user = userEvent.setup();
    render(Sidebar, createProps());
    await waitFor(() => {
      expect(screen.getByText("マイライブラリ")).toBeInTheDocument();
    });

    await openLibraryMenu(user);
    const items = await screen.findAllByRole("menuitem");
    await waitFor(() => {
      expect(items[0]).toHaveFocus();
    });
    await user.keyboard("{ArrowDown}");
    expect(
      screen.getByRole("menuitem", { name: "サブライブラリ" }),
    ).toHaveFocus();
    await user.keyboard("{ArrowUp}{ArrowUp}");
    expect(
      screen.getByRole("menuitem", { name: "+ ライブラリを追加" }),
    ).toHaveFocus();

    await user.keyboard("{Escape}");
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /ライブラリ/, expanded: false }),
    ).toHaveFocus();
  });

  it("ライブラリメニューの「ライブラリ設定」で onNavigate('settings') が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(Sidebar, props);

    await openLibraryMenu(user);
    await user.click(screen.getByText("ライブラリ設定"));

    expect(props.onNavigate).toHaveBeenCalledWith("settings");
  });

  it("ライブラリメニューの「+ ライブラリを追加」で onNavigate('add-library') が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(Sidebar, props);

    await openLibraryMenu(user);
    await user.click(screen.getByText("+ ライブラリを追加"));

    expect(props.onNavigate).toHaveBeenCalledWith("add-library");
  });

  it("ナビのホーム・ブラウズで onNavigate が呼ばれ、表示中の項目に aria-current が付く", async () => {
    const user = userEvent.setup();
    const props = createProps({ currentView: "home" });
    render(Sidebar, props);

    const home = screen.getByRole("button", { name: "ホーム" });
    expect(home).toHaveAttribute("aria-current", "page");

    await user.click(screen.getByRole("button", { name: "ブラウズ" }));
    expect(props.onNavigate).toHaveBeenCalledWith("library");
    await user.click(home);
    expect(props.onNavigate).toHaveBeenCalledWith("home");
  });

  it("「取り込む」で onNavigate('bulk-import') が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(Sidebar, props);

    await user.click(screen.getByRole("button", { name: "取り込む" }));

    expect(props.onNavigate).toHaveBeenCalledWith("bulk-import");
  });

  it("タグの一覧は表示しない（タグはブラウズの絞り込みから使う）", async () => {
    render(Sidebar, createProps());
    await waitFor(() => {
      expect(screen.getByText("お気に入り")).toBeInTheDocument();
    });
    expect(screen.queryByText("Tags")).not.toBeInTheDocument();
  });

  it("アプリ設定ボタンで onNavigateToAppSettings が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(Sidebar, props);

    await user.click(screen.getByRole("button", { name: "アプリ設定" }));

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
