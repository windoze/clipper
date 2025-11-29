import { invoke } from "@tauri-apps/api/core";
import type { ClipperApi, Clip, PagedResult, SearchFilters } from "@unwritten-codes/clipper-ui";

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
      additionalNotes?: string
    ): Promise<Clip> {
      return invoke<Clip>("create_clip", {
        content,
        tags: tags || [],
        additionalNotes,
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
      additionalNotes?: string | null
    ): Promise<Clip> {
      return invoke<Clip>("update_clip", {
        id,
        tags,
        additionalNotes,
      });
    },

    async deleteClip(id: string): Promise<void> {
      await invoke("delete_clip", { id });
    },

    getFileUrl(_clipId: string): string {
      // For Tauri, return empty - use getFileUrlAsync instead
      return "";
    },

    async getFileUrlAsync(clipId: string): Promise<string> {
      return invoke<string>("get_file_url", { clipId });
    },

    async copyToClipboard(content: string): Promise<void> {
      await invoke("copy_to_clipboard", { content });
    },

    async downloadFile(clipId: string, filename: string): Promise<void> {
      await invoke("download_file", { clipId, filename });
    },
  };
}
