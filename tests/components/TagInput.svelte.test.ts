import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  render,
  screen,
  cleanup,
  waitFor,
  fireEvent,
} from "@testing-library/svelte";
import { mockIPC } from "@tauri-apps/api/mocks";
import TagInput from "../../src/lib/components/TagInput.svelte";
import type { Tag } from "../../src/lib/types";

const MOCK_TAGS: Tag[] = [
  { id: 1, name: "風景", category: "genre" },
  { id: 2, name: "人物", category: "genre" },
  { id: 3, name: "田中太郎", category: "artist" },
  { id: 4, name: "サークルA", category: "circle" },
  { id: 5, name: "その他タグ", category: null },
];

function createProps(overrides = {}) {
  return {
    onSelectTag: vi.fn(),
    ...overrides,
  };
}

function getInput(): HTMLInputElement {
  return screen.getByPlaceholderText("タグを追加...") as HTMLInputElement;
}

async function typeAndSearch(value: string) {
  const input = getInput();
  fireEvent.input(input, { target: { value } });
  await vi.advanceTimersByTimeAsync(200);
  await waitFor(() => {
    expect(screen.queryByText(MOCK_TAGS[0].name)).toBeInTheDocument();
  });
}

beforeEach(() => {
  cleanup();
  vi.useFakeTimers();
  mockIPC((cmd: string) => {
    if (cmd === "search_tags") {
      return MOCK_TAGS;
    }
  });
});

afterEach(() => {
  vi.useRealTimers();
});

describe("TagInput コンポーネント", () => {
  it("プレースホルダーが正しく表示される", () => {
    render(TagInput, createProps());
    expect(getInput()).toBeInTheDocument();
  });

  it("カスタムプレースホルダーが表示される", () => {
    render(TagInput, createProps({ placeholder: "検索..." }));
    expect(screen.getByPlaceholderText("検索...")).toBeInTheDocument();
  });

  it("入力後200msのデバウンスで search_tags が呼ばれる", async () => {
    const searchSpy = vi.fn(() => MOCK_TAGS);
    mockIPC((cmd: string) => {
      if (cmd === "search_tags") return searchSpy();
    });
    render(TagInput, createProps());
    const input = getInput();

    fireEvent.input(input, { target: { value: "風" } });
    expect(searchSpy).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(100);
    expect(searchSpy).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(100);
    expect(searchSpy).toHaveBeenCalledOnce();
  });

  it("空入力では検索が実行されない", async () => {
    const searchSpy = vi.fn(() => MOCK_TAGS);
    mockIPC((cmd: string) => {
      if (cmd === "search_tags") return searchSpy();
    });
    render(TagInput, createProps());
    const input = getInput();

    fireEvent.input(input, { target: { value: "  " } });
    await vi.advanceTimersByTimeAsync(200);

    expect(searchSpy).not.toHaveBeenCalled();
  });

  it("検索結果がカテゴリごとにグループ表示される", async () => {
    const { container } = render(TagInput, createProps());
    await typeAndSearch("タグ");

    const headers = container.querySelectorAll(".tag-input-category-header");
    const headerTexts = Array.from(headers).map((h) => h.textContent);

    expect(headerTexts).toContain("artist");
    expect(headerTexts).toContain("circle");
    expect(headerTexts).toContain("genre");
    expect(headerTexts).toContain("その他");

    expect(headerTexts.indexOf("artist")).toBeLessThan(
      headerTexts.indexOf("circle"),
    );
    expect(headerTexts.indexOf("circle")).toBeLessThan(
      headerTexts.indexOf("genre"),
    );
  });

  it("excludeTagIds に含まれるタグは候補から除外される", async () => {
    render(TagInput, createProps({ excludeTagIds: [1, 3] }));
    const input = getInput();

    fireEvent.input(input, { target: { value: "タグ" } });
    await vi.advanceTimersByTimeAsync(200);

    await waitFor(() => {
      expect(screen.getByText("人物")).toBeInTheDocument();
    });
    expect(screen.queryByText("風景")).not.toBeInTheDocument();
    expect(screen.queryByText("田中太郎")).not.toBeInTheDocument();
  });

  it("候補をクリックすると onSelectTag が呼ばれ入力がクリアされる", async () => {
    const props = createProps();
    render(TagInput, props);
    await typeAndSearch("タグ");

    fireEvent.mouseDown(screen.getByText("風景"));

    expect(props.onSelectTag).toHaveBeenCalledWith(MOCK_TAGS[0]);
    await waitFor(() => {
      expect(getInput().value).toBe("");
    });
  });

  it("ArrowDown でハイライトが移動する", async () => {
    const { container } = render(TagInput, createProps());
    await typeAndSearch("タグ");

    const input = getInput();
    fireEvent.keyDown(input, { key: "ArrowDown" });

    await waitFor(() => {
      const highlighted = container.querySelector(
        ".tag-input-option-highlight",
      );
      expect(highlighted).toBeInTheDocument();
    });
  });

  it("ArrowUp/Down でハイライトが循環する", async () => {
    const { container } = render(TagInput, createProps());
    await typeAndSearch("タグ");

    const input = getInput();

    fireEvent.keyDown(input, { key: "ArrowUp" });
    await waitFor(() => {
      const options = container.querySelectorAll(".tag-input-option");
      const lastOption = options[options.length - 1];
      expect(lastOption).toHaveClass("tag-input-option-highlight");
    });
  });

  it("Enter でハイライト中のタグが選択される", async () => {
    const props = createProps();
    render(TagInput, props);
    await typeAndSearch("タグ");

    const input = getInput();
    fireEvent.keyDown(input, { key: "ArrowDown" });
    fireEvent.keyDown(input, { key: "Enter" });

    expect(props.onSelectTag).toHaveBeenCalledOnce();
  });

  it("Escape でドロップダウンが閉じる", async () => {
    render(TagInput, createProps());
    await typeAndSearch("タグ");

    expect(screen.getByText("風景")).toBeInTheDocument();

    const input = getInput();
    fireEvent.keyDown(input, { key: "Escape" });

    await waitFor(() => {
      expect(screen.queryByText("風景")).not.toBeInTheDocument();
    });
  });

  it("完全一致がない場合「作成」オプションが表示される", async () => {
    vi.useRealTimers();
    const props = createProps({
      onCreateTag: vi
        .fn()
        .mockResolvedValue({ id: 99, name: "新タグ", category: null }),
    });
    render(TagInput, props);

    const input = getInput();
    fireEvent.input(input, { target: { value: "新タグ" } });

    await waitFor(() => {
      expect(screen.getByText(/を作成/)).toBeInTheDocument();
    });
  });

  it("作成オプションを選択すると onCreateTag が呼ばれる", async () => {
    vi.useRealTimers();
    const onCreateTag = vi
      .fn()
      .mockResolvedValue({ id: 99, name: "新タグ", category: null });
    const props = createProps({ onCreateTag });
    render(TagInput, props);

    const input = getInput();
    fireEvent.input(input, { target: { value: "新タグ" } });

    await waitFor(() => {
      expect(screen.getByText(/を作成/)).toBeInTheDocument();
    });

    fireEvent.mouseDown(screen.getByText(/を作成/));

    expect(onCreateTag).toHaveBeenCalledWith("新タグ");
  });

  it("onCreateTag が未設定の場合は作成オプションが表示されない", async () => {
    render(TagInput, createProps());

    const input = getInput();
    fireEvent.input(input, { target: { value: "新タグ" } });
    await vi.advanceTimersByTimeAsync(200);

    await waitFor(() => {
      expect(screen.getByText("風景")).toBeInTheDocument();
    });
    expect(screen.queryByText(/を作成/)).not.toBeInTheDocument();
  });

  it("完全一致がある場合は作成オプションが表示されない", async () => {
    const props = createProps({
      onCreateTag: vi
        .fn()
        .mockResolvedValue({ id: 99, name: "風景", category: null }),
    });
    render(TagInput, props);

    const input = getInput();
    fireEvent.input(input, { target: { value: "風景" } });
    await vi.advanceTimersByTimeAsync(200);

    await waitFor(() => {
      expect(screen.getByText("風景")).toBeInTheDocument();
    });
    expect(screen.queryByText(/を作成/)).not.toBeInTheDocument();
  });
});
