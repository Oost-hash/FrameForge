import { emit, listen } from "@tauri-apps/api/event";
import { PREFERENCE_KEYS } from "../constants/preferences";
import { TAURI_EVENTS } from "../constants/tauri";
import preflightUrl from "../styles/preflight.css?url";

const LINK_ID = "ff-preflight";

export function preflightEnabled(): boolean {
  return localStorage.getItem(PREFERENCE_KEYS.PREFLIGHT) === "on";
}

function applyPreflight(on: boolean): void {
  const existing = document.getElementById(LINK_ID) as HTMLLinkElement | null;
  if (!on) {
    existing?.remove();
    return;
  }
  if (existing) {
    existing.disabled = false;
    return;
  }
  const link = document.createElement("link");
  link.id = LINK_ID;
  link.rel = "stylesheet";
  link.href = preflightUrl;
  document.head.appendChild(link);
}

export async function setPreflight(on: boolean): Promise<void> {
  localStorage.setItem(PREFERENCE_KEYS.PREFLIGHT, on ? "on" : "off");
  applyPreflight(on);
  await emit(TAURI_EVENTS.PREFLIGHT_TOGGLED, on).catch(() => {});
}

export function initPreflight(): void {
  applyPreflight(preflightEnabled());
  listen<boolean>(TAURI_EVENTS.PREFLIGHT_TOGGLED, e => {
    localStorage.setItem(PREFERENCE_KEYS.PREFLIGHT, e.payload ? "on" : "off");
    applyPreflight(e.payload);
  }).catch(() => {});
}
