import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import BulkImportView from "../../src/lib/components/BulkImportView.svelte";
import type { DiscoverResult } from "../../src/lib/types";

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

const SINGLE_FOLDER_RESULT: DiscoverResult = {
  folders: [
    {
      path: "/root/unregistered1",
      folderName: "unregistered1",
      imageCount: 7,
      parsedMetadata: { title: "パース結果", artist: null },
      alreadyRegistered: false,
    },
  ],
  images: [],
  skippedFolders: [],
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

  describe("メタデータ列（年・ジャンル・サークル・出典）", () => {
    beforeEach(() => {
      mockIPC((cmd: string) => {
        if (cmd === "discover_folders") return SINGLE_FOLDER_RESULT;
        if (cmd === "parse_folder_name")
          return { title: "パース結果", artist: null };
        if (cmd === "enqueue_import") return { jobId: "job-1" };
      });
    });

    it("常に年・ジャンル・サークル・出典の4列が表示される", async () => {
      render(BulkImportView, createProps({ initialRootPath: "/root" }));

      await waitFor(() => {
        expect(screen.getByText("取り込み対象の確認")).toBeInTheDocument();
      });

      expect(screen.getByText("年")).toBeInTheDocument();
      expect(screen.getByText("ジャンル")).toBeInTheDocument();
      expect(screen.getByText("サークル")).toBeInTheDocument();
      expect(screen.getByText("出典")).toBeInTheDocument();
    });

    it("入力した値が ImportRequest に反映され、年は数値変換される", async () => {
      const user = userEvent.setup();
      let capturedRequests: Array<{
        year: number | null;
        circle: string | null;
      }> = [];
      mockIPC((cmd: string, args: Record<string, unknown>) => {
        if (cmd === "discover_folders") return SINGLE_FOLDER_RESULT;
        if (cmd === "parse_folder_name")
          return { title: "パース結果", artist: null };
        if (cmd === "enqueue_import") {
          capturedRequests = args.requests as typeof capturedRequests;
          return { jobId: "job-1" };
        }
      });

      render(BulkImportView, createProps({ initialRootPath: "/root" }));

      await waitFor(() => {
        expect(screen.getByText("年")).toBeInTheDocument();
      });

      const yearInput = screen.getByLabelText("年");
      const circleInput = screen.getByLabelText("サークル");

      await user.type(yearInput, "2020");
      await user.type(circleInput, "テストサークル");
      await user.click(screen.getByText("1 件を取り込み"));

      await waitFor(() => {
        expect(capturedRequests).toHaveLength(1);
      });
      expect(capturedRequests[0].year).toBe(2020);
      expect(capturedRequests[0].circle).toBe("テストサークル");
    });

    it("genre・origin を含む全メタデータ列の入力値が ImportRequest に正しくマッピングされる", async () => {
      const user = userEvent.setup();
      let capturedRequests: Array<{
        year: number | null;
        genre: string | null;
        circle: string | null;
        origin: string | null;
      }> = [];
      mockIPC((cmd: string, args: Record<string, unknown>) => {
        if (cmd === "discover_folders") return SINGLE_FOLDER_RESULT;
        if (cmd === "parse_folder_name")
          return { title: "パース結果", artist: null };
        if (cmd === "enqueue_import") {
          capturedRequests = args.requests as typeof capturedRequests;
          return { jobId: "job-1" };
        }
      });

      render(BulkImportView, createProps({ initialRootPath: "/root" }));

      await waitFor(() => {
        expect(screen.getByText("ジャンル")).toBeInTheDocument();
      });

      await user.type(screen.getByLabelText("ジャンル"), "テストジャンル");
      await user.type(screen.getByLabelText("サークル"), "テストサークル");
      await user.type(screen.getByLabelText("出典"), "テスト出典");
      await user.type(screen.getByLabelText("年"), "2021");
      await user.click(screen.getByText("1 件を取り込み"));

      await waitFor(() => {
        expect(capturedRequests).toHaveLength(1);
      });
      expect(capturedRequests[0].genre).toBe("テストジャンル");
      expect(capturedRequests[0].circle).toBe("テストサークル");
      expect(capturedRequests[0].origin).toBe("テスト出典");
      expect(capturedRequests[0].year).toBe(2021);
    });

    it("genre・origin が空白のみの場合は trim されて null として送信される", async () => {
      const user = userEvent.setup();
      let capturedRequests: Array<{
        genre: string | null;
        origin: string | null;
      }> = [];
      mockIPC((cmd: string, args: Record<string, unknown>) => {
        if (cmd === "discover_folders") return SINGLE_FOLDER_RESULT;
        if (cmd === "parse_folder_name")
          return { title: "パース結果", artist: null };
        if (cmd === "enqueue_import") {
          capturedRequests = args.requests as typeof capturedRequests;
          return { jobId: "job-1" };
        }
      });

      render(BulkImportView, createProps({ initialRootPath: "/root" }));

      await waitFor(() => {
        expect(screen.getByText("ジャンル")).toBeInTheDocument();
      });

      await user.type(screen.getByLabelText("ジャンル"), "   ");
      await user.type(screen.getByLabelText("出典"), "   ");
      await user.click(screen.getByText("1 件を取り込み"));

      await waitFor(() => {
        expect(capturedRequests).toHaveLength(1);
      });
      expect(capturedRequests[0].genre).toBeNull();
      expect(capturedRequests[0].origin).toBeNull();
    });

    it("非数値の年は null として送信される", async () => {
      const user = userEvent.setup();
      let capturedRequests: Array<{ year: number | null }> = [];
      mockIPC((cmd: string, args: Record<string, unknown>) => {
        if (cmd === "discover_folders") return SINGLE_FOLDER_RESULT;
        if (cmd === "parse_folder_name")
          return { title: "パース結果", artist: null };
        if (cmd === "enqueue_import") {
          capturedRequests = args.requests as typeof capturedRequests;
          return { jobId: "job-1" };
        }
      });

      render(BulkImportView, createProps({ initialRootPath: "/root" }));

      await waitFor(() => {
        expect(screen.getByText("年")).toBeInTheDocument();
      });

      const yearInput = screen.getByLabelText("年");

      await user.type(yearInput, "非数値");
      await user.click(screen.getByText("1 件を取り込み"));

      await waitFor(() => {
        expect(capturedRequests).toHaveLength(1);
      });
      expect(capturedRequests[0].year).toBeNull();
    });
  });
});
