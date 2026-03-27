import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import ContextMenu from "../../src/lib/components/ContextMenu.svelte";

function createProps(overrides = {}) {
  return {
    x: 100,
    y: 200,
    items: [
      { label: "編集", action: vi.fn() },
      { label: "削除", action: vi.fn() },
    ],
    onClose: vi.fn(),
    ...overrides,
  };
}

beforeEach(() => {
  cleanup();
});

describe("ContextMenu コンポーネント", () => {
  it("メニュー項目が正しく表示される", () => {
    const props = createProps();
    render(ContextMenu, props);

    expect(screen.getByText("編集")).toBeInTheDocument();
    expect(screen.getByText("削除")).toBeInTheDocument();
  });

  it("指定位置に表示される", () => {
    const props = createProps({ x: 150, y: 250 });
    const { container } = render(ContextMenu, props);

    const menu = container.querySelector(".context-menu")!;
    expect(menu).toHaveStyle({ left: "150px", top: "250px" });
  });

  it("メニュー項目をクリックすると対応する action が実行される", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(ContextMenu, props);

    await user.click(screen.getByText("編集"));

    expect(props.items[0].action).toHaveBeenCalledOnce();
    expect(props.items[1].action).not.toHaveBeenCalled();
  });

  it("メニュー項目クリック後に onClose が呼ばれない", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(ContextMenu, props);

    await user.click(screen.getByText("編集"));

    expect(props.onClose).not.toHaveBeenCalled();
  });

  it("メニュー外をクリックすると onClose が呼ばれる", async () => {
    const user = userEvent.setup();
    const props = createProps();
    render(ContextMenu, props);

    await user.click(document.body);

    expect(props.onClose).toHaveBeenCalledOnce();
  });

  it("右クリックのデフォルト動作が抑制される", () => {
    const props = createProps();
    const { container } = render(ContextMenu, props);

    const menu = container.querySelector(".context-menu")!;
    const event = new MouseEvent("contextmenu", {
      bubbles: true,
      cancelable: true,
    });
    menu.dispatchEvent(event);

    expect(event.defaultPrevented).toBe(true);
  });
});
