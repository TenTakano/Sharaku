import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import BulkImportView from "../../src/lib/components/BulkImportView.svelte";
import type {
  AppSettings,
  DiscoveredFolder,
  UnregisteredEntry,
} from "../../src/lib/types";

const MOCK_SETTINGS: AppSettings = {
  resourceMode: "full",
  directoryTemplate: "{artist}/{title}",
  typeLabelImage: "Image",
  typeLabelFolder: "Folder",
  deleteFileAction: "ask",
};

const MOCK_FOLDERS: DiscoveredFolder[] = [
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
  {
    path: "/root/folder3",
    folderName: "folder3",
    imageCount: 10,
    parsedMetadata: { title: "作品C", artist: "アーティストC" },
    alreadyRegistered: false,
  },
];

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
    if (cmd === "discover_folders") return MOCK_FOLDERS;
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

    it("検出フォルダ数と選択件数が表示される", async () => {
      render(BulkImportView, createProps({ initialEntries: entries }));

      await waitFor(() => {
        expect(screen.getByText(/2 フォルダ検出/)).toBeInTheDocument();
        expect(screen.getByText(/2 件選択中/)).toBeInTheDocument();
      });
    });

    it("すべて選択チェックボックスが表示される", async () => {
      render(BulkImportView, createProps({ initialEntries: entries }));

      await waitFor(() => {
        expect(screen.getByText("すべて選択")).toBeInTheDocument();
      });
    });

    it("取り込みボタンに選択件数が表示される", async () => {
      render(BulkImportView, createProps({ initialEntries: entries }));

      await waitFor(() => {
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
