import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import { LogicalPosition, LogicalSize, availableMonitors } from "@tauri-apps/api/window";
import { TAURI_COMMANDS, TAURI_EVENTS } from "../constants/tauri";
import { ensureRivenWindow, rivenPlacement } from "./rivenWindow";
import { overlayScale } from "./uiScale";

type Rect = [number, number, number, number];

async function rectOrScreen(): Promise<Rect> {
  try {
    return await invoke<Rect>("get_warframe_window_rect");
  } catch {
    try {
      const m = (await availableMonitors())[0];
      if (m) return [m.position.x, m.position.y, m.size.width, m.size.height];
    } catch {}
    return [0, 0, 1920, 1080];
  }
}

export async function showRewardOverlay(): Promise<void> {
  const [wx, wy, ww, wh] = await rectOrScreen();
  const offsetY = Math.round(wh * 0.60);
  const stripH = Math.min(Math.round(wh * 0.30 * overlayScale()), wh - offsetY);
  try {
    await emit(TAURI_EVENTS.RELIC_REWARD_PREVIEW, {});
    await invoke("show_overlay_window", { x: wx, y: wy + offsetY, w: ww, h: stripH });
  } catch {}
}

export async function hideRewardOverlay(): Promise<void> {
  await emit(TAURI_EVENTS.RELIC_REWARD_PREVIEW_CLOSE, {});
}

export async function showPickOverlay(): Promise<void> {
  await invoke(TAURI_COMMANDS.TEST_RELIC_PICK, { era: "ALL" }).catch(() => {});
}

export async function placePickOverlay(): Promise<void> {
  await showPickOverlay();
}

export async function hidePickOverlay(): Promise<void> {
  await emit(TAURI_EVENTS.RELIC_PICK_PREVIEW_CLOSE, {});
}

async function ensurePlacedRiven(): Promise<{ fresh: boolean } | null> {
  const [wx, wy, , wh] = await rectOrScreen();
  const result = await ensureRivenWindow(wx, wy, wh);
  if (!result) return null;
  if (!result.fresh) {
    try {
      const p = await rivenPlacement(wx, wy, wh);
      await result.win.setPosition(new LogicalPosition(p.x, p.y));
      await result.win.setSize(new LogicalSize(p.width, p.height));
    } catch {}
  }
  return { fresh: result.fresh };
}

export async function showRivenOverlay(): Promise<void> {
  await ensurePlacedRiven();
}

export async function hideRivenOverlay(): Promise<void> {
  await emit(TAURI_EVENTS.RIVEN_PREVIEW_CLOSE, {});
}

export async function showRivenDummy(): Promise<void> {
  const placed = await ensurePlacedRiven();
  if (!placed) return;
  if (placed.fresh) {
    await new Promise<void>(resolve => {
      let unlisten: (() => void) | undefined;
      const finish = () => { clearTimeout(timer); unlisten?.(); resolve(); };
      const timer = setTimeout(finish, 3000);
      listen(TAURI_EVENTS.RIVEN_WINDOW_READY, finish).then(fn => { unlisten = fn; }).catch(() => resolve());
    });
  }

  await emit(TAURI_EVENTS.RIVEN_PREVIEW_START, {});
  await emit(TAURI_EVENTS.RIVEN_ANALYSIS_UPDATE, {
    dummy: true,
    analysis: {
      weapon: "Tenora Prime",
      matched_positives: ["Critical Chance", "Damage to Grineer"],
      missing_positives: ["Multi-shot"],
      safe_negatives_present: ["Zoom"],
      harmful_negatives: ["Damage to Corpus"],
      total_wanted: 4,
      score: 78,
      verdict: "Strong roll",
      notes: "Dummy payload for preflight comparison",
      alternatives: [],
    },
    rollCount: 4,
    ocrRaw: "TENORA PRIME\n+180% Critical Chance",
    weapon: "Tenora Prime",
    positives: ["+180% Critical Chance", "+145% Damage to Grineer"],
    negatives: ["-20% Damage to Corpus"],
    rolledStats: [
      { name: "Critical Chance", value: "+180%", positive: true },
      { name: "Damage to Grineer", value: "+145%", positive: true },
      { name: "Multi-shot", value: "+85%", positive: true },
      { name: "Damage to Corpus", value: "-20%", positive: false },
    ],
    originalStats: [
      { name: "Critical Chance", value: "+153%", positive: true },
      { name: "Damage to Grineer", value: "+121%", positive: true },
      { name: "Multi-shot", value: "+77%", positive: true },
      { name: "Zoom", value: "-10%", positive: false },
    ],
    isComparison: false,
  });
}
