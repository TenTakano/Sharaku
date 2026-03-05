<script lang="ts">
  import {
    getThemeMode,
    setThemeMode as setThemeModeStore,
  } from "../stores/theme.svelte";
  import { addToast } from "../stores/toast.svelte";
  import type { ThemeMode } from "../types";

  async function handleThemeChange(mode: ThemeMode) {
    try {
      await setThemeModeStore(mode);
    } catch (e) {
      addToast("error", `テーマの変更に失敗しました: ${e}`);
    }
  }
</script>

<div class="settings-scroll">
  <div class="settings-content">
    <section class="settings-section">
      <h2>テーマ</h2>
      <p class="settings-description">アプリの外観テーマを選択します。</p>
      <div class="resource-mode-select">
        <label class="resource-mode-option">
          <input
            type="radio"
            name="theme-mode"
            checked={getThemeMode() === "system"}
            onchange={() => handleThemeChange("system")}
          />
          <span class="resource-mode-label">OS設定に従う</span>
          <span class="resource-mode-desc"
            >OSのダーク/ライト設定に自動追従</span
          >
        </label>
        <label class="resource-mode-option">
          <input
            type="radio"
            name="theme-mode"
            checked={getThemeMode() === "light"}
            onchange={() => handleThemeChange("light")}
          />
          <span class="resource-mode-label">ライト</span>
          <span class="resource-mode-desc">常にライトテーマを使用</span>
        </label>
        <label class="resource-mode-option">
          <input
            type="radio"
            name="theme-mode"
            checked={getThemeMode() === "dark"}
            onchange={() => handleThemeChange("dark")}
          />
          <span class="resource-mode-label">ダーク</span>
          <span class="resource-mode-desc">常にダークテーマを使用</span>
        </label>
      </div>
    </section>
  </div>
</div>
