import { getCurrentWebview } from "@tauri-apps/api/webview";

export const TEXT_SIZE_SCALE_MAP: Record<string, number> = {
  small: 0.9,
  default: 1,
  large: 1.1,
  xlarge: 1.2,
};

export async function applyTextSize(size: string) {
  const scale = TEXT_SIZE_SCALE_MAP[size] ?? 1;

  document.documentElement.style.zoom = "1";

  try {
    await getCurrentWebview().setZoom(scale);
  } catch {
    document.documentElement.style.zoom = String(scale);
  }
}
