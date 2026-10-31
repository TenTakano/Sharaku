import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { mockIPC } from "@tauri-apps/api/mocks";
import HomeView from "../../src/lib/components/HomeView.svelte";
import { clearThumbnailCache } from "../../src/lib/thumbnailCache";
import type { WorkWithHistory } from "../../src/lib/types";

function work(
  id: number,
  overrides: Partial<WorkWithHistory> = {},
): WorkWithHistory {
  return {
    id,
    title: `作品${id}`,
    workType: "folder",
    pageCount: 10,
    createdAt: "2026-01-01T00:00:00.000Z",
    lastViewedAt: null,
    viewCount: 0,
    lastPage: null,
    totalViewSeconds: 0,
    ...overrides,
  };
}

type ShelfData = Partial<Record<string, WorkWithHistory[]>>;

let calls: { cmd: string; args: Record<string, unknown> }[];

function setup(data: ShelfData) {
  calls = [];
  mockIPC((cmd: string, args?: unknown) => {
    const a = (args ?? {}) as Record<string, unknown>;
    calls.push({ cmd, args: a });
    if (cmd === "get_thumbnail") throw new Error("no thumbnail");
    if (cmd === "list_recently_viewed_works" && a.limit === 1) {
      return (data[cmd] ?? []).slice(0, 1);
    }
    return data[cmd] ?? [];
  });
}

function renderHome() {
  const props = {
    reloadTrigger: 0,
    onOpenWork: vi.fn(),
    onBrowseAll: vi.fn(),
  };
  render(HomeView, props);
  return props;
}

beforeEach(() => {
  cleanup();
  clearThumbnailCache();
});

describe("HomeView", () => {
  it("作品のある棚だけを定義順に表示する", async () => {
    setup({
      list_recently_added_works: [work(1), work(2)],
      list_unviewed_works: [work(2)],
      list_random_works: [work(1)],
    });
    renderHome();

    await waitFor(() => {
      expect(screen.getByText("最近追加した")).toBeInTheDocument();
    });
    const titles = screen
      .getAllByRole("heading", { level: 2 })
      .map((h) => h.textContent);
    expect(titles).toEqual(["最近追加した", "まだ見ていない", "ランダム"]);
    expect(screen.queryByText("最近見た")).not.toBeInTheDocument();
    expect(screen.getAllByText("未読").length).toBeGreaterThan(0);
  });

  it("作品を選ぶと、その棚の並びと一緒に onOpenWork を呼ぶ", async () => {
    const user = userEvent.setup();
    setup({ list_recently_added_works: [work(1), work(2), work(3)] });
    const props = renderHome();

    await waitFor(() => {
      expect(screen.getByText("作品2")).toBeInTheDocument();
    });
    await user.click(screen.getByText("作品2"));

    expect(props.onOpenWork).toHaveBeenCalledWith([1, 2, 3], 2);
  });

  it("すべて再生は棚の先頭から、すべて見るはブラウズへ", async () => {
    const user = userEvent.setup();
    setup({ list_recently_added_works: [work(5), work(6)] });
    const props = renderHome();

    await waitFor(() => {
      expect(screen.getByText("すべて再生")).toBeInTheDocument();
    });
    await user.click(screen.getByText("すべて再生"));
    expect(props.onOpenWork).toHaveBeenCalledWith([5, 6], 5);

    await user.click(screen.getByText("すべて見る"));
    expect(props.onBrowseAll).toHaveBeenCalled();
  });

  it("続きからは最後に見た作品を最後のページから再開できる", async () => {
    const user = userEvent.setup();
    setup({
      list_recently_viewed_works: [
        work(7, {
          lastViewedAt: new Date().toISOString(),
          viewCount: 1,
          lastPage: 4,
        }),
      ],
    });
    const props = renderHome();

    await waitFor(() => {
      expect(screen.getByText("続きから")).toBeInTheDocument();
    });
    expect(screen.getByText(/5 \/ 10 枚目で閉じました/)).toBeInTheDocument();

    await user.click(screen.getByText("5 枚目から再開"));
    expect(props.onOpenWork).toHaveBeenCalledWith([7], 7, 4);

    await user.click(screen.getByText("最初から"));
    expect(props.onOpenWork).toHaveBeenCalledWith([7], 7, 0);
  });

  it("引き直すでランダムの棚を取り直す", async () => {
    const user = userEvent.setup();
    setup({ list_random_works: [work(1)] });
    renderHome();

    await waitFor(() => {
      expect(screen.getByText("引き直す")).toBeInTheDocument();
    });
    const before = calls.filter((c) => c.cmd === "list_random_works").length;
    await user.click(screen.getByText("引き直す"));

    await waitFor(() => {
      expect(calls.filter((c) => c.cmd === "list_random_works").length).toBe(
        before + 1,
      );
    });
  });

  it("おまかせで見るはランダムに選んだ作品を開く", async () => {
    const user = userEvent.setup();
    setup({ list_random_works: [work(3), work(1)] });
    const props = renderHome();

    await waitFor(() => {
      expect(screen.getByText("ランダム")).toBeInTheDocument();
    });
    await user.click(screen.getByText("おまかせで見る"));

    await waitFor(() => {
      expect(props.onOpenWork).toHaveBeenCalledWith([3, 1], 3);
    });
  });

  it("作品が一つもなければ案内を表示する", async () => {
    setup({});
    renderHome();

    await waitFor(() => {
      expect(screen.getByText(/作品がありません/)).toBeInTheDocument();
    });
    expect(screen.getByText("おまかせで見る")).toBeDisabled();
  });
});
