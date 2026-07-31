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
      file: File,
      tags?: string[],
      additionalNotes?: string
    ): Promise<Clip> {
      const maxUploadSizeBytes = await invoke<number>("get_max_upload_size_bytes");
      if (file.size > maxUploadSizeBytes) {
        const fileSizeMb = file.size / (1024 * 1024);
        const maxSizeMb = maxUploadSizeBytes / (1024 * 1024);
        throw new Error(
          `File size (${fileSizeMb.toFixed(2)} MB) exceeds maximum allowed size (${maxSizeMb.toFixed(2)} MB)`
        );
      }

      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      return invoke<Clip>("upload_file_bytes", {
        bytes,
        filename: file.name || "uploaded_file",
        tags: tags || [],
        additionalNotes,
      });
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
      return invoke<string>("share_clip", {
        clipId,
        expiresInHours: expiresInHours ?? null,
      });
    },

    async listTags(page: number, pageSize: number): Promise<PagedTagResult> {
      return invoke<PagedTagResult>("list_tags", {
        page,
        pageSize,
      });
    },

    async searchTags(
      query: string,
      page: number,
      pageSize: number
    ): Promise<PagedTagResult> {
      return invoke<PagedTagResult>("search_tags", {
        query,
        page,
        pageSize,
      });
    },
  };
}
