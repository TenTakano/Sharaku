import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import BulkImportView from "../../src/lib/components/BulkImportView.svelte";
import type {
  AppSettings,
  DiscoverResult,
  UnregisteredEntry,
} from "../../src/lib/types";

const MOCK_SETTINGS: AppSettings = {
  resourceMode: "full",
  directoryTemplate: "{artist}/{title}",
  typeLabelImage: "Image",
  typeLabelFolder: "Folder",
  deleteFileAction: "ask",
};

const MOCK_DISCOVER_RESULT: DiscoverResult = {
  folders: [
    {
      path: "/root/folder1",
      folderName: "folder1",
      imageCount: 5,
      parsedMetadata: { title: "作品A", artist: "アーティストA" },
      alreadyRegistered: false,
    },
    {
      path: "/root/folder2",
      folderName: "folder2",
      imageCount: 3,
      parsedMetadata: { title: "作品B", artist: null },
      alreadyRegistered: true,
    },
  ],
  images: [
    {
      path: "/root/image1.jpg",
      fileName: "image1.jpg",
      parsedMetadata: { title: "画像1", artist: null },
      alreadyRegistered: false,
    },
  ],
  skippedFolders: [
    {
      path: "/root/intermediate",
      folderName: "intermediate",
      imageCount: 2,
    },
  ],
};

function createProps(overrides = {}) {
  return {
    onBack: vi.fn(),
    ...overrides,
  };
}

beforeEach(() => {
  cleanup();
  mockIPC((cmd: string) => {
    if (cmd === "get_settings") return MOCK_SETTINGS;
    if (cmd === "discover_folders") return MOCK_DISCOVER_RESULT;
    if (cmd === "discover_dropped_paths") return MOCK_DISCOVER_RESULT;
    if (cmd === "parse_folder_name")
      return { title: "パース結果", artist: null };
    if (cmd === "enqueue_import") return { jobId: "job-1" };
    if (cmd === "plugin:dialog|open") return "/root";
  });
});

describe("BulkImportView コンポーネント", () => {
  describe("探索ステップ", () => {
    it("フォルダ選択ボタンが表示される", () => {
      render(BulkImportView, createProps());
      expect(screen.getByText("フォルダを選択...")).toBeInTheDocument();
    });

    it("タイトルと説明が表示される", () => {
      render(BulkImportView, createProps());
      expect(screen.getByText("探索するフォルダを選択")).toBeInTheDocument();
    });
  });

  describe("initialRootPath でのフォルダ探索", () => {
    it("discover_folders の結果からフォルダ/画像混在のレビューが表示される", async () => {
      render(BulkImportView, createProps({ initialRootPath: "/root" }));

      await waitFor(() => {
        expect(screen.getByText("取り込み対象の確認")).toBeInTheDocument();
      });

      expect(screen.getByText("folder1")).toBeInTheDocument();
      expect(screen.getByText("folder2")).toBeInTheDocument();
      expect(screen.getByText("image1.jpg")).toBeInTheDocument();
      expect(screen.getAllByText("フォルダ")).toHaveLength(2);
      expect(screen.getAllByText("画像")).toHaveLength(1);
    });

    it("中間フォルダのスキップ警告が表示され詳細トグルで一覧が表示される", async () => {
      const user = userEvent.setup();
      const { container } = render(
        BulkImportView,
        createProps({ initialRootPath: "/root" }),
      );

      await waitFor(() => {
        expect(
          screen.getByText(
            /1 件の中間フォルダにある 2 枚の画像はスキップされました/,
          ),
        ).toBeInTheDocument();
      });

      expect(container.querySelector(".bulk-skipped-list")).toBeNull();

      await user.click(screen.getByText("詳細を表示"));

      const skippedList = container.querySelector(".bulk-skipped-list");
      expect(skippedList).not.toBeNull();
      expect(skippedList?.textContent).toContain("intermediate");
    });
  });

  describe("initialDroppedPaths でのドロップ探索", () => {
    it("discover_dropped_paths が呼ばれレビューステップに進む", async () => {
      render(
        BulkImportView,
        createProps({ initialDroppedPaths: ["/root/a", "/root/b"] }),
      );

      await waitFor(() => {
        expect(screen.getByText("取り込み対象の確認")).toBeInTheDocument();
        expect(screen.getByText(/3 件検出/)).toBeInTheDocument();
      });
    });
  });

  describe("initialEntries でレビューステップ", () => {
    const entries: UnregisteredEntry[] = [
      {
        path: "/root/unregistered1",
        folderName: "unregistered1",
        imageCount: 7,
      },
      {
        path: "/root/unregistered2",
        folderName: "unregistered2",
        imageCount: 4,
      },
    ];

    it("initialEntries が指定されている場合、直接レビューステップに進む", async () => {
      render(BulkImportView, createProps({ initialEntries: entries }));

      await waitFor(() => {
        expect(screen.getByText("取り込み対象の確認")).toBeInTheDocument();
      });
    });

    it("検出件数・選択件数・すべて選択チェックボックス・取り込みボタンが表示される", async () => {
      render(BulkImportView, createProps({ initialEntries: entries }));

      await waitFor(() => {
        expect(screen.getByText(/2 件検出/)).toBeInTheDocument();
        expect(screen.getByText(/2 件選択中/)).toBeInTheDocument();
        expect(screen.getByText("すべて選択")).toBeInTheDocument();
        expect(screen.getByText("2 件を取り込み")).toBeInTheDocument();
      });
    });

    it("取り込み実行で enqueue_import が呼ばれ onBack が呼ばれる", async () => {
      const user = userEvent.setup();
      const props = createProps({ initialEntries: entries });
      render(BulkImportView, props);

      await waitFor(() => {
        expect(screen.getByText("2 件を取り込み")).toBeInTheDocument();
      });

      await user.click(screen.getByText("2 件を取り込み"));

      await waitFor(() => {
        expect(props.onBack).toHaveBeenCalled();
      });
    });
  });
});
