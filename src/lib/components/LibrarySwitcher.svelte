<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { Library } from "../types";

  interface Props {
    activeLibrary: Library | null;
    onSwitch: (library: Library) => void;
  }

  let { activeLibrary, onSwitch }: Props = $props();

  let libraries = $state<Library[]>([]);
  let isOpen = $state(false);
  let adding = $state(false);

  async function loadLibraries() {
    try {
      libraries = await invoke<Library[]>("list_libraries");
    } catch {
      libraries = [];
    }
  }

  function toggleDropdown() {
    if (!isOpen) {
      loadLibraries();
    }
    isOpen = !isOpen;
  }

  function closeDropdown() {
    isOpen = false;
  }

  async function selectLibrary(lib: Library) {
    if (lib.id === activeLibrary?.id) {
      closeDropdown();
      return;
    }
    try {
      await invoke("switch_library", { id: lib.id });
      onSwitch(lib);
    } catch (e) {
      console.error("ライブラリ切り替え失敗:", e);
    }
    closeDropdown();
  }

  async function addLibrary() {
    const selected = await open({ directory: true });
    if (!selected) return;

    adding = true;
    try {
      const dirName = selected.split("/").pop() || selected;
      const lib = await invoke<Library>("create_library", {
        name: dirName,
        path: selected,
      });
      onSwitch(lib);
    } catch (e) {
      console.error("ライブラリ追加失敗:", e);
    } finally {
      adding = false;
      closeDropdown();
    }
  }
</script>

<div class="library-switcher">
  <button class="library-switcher-btn" onclick={toggleDropdown}>
    {activeLibrary?.name ?? "ライブラリ未選択"}
    <span class="library-switcher-arrow">{isOpen ? "\u25B2" : "\u25BC"}</span>
  </button>

  {#if isOpen}
    <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
    <div class="library-switcher-overlay" onclick={closeDropdown}></div>
    <div class="library-switcher-dropdown">
      {#each libraries as lib (lib.id)}
        <button
          class="library-switcher-item"
          class:active={lib.id === activeLibrary?.id}
          onclick={() => selectLibrary(lib)}
        >
          <span class="library-item-name">{lib.name}</span>
          <span class="library-item-path">{lib.path}</span>
        </button>
      {/each}
      <button
        class="library-switcher-add"
        onclick={addLibrary}
        disabled={adding}
      >
        + ライブラリを追加
      </button>
    </div>
  {/if}
</div>
