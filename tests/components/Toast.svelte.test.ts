import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, cleanup } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import Toast from "../../src/lib/components/Toast.svelte";
import {
  addToast,
  removeToast,
  getToasts,
} from "../../src/lib/stores/toast.svelte";

beforeEach(() => {
  for (const toast of getToasts()) {
    removeToast(toast.id);
  }
  cleanup();
});

describe("Toast コンポーネント", () => {
  it("トーストがない場合は何も表示されない", () => {
    const { container } = render(Toast);
    expect(container.querySelector(".toast")).not.toBeInTheDocument();
  });

  it("success トーストが正しく表示される", () => {
    addToast("success", "保存しました");
    const { container } = render(Toast);
    expect(screen.getByText("保存しました")).toBeInTheDocument();
    expect(container.querySelector(".toast-success")).toBeInTheDocument();
  });

  it("error トーストが正しく表示される", () => {
    addToast("error", "エラーが発生しました");
    const { container } = render(Toast);
    expect(screen.getByText("エラーが発生しました")).toBeInTheDocument();
    expect(container.querySelector(".toast-error")).toBeInTheDocument();
  });

  it("複数のトーストが同時に表示される", () => {
    addToast("success", "成功メッセージ");
    addToast("error", "エラーメッセージ");
    render(Toast);
    expect(screen.getByText("成功メッセージ")).toBeInTheDocument();
    expect(screen.getByText("エラーメッセージ")).toBeInTheDocument();
  });

  it("各トーストにメッセージと閉じるボタンが含まれる", () => {
    addToast("success", "テスト");
    const { container } = render(Toast);
    expect(container.querySelector(".toast-message")).toBeInTheDocument();
    expect(container.querySelector(".toast-close")).toBeInTheDocument();
  });

  it("×ボタンをクリックするとトーストが削除される", async () => {
    const user = userEvent.setup();
    addToast("success", "削除テスト");
    const { container } = render(Toast);

    const closeButton = container.querySelector(".toast-close")!;
    await user.click(closeButton);

    expect(screen.queryByText("削除テスト")).not.toBeInTheDocument();
  });
});
