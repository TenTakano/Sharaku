import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import { getToasts, removeToast } from "../../src/lib/stores/toast.svelte";
import type { PlaylistItem } from "../../src/lib/types";

vi.mock("@dnd-kit/svelte", async () => {
  const { default: DragDropProviderMock } =
    await import("../__mocks__/DragDropProviderMock.svelte");
  return { DragDropProvider: DragDropProviderMock };
});

vi.mock("@dnd-kit/svelte/sortable", () => ({
  createSortable: () => ({
    isDragging: false,
    isDropping: false,
    isDragSource: false,
    isDropTarget: false,
    attach: () => () => {},
    attachHandle: () => () => {},
  }),
}));

const { default: PlaylistView } =
  await import("../../src/lib/components/PlaylistView.svelte");
const { dragDropHandlers } =
  await import("../__mocks__/DragDropProviderMock.svelte");

const DUMMY_BYTES = [0x52, 0x49, 0x46, 0x46];

const MOCK_ITEMS: PlaylistItem[] = [
  {
    workId: 1,
    title: "作品A",
    workType: "image",
    pageCount: 3,
    createdAt: "2025-01-01T00:00:00Z",
  },
  {
    workId: 2,
    title: "作品B",
    workType: "image",
    pageCount: 5,
    createdAt: "2025-02-01T00:00:00Z",
  },
];

function createProps(overrides = {}) {
  return {
    playlistId: 1,
    onSelectWork: vi.fn(),
    onWorksLoaded: vi.fn(),
    ...overrides,
  };
}

beforeEach(() => {
  cleanup();
  for (const toast of getToasts()) {
    removeToast(toast.id);
  }
  mockIPC((cmd: string) => {
    if (cmd === "get_playlist_items") return MOCK_ITEMS;
    if (cmd === "get_thumbnail") return DUMMY_BYTES;
  });
});

function dragReorder(sourceWorkId: number, targetWorkId: number) {
  const operation = {
    source: { id: sourceWorkId },
    target: { id: targetWorkId },
  };
  dragDropHandlers.onDragOver?.({ operation });
  dragDropHandlers.onDragEnd?.({ operation, canceled: false });
}

describe("PlaylistView コンポーネント", () => {
  it("プレイリストの作品一覧を表示する", async () => {
    render(PlaylistView, createProps());

    await waitFor(() => {
      expect(screen.getByText("作品A")).toBeInTheDocument();
      expect(screen.getByText("作品B")).toBeInTheDocument();
    });
  });

  it("読み込み完了時に onWorksLoaded が並び順の workId 配列で呼ばれる", async () => {
    const props = createProps();
    render(PlaylistView, props);

    await waitFor(() => {
      expect(props.onWorksLoaded).toHaveBeenCalledWith([1, 2]);
    });
  });

  it("作品カードのクリックで onSelectWork が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(PlaylistView, props);

    await waitFor(() => {
      expect(screen.getByText("作品A")).toBeInTheDocument();
    });
    await user.click(screen.getByText("作品A"));

    expect(props.onSelectWork).toHaveBeenCalledWith(1);
  });

  it("作品が空の場合は空状態メッセージを表示する", async () => {
    mockIPC((cmd: string) => {
      if (cmd === "get_playlist_items") return [];
    });
    render(PlaylistView, createProps());

    await waitFor(() => {
      expect(
        screen.getByText(/このプレイリストには作品がありません/),
      ).toBeInTheDocument();
    });
  });

  it("削除ボタン→確認で remove_item_from_playlist が呼ばれ、一覧から消える", async () => {
    const user = userEvent.setup();
    const removeSpy = vi.fn();
    mockIPC((cmd: string) => {
      if (cmd === "get_playlist_items") return MOCK_ITEMS;
      if (cmd === "get_thumbnail") return DUMMY_BYTES;
      if (cmd === "remove_item_from_playlist") return removeSpy();
    });
    render(PlaylistView, createProps());

    await waitFor(() => {
      expect(screen.getByText("作品A")).toBeInTheDocument();
    });

    const removeButtons = screen.getAllByLabelText("プレイリストから削除");
    await user.click(removeButtons[0]);

    await waitFor(() => {
      expect(
        screen.getByText("「作品A」をプレイリストから削除しますか？"),
      ).toBeInTheDocument();
    });
    await user.click(screen.getByText("削除", { selector: "button" }));

    await waitFor(() => {
      expect(removeSpy).toHaveBeenCalledOnce();
    });
  });

  it("削除確認ダイアログのキャンセルで remove_item_from_playlist が呼ばれない", async () => {
    const user = userEvent.setup();
    const removeSpy = vi.fn();
    mockIPC((cmd: string) => {
      if (cmd === "get_playlist_items") return MOCK_ITEMS;
      if (cmd === "get_thumbnail") return DUMMY_BYTES;
      if (cmd === "remove_item_from_playlist") return removeSpy();
    });
    render(PlaylistView, createProps());

    await waitFor(() => {
      expect(screen.getByText("作品A")).toBeInTheDocument();
    });

    const removeButtons = screen.getAllByLabelText("プレイリストから削除");
    await user.click(removeButtons[0]);

    await waitFor(() => {
      expect(screen.getByText("キャンセル")).toBeInTheDocument();
    });
    await user.click(screen.getByText("キャンセル"));

    expect(removeSpy).not.toHaveBeenCalled();
  });

  it("ドラッグ&ドロップで並び替えると新しい順序で reorder_playlist_items が呼ばれる", async () => {
    const reorderSpy = vi.fn();
    const props = createProps();
    mockIPC((cmd: string) => {
      if (cmd === "get_playlist_items") return MOCK_ITEMS;
      if (cmd === "get_thumbnail") return DUMMY_BYTES;
      if (cmd === "reorder_playlist_items") return reorderSpy();
    });
    render(PlaylistView, props);

    await waitFor(() => {
      expect(screen.getByText("作品A")).toBeInTheDocument();
    });

    dragReorder(1, 2);

    await waitFor(() => {
      expect(reorderSpy).toHaveBeenCalledOnce();
      expect(props.onWorksLoaded).toHaveBeenLastCalledWith([2, 1]);
    });
  });

  it("ドラッグがキャンセルされた場合は reorder_playlist_items を呼ばず再取得する", async () => {
    const reorderSpy = vi.fn();
    const getItemsSpy = vi.fn(() => MOCK_ITEMS);
    mockIPC((cmd: string) => {
      if (cmd === "get_playlist_items") return getItemsSpy();
      if (cmd === "get_thumbnail") return DUMMY_BYTES;
      if (cmd === "reorder_playlist_items") return reorderSpy();
    });
    render(PlaylistView, createProps());

    await waitFor(() => {
      expect(screen.getByText("作品A")).toBeInTheDocument();
    });
    expect(getItemsSpy).toHaveBeenCalledOnce();

    const operation = { source: { id: 1 }, target: { id: 2 } };
    dragDropHandlers.onDragOver?.({ operation });
    dragDropHandlers.onDragEnd?.({ operation, canceled: true });

    await waitFor(() => {
      expect(getItemsSpy).toHaveBeenCalledTimes(2);
    });
    expect(reorderSpy).not.toHaveBeenCalled();
  });

  it("並び替え失敗時はエラートーストを表示し元の順序に復元する", async () => {
    const getItemsSpy = vi.fn(() => MOCK_ITEMS);
    mockIPC((cmd: string) => {
      if (cmd === "get_playlist_items") return getItemsSpy();
      if (cmd === "get_thumbnail") return DUMMY_BYTES;
      if (cmd === "reorder_playlist_items") throw new Error("db error");
    });
    render(PlaylistView, createProps());

    await waitFor(() => {
      expect(screen.getByText("作品A")).toBeInTheDocument();
    });
    expect(getItemsSpy).toHaveBeenCalledOnce();

    dragReorder(1, 2);

    await waitFor(() => {
      expect(getToasts()).toHaveLength(1);
    });
    expect(getToasts()[0].type).toBe("error");
    await waitFor(() => {
      expect(getItemsSpy).toHaveBeenCalledTimes(2);
    });
  });
});
