import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  render,
  screen,
  cleanup,
  waitFor,
  fireEvent,
} from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import EditWorkDialog from "../../src/lib/components/EditWorkDialog.svelte";
import type { WorkDetail } from "../../src/lib/types";
import { getToasts, removeToast } from "../../src/lib/stores/toast.svelte";

function createWorkDetail(overrides: Partial<WorkDetail> = {}): WorkDetail {
  return {
    id: 1,
    title: "テスト作品",
    path: "/path/to/work",
    workType: "image",
    pageCount: 10,
    createdAt: "2025-01-01T00:00:00Z",
    artist: "テストアーティスト",
    year: 2025,
    genre: "イラスト",
    circle: null,
    origin: null,
    ...overrides,
  };
}

function createProps(overrides = {}) {
  return {
    work: createWorkDetail(),
    onClose: vi.fn(),
    onUpdated: vi.fn(),
    ...overrides,
  };
}

let updateWorkSpy: ReturnType<typeof vi.fn>;

beforeEach(() => {
  cleanup();
  for (const toast of getToasts()) {
    removeToast(toast.id);
  }
  updateWorkSpy = vi.fn();
  mockIPC((cmd: string) => {
    if (cmd === "update_work") return updateWorkSpy();
  });
});

describe("EditWorkDialog コンポーネント", () => {
  it("ダイアログタイトルが表示される", () => {
    render(EditWorkDialog, createProps());
    expect(screen.getByText("メタデータを編集")).toBeInTheDocument();
  });

  it("既存のメタデータがフォームに初期値として入力されている", () => {
    render(EditWorkDialog, createProps());

    expect(screen.getByPlaceholderText("タイトル")).toHaveValue("テスト作品");
    expect(screen.getByPlaceholderText("アーティスト")).toHaveValue(
      "テストアーティスト",
    );
    expect(screen.getByPlaceholderText("年")).toHaveValue(2025);
    expect(screen.getByPlaceholderText("ジャンル")).toHaveValue("イラスト");
  });

  it("null のフィールドは空文字で表示される", () => {
    render(
      EditWorkDialog,
      createProps({ work: createWorkDetail({ circle: null, origin: null }) }),
    );

    expect(screen.getByPlaceholderText("サークル")).toHaveValue("");
    expect(screen.getByPlaceholderText("出典")).toHaveValue("");
  });

  it("タイトルが空の場合は保存ボタンが無効になる", async () => {
    const user = userEvent.setup();
    render(EditWorkDialog, createProps());

    const titleInput = screen.getByPlaceholderText("タイトル");
    await user.clear(titleInput);

    expect(screen.getByText("保存")).toBeDisabled();
  });

  it("保存ボタンクリックで update_work が正しい引数で呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    mockIPC((cmd: string, args: Record<string, unknown>) => {
      if (cmd === "update_work") return updateWorkSpy(args);
    });
    render(EditWorkDialog, props);

    await user.click(screen.getByText("保存"));

    await waitFor(() => {
      expect(updateWorkSpy).toHaveBeenCalledWith({
        id: 1,
        title: "テスト作品",
        artist: "テストアーティスト",
        year: 2025,
        genre: "イラスト",
        circle: null,
        origin: null,
      });
    });
  });

  it("保存成功時に成功トーストが表示され onUpdated と onClose が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(EditWorkDialog, props);

    await user.click(screen.getByText("保存"));

    await waitFor(() => {
      const toasts = getToasts();
      expect(toasts.some((t) => t.type === "success")).toBe(true);
      expect(props.onUpdated).toHaveBeenCalledWith(1);
      expect(props.onClose).toHaveBeenCalled();
    });
  });

  it("保存失敗時にエラートーストが表示される", async () => {
    const user = userEvent.setup();
    mockIPC((cmd: string) => {
      if (cmd === "update_work") throw new Error("DB error");
    });
    render(EditWorkDialog, createProps());

    await user.click(screen.getByText("保存"));

    await waitFor(() => {
      const toasts = getToasts();
      expect(toasts.some((t) => t.type === "error")).toBe(true);
    });
  });

  it("Escape キーで onClose が呼ばれる", () => {
    const props = createProps();
    render(EditWorkDialog, props);

    fireEvent.keyDown(window, { key: "Escape" });

    expect(props.onClose).toHaveBeenCalledOnce();
  });

  it("オーバーレイクリックで onClose が呼ばれる", () => {
    const props = createProps();
    const { container } = render(EditWorkDialog, props);
    const overlay = container.querySelector(".edit-work-overlay")!;

    fireEvent.mouseDown(overlay);

    expect(props.onClose).toHaveBeenCalledOnce();
  });

  it("ダイアログ内部クリックでは onClose が呼ばれない", () => {
    const props = createProps();
    const { container } = render(EditWorkDialog, props);
    const dialog = container.querySelector(".edit-work-dialog")!;

    fireEvent.mouseDown(dialog);

    expect(props.onClose).not.toHaveBeenCalled();
  });
});
