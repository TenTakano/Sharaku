import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, fireEvent } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import ConfirmDialog from "../../src/lib/components/ConfirmDialog.svelte";

function createProps(overrides = {}) {
  return {
    title: "確認",
    message: "この操作を実行しますか？",
    confirmLabel: "実行",
    onConfirm: vi.fn(),
    onCancel: vi.fn(),
    ...overrides,
  };
}

beforeEach(() => {
  cleanup();
});

describe("ConfirmDialog コンポーネント", () => {
  it("タイトル、メッセージ、ボタンが正しく表示される", () => {
    const props = createProps();
    render(ConfirmDialog, props);

    expect(screen.getByText("確認")).toBeInTheDocument();
    expect(screen.getByText("この操作を実行しますか？")).toBeInTheDocument();
    expect(screen.getByText("実行")).toBeInTheDocument();
    expect(screen.getByText("キャンセル")).toBeInTheDocument();
  });

  it("確認ボタンをクリックすると onConfirm が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(ConfirmDialog, props);

    await user.click(screen.getByText("実行"));

    expect(props.onConfirm).toHaveBeenCalledOnce();
    expect(props.onCancel).not.toHaveBeenCalled();
  });

  it("キャンセルボタンをクリックすると onCancel が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(ConfirmDialog, props);

    await user.click(screen.getByText("キャンセル"));

    expect(props.onCancel).toHaveBeenCalled();
    expect(props.onConfirm).not.toHaveBeenCalled();
  });

  it("Escape キーを押すと onCancel が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(ConfirmDialog, props);

    await user.keyboard("{Escape}");

    expect(props.onCancel).toHaveBeenCalledOnce();
  });

  it("オーバーレイをクリックすると onCancel が呼ばれる", () => {
    const props = createProps();
    const { container } = render(ConfirmDialog, props);

    const overlay = container.querySelector(".confirm-dialog-overlay")!;
    fireEvent.mouseDown(overlay);

    expect(props.onCancel).toHaveBeenCalledOnce();
  });

  it("ダイアログ内部をクリックしても onCancel が呼ばれない", () => {
    const props = createProps();
    const { container } = render(ConfirmDialog, props);

    const dialog = container.querySelector(".confirm-dialog")!;
    fireEvent.mouseDown(dialog);

    expect(props.onCancel).not.toHaveBeenCalled();
  });

  it("danger が true の場合、確認ボタンに danger クラスが付与される", () => {
    const props = createProps({ danger: true });
    const { container } = render(ConfirmDialog, props);

    expect(
      container.querySelector(".confirm-dialog-danger"),
    ).toBeInTheDocument();
  });

  it("danger がデフォルトの場合、danger クラスが付与されない", () => {
    const props = createProps();
    const { container } = render(ConfirmDialog, props);

    expect(
      container.querySelector(".confirm-dialog-danger"),
    ).not.toBeInTheDocument();
  });
});
