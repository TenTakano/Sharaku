<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import Shelf from "./Shelf.svelte";
  import { SHELF_LIMIT, SHELVES, type ShelfDefinition } from "../shelves";
  import { loadThumbnail } from "../thumbnailCache";
  import { createLatestRequestGuard } from "../utils/latestRequest";
  import { formatAgo } from "../utils/relativeTime";
  import type { WorkWithHistory } from "../types";

  interface Props {
    reloadTrigger: number;
    /** Opens `workId` in the viewer; `workIds` is the sequence ←/→ walks through. */
    onOpenWork: (workIds: number[], workId: number, page?: number) => void;
    onBrowseAll: () => void;
  }

  let { reloadTrigger, onOpenWork, onBrowseAll }: Props = $props();

  let shelfWorks = $state<Record<string, WorkWithHistory[]>>({});
  let resume = $state<WorkWithHistory | null>(null);
  let resumeThumbnail = $state<string | null>(null);
  let loaded = $state(false);
  let now = $state(new Date());

  let visibleShelves = $derived(
    SHELVES.filter((s) => (shelfWorks[s.id]?.length ?? 0) > 0),
  );
  let isEmpty = $derived(loaded && !resume && visibleShelves.length === 0);

  async function fetchShelf(shelf: ShelfDefinition) {
    try {
      return (
        (await invoke<WorkWithHistory[] | null>(shelf.command, {
          limit: SHELF_LIMIT,
        })) ?? []
      );
    } catch (e) {
      console.error(`Failed to load shelf ${shelf.id}:`, e);
      return [];
    }
  }

  async function fetchResume(): Promise<WorkWithHistory | null> {
    try {
      const [last] =
        (await invoke<WorkWithHistory[] | null>("list_recently_viewed_works", {
          limit: 1,
        })) ?? [];
      return last ?? null;
    } catch (e) {
      console.error("Failed to load resume:", e);
      return null;
    }
  }

  // A reload (library switch) can start before the previous one finishes;
  // only the latest one may write its results.
  const loadGuard = createLatestRequestGuard();

  async function loadAll() {
    const requestId = loadGuard.next();
    const startedAt = new Date();
    const [results, last] = await Promise.all([
      Promise.all(SHELVES.map(fetchShelf)),
      fetchResume(),
    ]);
    const thumbnail = last ? await loadThumbnail(last.id) : null;
    if (!loadGuard.isLatest(requestId)) return;
    now = startedAt;
    shelfWorks = Object.fromEntries(SHELVES.map((s, i) => [s.id, results[i]]));
    resume = last;
    resumeThumbnail = thumbnail;
    loaded = true;
  }

  async function reshuffle(shelf: ShelfDefinition) {
    shelfWorks = { ...shelfWorks, [shelf.id]: await fetchShelf(shelf) };
  }

  async function watchSomething() {
    const random = SHELVES.find((s) => s.canShuffle);
    const works = random ? await fetchShelf(random) : [];
    if (works.length === 0) return;
    const ids = works.map((w) => w.id);
    onOpenWork(ids, ids[0]);
  }

  function openResume(page: number) {
    if (!resume) return;
    onOpenWork([resume.id], resume.id, page);
  }

  $effect(() => {
    void reloadTrigger;
    loadAll();
  });
</script>

<div class="home">
  <div class="home-top">
    <button
      class="home-surprise"
      onclick={watchSomething}
      disabled={loaded && isEmpty}
    >
      おまかせで見る
    </button>
  </div>

  {#if resume}
    {@const page = resume.lastPage ?? 0}
    <section class="home-resume" aria-labelledby="home-resume-title">
      <div class="home-resume-cover">
        {#if resumeThumbnail}
          <img src={resumeThumbnail} alt="" />
        {/if}
      </div>
      <div class="home-resume-body">
        <span class="home-resume-label">続きから</span>
        <h2 id="home-resume-title" class="home-resume-title">
          {resume.title}
        </h2>
        <span class="home-resume-meta">
          {resume.lastViewedAt
            ? `${formatAgo(resume.lastViewedAt, now)} · `
            : ""}{page + 1} / {resume.pageCount} 枚目で閉じました
        </span>
        <div class="home-resume-progress" aria-hidden="true">
          <div
            class="home-resume-progress-bar"
            style="width: {Math.min(
              100,
              ((page + 1) / Math.max(1, resume.pageCount)) * 100,
            )}%"
          ></div>
        </div>
        <div class="home-resume-actions">
          {#if page > 0}
            <button
              class="home-resume-primary"
              onclick={() => openResume(page)}
            >
              {page + 1} 枚目から再開
            </button>
          {/if}
          <button
            class={page > 0 ? "home-resume-secondary" : "home-resume-primary"}
            onclick={() => openResume(0)}
          >
            最初から
          </button>
        </div>
      </div>
    </section>
  {/if}

  {#each visibleShelves as shelf (shelf.id)}
    <Shelf
      {shelf}
      works={shelfWorks[shelf.id]}
      {now}
      {onOpenWork}
      {onBrowseAll}
      onShuffle={() => reshuffle(shelf)}
    />
  {/each}

  {#if isEmpty}
    <div class="home-empty">
      <p>
        作品がありません。フォルダをドロップするか、取り込みから作品を追加してください。
      </p>
    </div>
  {/if}
</div>
