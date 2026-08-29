import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import AddToPlaylistDialog from "../../src/lib/components/AddToPlaylistDialog.svelte";
import { getToasts, removeToast } from "../../src/lib/stores/toast.svelte";
import type { Playlist } from "../../src/lib/types";

const MOCK_PLAYLISTS: Playlist[] = [
  { id: 1, name: "お気に入り" },
  { id: 2, name: "あとで読む" },
];

function createProps(overrides = {}) {
  return {
    workId: 42,
    onClose: vi.fn(),
    ...overrides,
  };
}

beforeEach(() => {
  cleanup();
  for (const toast of getToasts()) {
    removeToast(toast.id);
  }
  mockIPC((cmd: string) => {
    if (cmd === "list_playlists") return MOCK_PLAYLISTS;
  });
});

describe("AddToPlaylistDialog コンポーネント", () => {
  it("プレイリスト一覧が表示される", async () => {
    render(AddToPlaylistDialog, createProps());

    await waitFor(() => {
      expect(screen.getByText("お気に入り")).toBeInTheDocument();
      expect(screen.getByText("あとで読む")).toBeInTheDocument();
    });
  });

  it("プレイリストをクリックすると add_item_to_playlist が呼ばれ、onClose される", async () => {
    const user = userEvent.setup();
    const addSpy = vi.fn();
    mockIPC((cmd: string, args: Record<string, unknown>) => {
      if (cmd === "list_playlists") return MOCK_PLAYLISTS;
      if (cmd === "add_item_to_playlist") return addSpy(args);
    });
    const props = createProps();
    render(AddToPlaylistDialog, props);

    await waitFor(() => {
      expect(screen.getByText("お気に入り")).toBeInTheDocument();
    });
    await user.click(screen.getByText("お気に入り"));

    await waitFor(() => {
      expect(addSpy).toHaveBeenCalledOnce();
      expect(addSpy).toHaveBeenCalledWith({ playlistId: 1, workId: 42 });
      expect(props.onClose).toHaveBeenCalledOnce();
    });
  });

  it("プレイリストがない場合は空メッセージを表示する", async () => {
    mockIPC((cmd: string) => {
      if (cmd === "list_playlists") return [];
    });
    render(AddToPlaylistDialog, createProps());

    await waitFor(() => {
      expect(screen.getByText("プレイリストがありません")).toBeInTheDocument();
    });
  });

  it("新規プレイリスト作成→追加のフローで create_playlist と add_item_to_playlist が呼ばれる", async () => {
    const user = userEvent.setup();
    const createSpy = vi.fn(() => ({ id: 3, name: "新規プレイリスト" }));
    const addSpy = vi.fn();
    mockIPC((cmd: string, args: Record<string, unknown>) => {
      if (cmd === "list_playlists") return MOCK_PLAYLISTS;
      if (cmd === "create_playlist") return createSpy(args);
      if (cmd === "add_item_to_playlist") return addSpy(args);
    });
    const props = createProps({ onPlaylistCreated: vi.fn() });
    render(AddToPlaylistDialog, props);

    await user.click(screen.getByText("+ 新規プレイリストを作成して追加"));
    const input = screen.getByPlaceholderText("新しいプレイリスト名");
    await user.type(input, "新規プレイリスト{Enter}");

    await waitFor(() => {
      expect(createSpy).toHaveBeenCalledOnce();
      expect(createSpy).toHaveBeenCalledWith({ name: "新規プレイリスト" });
      expect(addSpy).toHaveBeenCalledOnce();
      expect(addSpy).toHaveBeenCalledWith({ playlistId: 3, workId: 42 });
      expect(props.onPlaylistCreated).toHaveBeenCalledOnce();
      expect(props.onClose).toHaveBeenCalledOnce();
    });
  });

  it("キャンセルボタンで onClose が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(AddToPlaylistDialog, props);

    await user.click(screen.getByText("キャンセル"));

    expect(props.onClose).toHaveBeenCalledOnce();
  });

  it("Escape キーで onClose が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(AddToPlaylistDialog, props);

    await user.keyboard("{Escape}");

    expect(props.onClose).toHaveBeenCalledOnce();
  });

  it("add_item_to_playlist が失敗した場合エラートーストを表示し onClose は呼ばれない", async () => {
    const user = userEvent.setup();
    mockIPC((cmd: string) => {
      if (cmd === "list_playlists") return MOCK_PLAYLISTS;
      if (cmd === "add_item_to_playlist") throw new Error("db error");
    });
    const props = createProps();
    render(AddToPlaylistDialog, props);

    await waitFor(() => {
      expect(screen.getByText("お気に入り")).toBeInTheDocument();
    });
    await user.click(screen.getByText("お気に入り"));

    await waitFor(() => {
      expect(getToasts()).toHaveLength(1);
    });
    expect(getToasts()[0].type).toBe("error");
    expect(props.onClose).not.toHaveBeenCalled();
  });

  it("create_playlist が失敗した場合エラートーストを表示する", async () => {
    const user = userEvent.setup();
    mockIPC((cmd: string) => {
      if (cmd === "list_playlists") return MOCK_PLAYLISTS;
      if (cmd === "create_playlist") throw new Error("db error");
    });
    const props = createProps();
    render(AddToPlaylistDialog, props);

    await user.click(screen.getByText("+ 新規プレイリストを作成して追加"));
    const input = screen.getByPlaceholderText("新しいプレイリスト名");
    await user.type(input, "新規プレイリスト{Enter}");

    await waitFor(() => {
      expect(getToasts()).toHaveLength(1);
    });
    expect(getToasts()[0].type).toBe("error");
    expect(props.onClose).not.toHaveBeenCalled();
  });
});
