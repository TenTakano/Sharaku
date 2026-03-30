import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  render,
  screen,
  cleanup,
  waitFor,
  fireEvent,
} from "@testing-library/svelte";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import WorkViewer from "../../src/lib/components/WorkViewer.svelte";
import type { WorkDetail, Tag } from "../../src/lib/types";

globalThis.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
} as unknown as typeof globalThis.ResizeObserver;

const MOCK_WORK: WorkDetail = {
  id: 1,
  title: "テスト作品",
  path: "/path/to/work",
  workType: "image",
  pageCount: 3,
  createdAt: "2025-01-01T00:00:00Z",
  artist: "テストアーティスト",
  year: 2025,
  genre: null,
  circle: null,
  origin: null,
};

const MOCK_TAGS: Tag[] = [
  { id: 10, name: "風景", category: "genre" },
  { id: 11, name: "人物", category: "genre" },
];

function createProps(overrides = {}) {
  return {
    workId: 1,
    workIds: [1, 2, 3],
    libraryName: "マイライブラリ",
    onBack: vi.fn(),
    onNavigateWork: vi.fn(),
    ...overrides,
  };
}

beforeEach(() => {
  cleanup();
  mockWindows("main");
  mockIPC((cmd: string) => {
    if (cmd === "get_work") return MOCK_WORK;
    if (cmd === "get_tags_for_work") return MOCK_TAGS;
    if (cmd === "add_tag_to_work") return undefined;
    if (cmd === "remove_tag_from_work") return undefined;
    if (cmd === "create_tag") return { id: 99, name: "新タグ", category: null };
    if (cmd === "search_tags") return [];
    if (cmd === "update_work") return undefined;
    if (cmd === "plugin:window|is_fullscreen") return false;
    if (cmd === "plugin:window|set_fullscreen") return undefined;
  });
});

describe("WorkViewer コンポーネント", () => {
  describe("基本表示", () => {
    it("作品タイトルが表示される", async () => {
      render(WorkViewer, createProps());
      await waitFor(() => {
        expect(screen.getByText("テスト作品")).toBeInTheDocument();
      });
    });

    it("ライブラリ名の戻るボタンが表示される", () => {
      render(WorkViewer, createProps());
      expect(screen.getByText(/マイライブラリ/)).toBeInTheDocument();
    });

    it("ページ情報が表示される", async () => {
      render(WorkViewer, createProps());
      await waitFor(() => {
        expect(screen.getByText(/1\/3/)).toBeInTheDocument();
      });
    });

    it("複数作品モードで作品情報が表示される", async () => {
      render(WorkViewer, createProps());
      await waitFor(() => {
        expect(screen.getByText(/作品 1\/3/)).toBeInTheDocument();
      });
    });

    it("フィットモードボタンが表示される", () => {
      render(WorkViewer, createProps());
      expect(screen.getByTitle("幅フィット (W)")).toBeInTheDocument();
      expect(screen.getByTitle("高さフィット (H)")).toBeInTheDocument();
      expect(screen.getByTitle("画面フィット (S)")).toBeInTheDocument();
    });

    it("ズーム率が表示される", () => {
      render(WorkViewer, createProps());
      expect(screen.getByText("100%")).toBeInTheDocument();
    });
  });

  describe("ページナビゲーション", () => {
    it("次へボタンクリックでページが進む", async () => {
      const user = (
        await import("@testing-library/user-event")
      ).default.setup();
      render(WorkViewer, createProps());
      await waitFor(() => {
        expect(screen.getByText(/1\/3/)).toBeInTheDocument();
      });

      await user.click(screen.getByTitle("次へ"));

      await waitFor(() => {
        expect(screen.getByText(/2\/3/)).toBeInTheDocument();
      });
    });

    it("前へボタンクリックでページが戻る", async () => {
      const user = (
        await import("@testing-library/user-event")
      ).default.setup();
      render(WorkViewer, createProps());
      await waitFor(() => {
        expect(screen.getByText(/1\/3/)).toBeInTheDocument();
      });

      await user.click(screen.getByTitle("次へ"));
      await waitFor(() => {
        expect(screen.getByText(/2\/3/)).toBeInTheDocument();
      });

      await user.click(screen.getByTitle("前へ"));
      await waitFor(() => {
        expect(screen.getByText(/1\/3/)).toBeInTheDocument();
      });
    });
  });

  describe("キーボードショートカット", () => {
    it("+ キーでズームインする", async () => {
      render(WorkViewer, createProps());

      fireEvent.keyDown(window, { key: "+" });

      await waitFor(() => {
        expect(screen.getByText("125%")).toBeInTheDocument();
      });
    });

    it("- キーでズームアウトする", async () => {
      render(WorkViewer, createProps());

      fireEvent.keyDown(window, { key: "+" });
      fireEvent.keyDown(window, { key: "+" });
      fireEvent.keyDown(window, { key: "-" });

      await waitFor(() => {
        expect(screen.getByText("125%")).toBeInTheDocument();
      });
    });

    it("0 キーでズームがリセットされる", async () => {
      render(WorkViewer, createProps());

      fireEvent.keyDown(window, { key: "+" });
      fireEvent.keyDown(window, { key: "0" });

      await waitFor(() => {
        expect(screen.getByText("100%")).toBeInTheDocument();
      });
    });

    it("Space キーでスライドショーが開始される", async () => {
      render(WorkViewer, createProps());

      fireEvent.keyDown(window, { key: " " });

      await waitFor(() => {
        expect(screen.getByText("⏸")).toBeInTheDocument();
      });
    });

    it("Escape キーで onBack が呼ばれる", () => {
      const props = createProps();
      render(WorkViewer, props);

      fireEvent.keyDown(window, { key: "Escape" });

      expect(props.onBack).toHaveBeenCalled();
    });
  });

  describe("タグ管理", () => {
    it("作品のタグが表示される", async () => {
      render(WorkViewer, createProps());

      await waitFor(() => {
        expect(screen.getByText("風景")).toBeInTheDocument();
        expect(screen.getByText("人物")).toBeInTheDocument();
      });
    });

    it("タグの×ボタンクリックで削除確認が表示される", async () => {
      const { container } = render(WorkViewer, createProps());

      await waitFor(() => {
        expect(screen.getByText("風景")).toBeInTheDocument();
      });

      const removeBtns = container.querySelectorAll(".tag-badge-remove");
      fireEvent.click(removeBtns[0] as HTMLElement);

      await waitFor(() => {
        expect(
          screen.getByText(/タグ「風景」を削除しますか/),
        ).toBeInTheDocument();
      });
    });
  });

  describe("編集ダイアログ", () => {
    it("Edit ボタンで編集ダイアログが表示される", async () => {
      const user = (
        await import("@testing-library/user-event")
      ).default.setup();
      render(WorkViewer, createProps());

      await waitFor(() => {
        expect(screen.getByText("テスト作品")).toBeInTheDocument();
      });

      await user.click(screen.getByTitle("メタデータを編集"));

      await waitFor(() => {
        expect(screen.getByText("メタデータを編集")).toBeInTheDocument();
      });
    });
  });
});
