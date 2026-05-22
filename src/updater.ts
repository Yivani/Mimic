import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

export interface AvailableUpdate {
  version: string;
  currentVersion: string;
  /** Download + install, reporting 0..1 progress, then relaunch. */
  install: (onProgress: (fraction: number) => void) => Promise<void>;
}

/** Checks the configured endpoint for a newer signed release. */
export async function checkForUpdate(): Promise<AvailableUpdate | null> {
  let update: Update | null;
  try {
    update = await check();
  } catch (e) {
    // Endpoint unreachable / not configured yet — fail quietly.
    console.warn("[mimic] update check failed:", e);
    return null;
  }
  if (!update) return null;

  return {
    version: update.version,
    currentVersion: update.currentVersion,
    install: async (onProgress) => {
      let total = 0;
      let downloaded = 0;
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength ?? 0;
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          if (total > 0) onProgress(Math.min(1, downloaded / total));
        } else if (event.event === "Finished") {
          onProgress(1);
        }
      });
      await relaunch();
    },
  };
}
