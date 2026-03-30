import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import Sidebar from "../../src/lib/components/Sidebar.svelte";
import type { Library, Tag } from "../../src/lib/types";

const MOCK_LIBRARIES: Library[] = [
  { id: "lib-1", name: "マイライブラリ", path: "/path/to/lib1" },
  { id: "lib-2", name: "サブライブラリ", path: "/path/to/lib2" },
];

const MOCK_TAGS: Tag[] = [
  { id: 1, name: "風景", category: "genre" },
  { id: 2, name: "人物", category: "genre" },
  { id: 3, name: "田中太郎", category: "artist" },
];

function createProps(overrides = {}) {
  return {
    activeLibrary: MOCK_LIBRARIES[0],
    currentView: "grid",
    reloadTrigger: 0,
    onSwitchLibrary: vi.fn(),
    onNavigate: vi.fn(),
    onTagSelect: vi.fn(),
    onNavigateToAppSettings: vi.fn(),
    selectedTagIds: [] as number[],
    ...overrides,
  };
}

beforeEach(() => {
  cleanup();
  mockIPC((cmd: string) => {
    if (cmd === "list_libraries") return MOCK_LIBRARIES;
    if (cmd === "list_tags") return MOCK_TAGS;
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
});
