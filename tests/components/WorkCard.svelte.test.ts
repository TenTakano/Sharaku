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
import WorkCardComponent, {
  WorkCard,
} from "../../src/lib/components/WorkCard.svelte";
import type { WorkSummary } from "../../src/lib/types";

const DUMMY_BYTES = [0x52, 0x49, 0x46, 0x46];

const originalCreateObjectURL = globalThis.URL.createObjectURL;
const originalRevokeObjectURL = globalThis.URL.revokeObjectURL;

let urlCounter = 0;
const createObjectURLMock = vi.fn(() => `blob:mock-url-${++urlCounter}`);
const revokeObjectURLMock = vi.fn();

afterAll(() => {
  globalThis.URL.createObjectURL = originalCreateObjectURL;
  globalThis.URL.revokeObjectURL = originalRevokeObjectURL;
});

function createWorkSummary(overrides: Partial<WorkSummary> = {}): WorkSummary {
  return {
    id: 1,
    title: "テスト作品",
    workType: "image",
    pageCount: 10,
    createdAt: "2025-01-01T00:00:00Z",
    ...overrides,
  };
}

function createProps(overrides = {}) {
  return {
    work: createWorkSummary(),
    onclick: vi.fn(),
    oncontextmenu: vi.fn(),
    ...overrides,
  };
}

beforeEach(() => {
  cleanup();
  WorkCard._thumbnailCache.clear();
  urlCounter = 0;
  createObjectURLMock.mockClear();
  revokeObjectURLMock.mockClear();
  globalThis.URL.createObjectURL = createObjectURLMock;
  globalThis.URL.revokeObjectURL = revokeObjectURLMock;

  mockIPC((cmd: string) => {
    if (cmd === "get_thumbnail") {
      return DUMMY_BYTES;
    }
  });
});

describe("WorkCard コンポーネント", () => {
  it("作品タイトルが表示される", () => {
    const props = createProps({ work: createWorkSummary({ title: "風景画" }) });
    render(WorkCardComponent, props);

    expect(screen.getByText("風景画")).toBeInTheDocument();
  });

  it("サムネイル読み込み中はローディング表示される", async () => {
    mockIPC((cmd: string) => {
      if (cmd === "get_thumbnail") {
        return new Promise<number[]>(() => {});
      }
    });
    const props = createProps();
    const { container } = render(WorkCardComponent, props);

    await waitFor(() => {
      expect(container.querySelector(".no-thumbnail")).toBeInTheDocument();
    });
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
    expect(screen.queryByText("No Image")).not.toBeInTheDocument();
  });

  it("サムネイル取得成功時に画像が表示される", async () => {
    const props = createProps();
    render(WorkCardComponent, props);

    const img = await screen.findByRole("img");
    expect(img).toHaveAttribute("alt", "テスト作品");
    expect(img).toHaveAttribute("src", "blob:mock-url-1");
    expect(createObjectURLMock).toHaveBeenCalledOnce();
  });

  it("サムネイル取得失敗時に「No Image」が表示される", async () => {
    mockIPC((cmd: string) => {
      if (cmd === "get_thumbnail") {
        throw new Error("not found");
      }
    });
    const props = createProps();
    render(WorkCardComponent, props);

    await waitFor(() => {
      expect(screen.getByText("No Image")).toBeInTheDocument();
    });
    expect(screen.queryByRole("img")).not.toBeInTheDocument();
  });

  it("同じ work.id の2回目のレンダリングではキャッシュが使われる", async () => {
    const getThumbnailSpy = vi.fn(() => DUMMY_BYTES);
    mockIPC((cmd: string) => {
      if (cmd === "get_thumbnail") {
        return getThumbnailSpy();
      }
    });

    const props = createProps();
    render(WorkCardComponent, props);
    await screen.findByRole("img");
    cleanup();

    render(WorkCardComponent, props);
    await screen.findByRole("img");

    expect(getThumbnailSpy).toHaveBeenCalledOnce();
  });

  it("クリック時に onclick が workId 付きで呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps({ work: createWorkSummary({ id: 42 }) });
    render(WorkCardComponent, props);

    await user.click(screen.getByRole("button"));

    expect(props.onclick).toHaveBeenCalledWith(42);
  });

  it("右クリック時に oncontextmenu が workId と MouseEvent 付きで呼ばれる", () => {
    const props = createProps({ work: createWorkSummary({ id: 7 }) });
    render(WorkCardComponent, props);

    fireEvent.contextMenu(screen.getByRole("button"));

    expect(props.oncontextmenu).toHaveBeenCalledWith(7, expect.any(MouseEvent));
  });

  it("clearCache で URL.revokeObjectURL が呼ばれキャッシュがクリアされる", async () => {
    const props = createProps();
    render(WorkCardComponent, props);
    await screen.findByRole("img");

    expect(WorkCard._thumbnailCache.size).toBe(1);

    WorkCard.clearCache();

    expect(revokeObjectURLMock).toHaveBeenCalledOnce();
    expect(WorkCard._thumbnailCache.size).toBe(0);
  });
});
