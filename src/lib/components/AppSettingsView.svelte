<script lang="ts">
  import {
    getBannerAutoClose,
    setBannerAutoClose as setBannerAutoCloseStore,
  } from "../stores/bannerAutoClose.svelte";
  import {
    getThemeMode,
    setThemeMode as setThemeModeStore,
  } from "../stores/theme.svelte";
  import { addToast } from "../stores/toast.svelte";
  import type { BannerAutoClose, ThemeMode } from "../types";

  async function handleThemeChange(mode: ThemeMode) {
    try {
      await setThemeModeStore(mode);
    } catch (e) {
      addToast("error", `テーマの変更に失敗しました: ${e}`);
    }
  }

  const bannerOptions: { value: BannerAutoClose; label: string }[] = [
    { value: 1, label: "1秒" },
    { value: 3, label: "3秒" },
    { value: 5, label: "5秒" },
    { value: 0, label: "手動で閉じる" },
  ];

  async function handleBannerAutoCloseChange(seconds: BannerAutoClose) {
    try {
      await setBannerAutoCloseStore(seconds);
    } catch (e) {
      addToast("error", `設定の変更に失敗しました: ${e}`);
    }
  }
</script>

<div class="settings-scroll">
  <div class="settings-content">
    <section class="settings-section">
      <h2>テーマ</h2>
      <p class="settings-description">アプリの外観テーマを選択します。</p>
      <div class="radio-option-group">
        <label class="radio-option">
          <input
            type="radio"
            name="theme-mode"
            checked={getThemeMode() === "system"}
            onchange={() => handleThemeChange("system")}
          />
          <span class="radio-option-label">OS設定に従う</span>
          <span class="radio-option-desc">OSのダーク/ライト設定に自動追従</span>
        </label>
        <label class="radio-option">
          <input
            type="radio"
            name="theme-mode"
            checked={getThemeMode() === "light"}
            onchange={() => handleThemeChange("light")}
          />
          <span class="radio-option-label">ライト</span>
          <span class="radio-option-desc">常にライトテーマを使用</span>
        </label>
        <label class="radio-option">
          <input
            type="radio"
            name="theme-mode"
            checked={getThemeMode() === "dark"}
            onchange={() => handleThemeChange("dark")}
          />
          <span class="radio-option-label">ダーク</span>
          <span class="radio-option-desc">常にダークテーマを使用</span>
        </label>
      </div>
    </section>

    <section class="settings-section">
      <h2>取り込みバナー</h2>
      <p class="settings-description">
        取り込み完了バナーが自動で消えるまでの時間を設定します。エラーがある場合は手動で閉じる必要があります。
      </p>
      <div class="radio-option-group">
        {#each bannerOptions as option (option.value)}
          <label class="radio-option">
            <input
              type="radio"
              name="banner-auto-close"
              checked={getBannerAutoClose() === option.value}
              onchange={() => handleBannerAutoCloseChange(option.value)}
            />
            <span class="radio-option-label">{option.label}</span>
          </label>
        {/each}
      </div>
    </section>
  </div>
</div>
