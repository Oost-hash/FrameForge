// [console-login feature] — delete this file to remove the feature
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { TAURI_EVENTS } from "./constants/tauri";
import type { ConsoleLoginSuccessPayload } from "./types/tauri";

interface Props {
  onLogin: (accountId: string, nonce: string) => void;
}

export default function ConsoleLogin({ onLogin }: Props) {
  const [status, setStatus] = useState<"idle" | "waiting" | "done" | "error">("idle");
  const [msg, setMsg]       = useState<string | null>(null);

  useEffect(() => {
    const ul = listen<ConsoleLoginSuccessPayload>(TAURI_EVENTS.CONSOLE_LOGIN_SUCCESS, e => {
      setStatus("done");
      setMsg("Session captured — inventory access active.");
      onLogin(e.payload.accountId, e.payload.nonce);
    });
    return () => { ul.then(fn => fn()); };
  }, [onLogin]);

  const open = async () => {
    setStatus("waiting");
    setMsg(null);
    try {
      await invoke("open_console_login");
    } catch (e) {
      setStatus("error");
      setMsg(String(e));
    }
  };

  const cancel = () => {
    setStatus("idle");
    setMsg(null);
  };

  return (
    <div className="settings-section mt-3 border-t border-white/6 pt-3">
      <div className="settings-section-title flex items-center gap-2">
        Console / Web Login
        <span className="rounded-[4px] border border-[#ffb400]/30 bg-[#ffb400]/15 px-[5px] py-px text-[9px] font-bold tracking-[.06em] text-[#ffb400]">EXPERIMENTAL</span>
      </div>

      <div className="mb-2.5 text-[11px] leading-[1.6] text-muted">
        Opens warframe.com in a secure browser window. Log in with any method —
        PlayStation, Xbox, Nintendo, or email/password. FrameForge intercepts
        the session automatically and closes the window.
      </div>

      {status !== "waiting" && (
        <button
          className="btn-secondary"
          onClick={open}
          style={status === "done" ? { opacity: 0.5 } : undefined}
        >
          {status === "done" ? "Re-open Login" : "Open Warframe Login"}
        </button>
      )}

      {status === "waiting" && (
        <div className="flex items-center gap-2.5">
          <span className="text-[12px] text-muted">Waiting for login…</span>
          <button className="btn-secondary px-2.5 py-0.5 text-[11px]" onClick={cancel}>
            Cancel
          </button>
        </div>
      )}

      {msg && (
        <div className="settings-msg mt-1.5" style={{ color: status === "error" ? "var(--red)" : "var(--green)" }}>
          {msg}
        </div>
      )}
    </div>
  );
}
