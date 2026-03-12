import { invoke } from "@tauri-apps/api/core";
import type { BannerAutoClose } from "../types";

let seconds = $state<BannerAutoClose>(3);

export function getBannerAutoClose(): BannerAutoClose {
  return seconds;
}

export async function initBannerAutoClose() {
  try {
    seconds = (await invoke<number>("get_banner_auto_close")) as BannerAutoClose;
  } catch {
    seconds = 3;
  }
}

export async function setBannerAutoClose(value: BannerAutoClose) {
  await invoke("set_banner_auto_close", { seconds: value });
  seconds = value;
}
