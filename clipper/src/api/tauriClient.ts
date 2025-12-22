import { invoke } from "@tauri-apps/api/core";
import type { ClipperApi, Clip, PagedResult, PagedTagResult, SearchFilters } from "@unwritten-codes/clipper-ui";

/**
 * Create a Tauri API client that uses invoke commands
 * to communicate with the Rust backend.
 */
export function createTauriApiClient(): ClipperApi {
  return {
    async listClips(
      filters: SearchFilters,
      page: number,
      pageSize: number
    ): Promise<PagedResult> {
      return invoke<PagedResult>("list_clips", {
        filters,
        page,
        pageSize,
      });
    },

    async searchClips(
      query: string,
      filters: SearchFilters,
      page: number,
      pageSize: number
    ): Promise<PagedResult> {
      return invoke<PagedResult>("search_clips", {
        query,
        filters,
        page,
        pageSize,
      });
    },

    async getClip(id: string): Promise<Clip> {
      return invoke<Clip>("get_clip", { id });
    },

    async createClip(
      content: string,
      tags?: string[],
      additionalNotes?: string,
      language?: string
    ): Promise<Clip> {
      return invoke<Clip>("create_clip", {
        content,
        tags: tags || [],
        additionalNotes,
        language,
      });
    },

    async uploadFile(
      _file: File,
      _tags?: string[],
      _additionalNotes?: string
    ): Promise<Clip> {
      // In Tauri, file uploads are handled via the path-based upload_file command
      // which is called directly from the drag-and-drop handler, not through this interface.
      // This method is primarily for the web UI.
      throw new Error("Use the upload_file Tauri command with a file path instead");
    },

    async updateClip(
      id: string,
      tags?: string[],
      additionalNotes?: string | null,
      language?: string | null
    ): Promise<Clip> {
      return invoke<Clip>("update_clip", {
        id,
        tags,
        additionalNotes,
        language,
      });
    },

    async deleteClip(id: string): Promise<void> {
      await invoke("delete_clip", { id });
    },

    getFileUrl(_clipId: string): string {
      // For Tauri, return empty - use getFileUrlAsync instead
      return "";
    },

    async getFileUrlAsync(clipId: string, filename?: string): Promise<string> {
      // If filename is provided, fetch the image data through Rust and return as data URL
      // This allows loading images even when the server uses a self-signed certificate
      // that the WebView doesn't trust (but the Rust client does)
      if (filename) {
        return invoke<string>("get_file_data_url", { clipId, filename });
      }
      // Fallback to direct URL (may fail with self-signed certs)
      return invoke<string>("get_file_url", { clipId });
    },

    async copyToClipboard(content: string): Promise<void> {
      await invoke("copy_to_clipboard", { content });
    },

    async copyImageToClipboard(clipId: string): Promise<void> {
      await invoke("copy_image_to_clipboard", { clipId });
    },

    async downloadFile(clipId: string, filename: string): Promise<void> {
      await invoke("download_file", { clipId, filename });
    },

    async shareClip(clipId: string, expiresInHours?: number): Promise<string> {
      // Get the server URL and settings (for auth token)
      const serverUrl = await invoke<string>("get_server_url");
      const settings = await invoke<{
        useBundledServer?: boolean;
        bundledServerToken?: string;
        externalServerToken?: string;
      }>("get_settings");

      // Get the appropriate token based on server mode
      const token = settings.useBundledServer
        ? settings.bundledServerToken
        : settings.externalServerToken;

      const headers: Record<string, string> = {
        "Content-Type": "application/json",
      };
      if (token) {
        headers["Authorization"] = `Bearer ${token}`;
      }

      const body: { expires_in_hours?: number } = {};
      if (expiresInHours !== undefined) {
        body.expires_in_hours = expiresInHours;
      }

      const response = await fetch(`${serverUrl}/clips/${clipId}/short-url`, {
        method: "POST",
        headers,
        body: JSON.stringify(body),
      });
      if (!response.ok) {
        throw new Error(`Failed to share clip: ${response.status}`);
      }
      const result = await response.json();
      return result.full_url;
    },

    async listTags(page: number, pageSize: number): Promise<PagedTagResult> {
      const serverUrl = await invoke<string>("get_server_url");
      const settings = await invoke<{
        useBundledServer?: boolean;
        bundledServerToken?: string;
        externalServerToken?: string;
      }>("get_settings");

      const token = settings.useBundledServer
        ? settings.bundledServerToken
        : settings.externalServerToken;

      const headers: Record<string, string> = {};
      if (token) {
        headers["Authorization"] = `Bearer ${token}`;
      }

      const params = new URLSearchParams();
      params.set("page", String(page));
      params.set("page_size", String(pageSize));

      const response = await fetch(`${serverUrl}/tags?${params.toString()}`, {
        headers,
      });
      if (!response.ok) {
        throw new Error(`Failed to list tags: ${response.status}`);
      }
      return response.json();
    },

    async searchTags(
      query: string,
      page: number,
      pageSize: number
    ): Promise<PagedTagResult> {
      const serverUrl = await invoke<string>("get_server_url");
      const settings = await invoke<{
        useBundledServer?: boolean;
        bundledServerToken?: string;
        externalServerToken?: string;
      }>("get_settings");

      const token = settings.useBundledServer
        ? settings.bundledServerToken
        : settings.externalServerToken;

      const headers: Record<string, string> = {};
      if (token) {
        headers["Authorization"] = `Bearer ${token}`;
      }

      const params = new URLSearchParams();
      params.set("q", query);
      params.set("page", String(page));
      params.set("page_size", String(pageSize));

      const response = await fetch(`${serverUrl}/tags/search?${params.toString()}`, {
        headers,
      });
      if (!response.ok) {
        throw new Error(`Failed to search tags: ${response.status}`);
      }
      return response.json();
    },
  };
}
