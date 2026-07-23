import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, cleanup } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import ImportBanner from "../../src/lib/components/ImportBanner.svelte";
import type { ImportJobState } from "../../src/lib/stores/importQueue.svelte";

const mockGetJobs = vi.fn<() => ImportJobState[]>(() => []);
const mockDismissJob = vi.fn();

vi.mock("../../src/lib/stores/importQueue.svelte", () => ({
  getJobs: (...args: unknown[]) => mockGetJobs(...(args as [])),
  dismissJob: (...args: unknown[]) => mockDismissJob(...(args as [string])),
}));

beforeEach(() => {
  cleanup();
  mockGetJobs.mockReset();
  mockDismissJob.mockReset();
  mockGetJobs.mockReturnValue([]);
});

function createJob(overrides: Partial<ImportJobState> = {}): ImportJobState {
  return {
    jobId: "job-1",
    status: "running",
    total: 3,
    current: 1,
    currentTitle: "テスト画像",
    succeeded: 0,
    failed: 0,
    errors: [],
    ...overrides,
  };
}

describe("ImportBanner コンポーネント", () => {
  it("queued 状態のジョブは表示されない", () => {
    mockGetJobs.mockReturnValue([createJob({ status: "queued" })]);
    const { container } = render(ImportBanner);
    expect(container.querySelector(".import-banner")).not.toBeInTheDocument();
  });

  it("running 状態で「取り込み中」とタイトルが表示される", () => {
    mockGetJobs.mockReturnValue([createJob()]);
    render(ImportBanner);
    expect(screen.getByText(/取り込み中/)).toBeInTheDocument();
    expect(screen.getByText(/テスト画像/)).toBeInTheDocument();
  });

  it("running 状態で total > 1 の場合にプログレスバーが表示される", () => {
    mockGetJobs.mockReturnValue([createJob({ total: 5, current: 2 })]);
    const { container } = render(ImportBanner);
    const progress = container.querySelector("progress");
    expect(progress).toBeInTheDocument();
    expect(progress).toHaveAttribute("max", "5");
    expect(progress).toHaveAttribute("value", "2");
  });

  it("running 状態で total = 1 の場合にプログレスバーが表示されない", () => {
    mockGetJobs.mockReturnValue([createJob({ total: 1 })]);
    const { container } = render(ImportBanner);
    expect(container.querySelector("progress")).not.toBeInTheDocument();
  });

  it("completed 状態で成功メッセージが表示される", () => {
    mockGetJobs.mockReturnValue([
      createJob({ status: "completed", succeeded: 3, failed: 0 }),
    ]);
    render(ImportBanner);
    expect(screen.getByText(/3件成功/)).toBeInTheDocument();
  });

  it("completed 状態で失敗がある場合にエラー詳細が表示される", () => {
    mockGetJobs.mockReturnValue([
      createJob({
        status: "completed",
        succeeded: 2,
        failed: 1,
        errors: [{ title: "画像A", message: "ファイルが見つかりません" }],
      }),
    ]);
    render(ImportBanner);
    expect(screen.getByText(/2件成功/)).toBeInTheDocument();
    expect(screen.getByText(/1件失敗/)).toBeInTheDocument();
    expect(screen.getByText("画像A")).toBeInTheDocument();
  });

  it("failed 状態で「取り込みに失敗しました」が表示される", () => {
    mockGetJobs.mockReturnValue([createJob({ status: "failed" })]);
    render(ImportBanner);
    expect(screen.getByText("取り込みに失敗しました")).toBeInTheDocument();
  });

  it("閉じるボタンで dismissJob が呼ばれる", async () => {
    const user = userEvent.setup();
    mockGetJobs.mockReturnValue([
      createJob({ status: "completed", succeeded: 1 }),
    ]);
    render(ImportBanner);

    const closeBtn = screen.getByText("×");
    await user.click(closeBtn);

    expect(mockDismissJob).toHaveBeenCalledWith("job-1");
  });
});
