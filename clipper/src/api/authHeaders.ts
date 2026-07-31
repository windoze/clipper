import { invoke } from "@tauri-apps/api/core";

interface AuthSettings {
  useBundledServer?: boolean;
  bundledServerToken?: string | null;
  externalServerToken?: string | null;
}

export async function getAuthHeaders(): Promise<Record<string, string>> {
  const settings = await invoke<AuthSettings>("get_settings");
  const token = settings.useBundledServer
    ? settings.bundledServerToken
    : settings.externalServerToken;

  if (!token) {
    return {};
  }

  return {
    Authorization: `Bearer ${token}`,
  };
}
