import { describe, it, expect, vi, beforeEach, afterAll } from "vitest";
import {
  render,
  screen,
  cleanup,
  waitFor,
  fireEvent,
} from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import type { WorkSummary, Tag, AppSettings } from "../../src/lib/types";
import { getToasts, removeToast } from "../../src/lib/stores/toast.svelte";

globalThis.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof globalThis.ResizeObserver;

vi.mock("virtua/svelte", async () => {
  const VListMock = (await import("../__mocks__/VListMock.svelte")).default;
  return { VList: VListMock };
});

const { default: WorkGrid } =
  await import("../../src/lib/components/WorkGrid.svelte");

const DUMMY_BYTES = [0x52, 0x49, 0x46, 0x46];

const MOCK_WORKS: WorkSummary[] = [
  {
    id: 1,
    title: "風景画A",
    workType: "image",
    pageCount: 5,
    createdAt: "2025-01-01T00:00:00Z",
  },
  {
    id: 2,
    title: "人物画B",
    workType: "image",
    pageCount: 3,
    createdAt: "2025-02-01T00:00:00Z",
  },
];

const MOCK_SETTINGS: AppSettings = {
  deleteFileAction: "ask",
};

const originalCreateObjectURL = globalThis.URL.createObjectURL;
const originalRevokeObjectURL = globalThis.URL.revokeObjectURL;

let urlCounter = 0;
const createObjectURLMock = vi.fn(() => `blob:mock-url-${++urlCounter}`);
const revokeObjectURLMock = vi.fn();

afterAll(() => {
  globalThis.URL.createObjectURL = originalCreateObjectURL;
  globalThis.URL.revokeObjectURL = originalRevokeObjectURL;
});

function createProps(overrides = {}) {
  return {
    reloadTrigger: 0,
    filterTags: [] as Tag[],
    tagSearchMode: "and" as const,
    onSelectWork: vi.fn(),
    onWorksLoaded: vi.fn(),
    onFilterTagsChange: vi.fn(),
    onTagSearchModeChange: vi.fn(),
    ...overrides,
  };
}

beforeEach(() => {
  cleanup();
  for (const toast of getToasts()) {
    removeToast(toast.id);
  }
  urlCounter = 0;
  createObjectURLMock.mockClear();
  revokeObjectURLMock.mockClear();
  globalThis.URL.createObjectURL = createObjectURLMock;
  globalThis.URL.revokeObjectURL = revokeObjectURLMock;

  mockIPC((cmd: string) => {
    if (cmd === "list_works") return MOCK_WORKS;
    if (cmd === "search_works_by_tags") return MOCK_WORKS;
    if (cmd === "get_thumbnail") return DUMMY_BYTES;
    if (cmd === "get_settings") return MOCK_SETTINGS;
    if (cmd === "get_work")
      return {
        ...MOCK_WORKS[0],
        path: "/path",
        artist: null,
        year: null,
        genre: null,
        circle: null,
        origin: null,
      };
    if (cmd === "delete_work") return undefined;
    if (cmd === "search_tags") return [];
    if (cmd === "list_playlists") return [];
  });
});

describe("WorkGrid コンポーネント", () => {
  it("作品一覧が表示される", async () => {
    render(WorkGrid, createProps());

    await waitFor(() => {
      expect(screen.getByText("風景画A")).toBeInTheDocument();
      expect(screen.getByText("人物画B")).toBeInTheDocument();
    });
  });

  it("作品数が表示される", async () => {
    render(WorkGrid, createProps());

    await waitFor(() => {
      expect(screen.getByText("2 works")).toBeInTheDocument();
    });
  });

  it("作品がない場合に空状態メッセージが表示される", async () => {
    mockIPC((cmd: string) => {
      if (cmd === "list_works") return [];
      if (cmd === "search_tags") return [];
    });
    render(WorkGrid, createProps());

    await waitFor(() => {
      expect(screen.getByText(/作品がありません/)).toBeInTheDocument();
    });
  });

  it("ソート選択が表示される", () => {
    render(WorkGrid, createProps());
    expect(screen.getByLabelText("Sort:")).toBeInTheDocument();
  });

  it("タグフィルターバッジが表示される", () => {
    const filterTags: Tag[] = [{ id: 1, name: "風景", category: "genre" }];
    render(WorkGrid, createProps({ filterTags }));
    expect(screen.getByText("風景")).toBeInTheDocument();
  });

  it("タグバッジの×ボタンで onFilterTagsChange が呼ばれる", async () => {
    const user = userEvent.setup();
    const filterTags: Tag[] = [{ id: 1, name: "風景", category: "genre" }];
    const props = createProps({ filterTags });
    const { container } = render(WorkGrid, props);

    const removeBtn = container.querySelector(".tag-badge-remove")!;
    await user.click(removeBtn as HTMLElement);

    expect(props.onFilterTagsChange).toHaveBeenCalledWith([]);
  });

  it("フィルタータグが2つ以上の場合にAND/ORトグルが表示される", () => {
    const filterTags: Tag[] = [
      { id: 1, name: "風景", category: "genre" },
      { id: 2, name: "人物", category: "genre" },
    ];
    render(WorkGrid, createProps({ filterTags }));
    expect(screen.getByText("AND")).toBeInTheDocument();
  });

  it("AND/ORトグルクリックで onTagSearchModeChange が呼ばれる", async () => {
    const user = userEvent.setup();
    const filterTags: Tag[] = [
      { id: 1, name: "風景", category: "genre" },
      { id: 2, name: "人物", category: "genre" },
    ];
    const props = createProps({ filterTags });
    render(WorkGrid, props);

    await user.click(screen.getByText("AND"));
    expect(props.onTagSearchModeChange).toHaveBeenCalledWith("or");
  });

  it("作品クリックで onSelectWork が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(WorkGrid, props);

    await waitFor(() => {
      expect(screen.getByText("風景画A")).toBeInTheDocument();
    });

    const workButtons = screen.getAllByRole("button", { name: /風景画A/ });
    await user.click(workButtons[0]);

    expect(props.onSelectWork).toHaveBeenCalledWith(1);
  });

  it("右クリックでコンテキストメニューが表示される", async () => {
    render(WorkGrid, createProps());

    await waitFor(() => {
      expect(screen.getByText("風景画A")).toBeInTheDocument();
    });

    const workButtons = screen.getAllByRole("button", { name: /風景画A/ });
    fireEvent.contextMenu(workButtons[0]);

    await waitFor(() => {
      expect(screen.getByText("メタデータを編集")).toBeInTheDocument();
      expect(screen.getByText("プレイリストに追加")).toBeInTheDocument();
      expect(screen.getByText("削除")).toBeInTheDocument();
    });
  });

  it("コンテキストメニューから「プレイリストに追加」でダイアログを開ける", async () => {
    const user = userEvent.setup();
    mockIPC((cmd: string) => {
      if (cmd === "list_works") return MOCK_WORKS;
      if (cmd === "get_thumbnail") return DUMMY_BYTES;
      if (cmd === "list_playlists") return [{ id: 1, name: "お気に入り" }];
    });
    render(WorkGrid, createProps());

    await waitFor(() => {
      expect(screen.getByText("風景画A")).toBeInTheDocument();
    });

    const workButtons = screen.getAllByRole("button", { name: /風景画A/ });
    fireEvent.contextMenu(workButtons[0]);

    await waitFor(() => {
      expect(screen.getByText("プレイリストに追加")).toBeInTheDocument();
    });
    await user.click(screen.getByText("プレイリストに追加"));

    await waitFor(() => {
      expect(
        screen.getByText("プレイリストに追加", { selector: "h3" }),
      ).toBeInTheDocument();
      expect(screen.getByText("お気に入り")).toBeInTheDocument();
    });
  });

  it("コンテキストメニューから削除確認ダイアログを開ける", async () => {
    const user = userEvent.setup();
    render(WorkGrid, createProps());

    await waitFor(() => {
      expect(screen.getByText("風景画A")).toBeInTheDocument();
    });

    const workButtons = screen.getAllByRole("button", { name: /風景画A/ });
    fireEvent.contextMenu(workButtons[0]);

    await waitFor(() => {
      expect(screen.getByText("削除")).toBeInTheDocument();
    });

    await user.click(screen.getByText("削除"));

    await waitFor(() => {
      expect(screen.getByText("作品の削除")).toBeInTheDocument();
      expect(screen.getByText(/を削除しますか/)).toBeInTheDocument();
    });
  });
});
