import { createContext, useContext } from "react";
import { Clip, PagedResult, SearchFilters } from "../types";

/**
 * API client interface that abstracts the differences between
 * REST API (web) and Tauri invoke commands (desktop).
 */
export interface ClipperApi {
  /** List clips with optional filters and pagination */
  listClips(
    filters: SearchFilters,
    page: number,
    pageSize: number
  ): Promise<PagedResult>;

  /** Search clips with query string */
  searchClips(
    query: string,
    filters: SearchFilters,
    page: number,
    pageSize: number
  ): Promise<PagedResult>;

  /** Get a single clip by ID */
  getClip(id: string): Promise<Clip>;

  /** Create a new clip from text content */
  createClip(
    content: string,
    tags?: string[],
    additionalNotes?: string
  ): Promise<Clip>;

  /** Upload a file as a new clip */
  uploadFile(file: File, tags?: string[], additionalNotes?: string): Promise<Clip>;

  /** Update clip tags and/or notes */
  updateClip(
    id: string,
    tags?: string[],
    additionalNotes?: string | null
  ): Promise<Clip>;

  /** Delete a clip */
  deleteClip(id: string): Promise<void>;

  /** Get the URL for a clip's file attachment (sync version, returns URL or empty string) */
  getFileUrl(clipId: string): string;

  /** Get the URL for a clip's file attachment (async version for platforms like Tauri) */
  getFileUrlAsync?: (clipId: string) => Promise<string>;

  /** Copy content to clipboard */
  copyToClipboard(content: string): Promise<void>;

  /** Download a file attachment */
  downloadFile(clipId: string, filename: string): Promise<void>;
}

// Context for the API client
const ApiContext = createContext<ClipperApi | null>(null);

export const ApiProvider = ApiContext.Provider;

/**
 * Hook to access the Clipper API client.
 * Must be used within an ApiProvider.
 */
export function useApi(): ClipperApi {
  const api = useContext(ApiContext);
  if (!api) {
    throw new Error("useApi must be used within an ApiProvider");
  }
  return api;
}

/**
 * Create a REST API client for the web UI.
 * @param baseUrl Base URL for the API (empty string for same-origin)
 */
export function createRestApiClient(baseUrl: string = ""): ClipperApi {
  async function handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
      const text = await response.text();
      throw new Error(text || `HTTP ${response.status}`);
    }
    return response.json();
  }

  return {
    async listClips(
      filters: SearchFilters,
      page: number,
      pageSize: number
    ): Promise<PagedResult> {
      const params = new URLSearchParams();
      params.set("page", String(page));
      params.set("page_size", String(pageSize));

      if (filters.start_date) {
        params.set("start_date", filters.start_date);
      }
      if (filters.end_date) {
        params.set("end_date", filters.end_date);
      }
      if (filters.tags && filters.tags.length > 0) {
        params.set("tags", filters.tags.join(","));
      }

      const response = await fetch(`${baseUrl}/clips?${params.toString()}`);
      return handleResponse<PagedResult>(response);
    },

    async searchClips(
      query: string,
      filters: SearchFilters,
      page: number,
      pageSize: number
    ): Promise<PagedResult> {
      const params = new URLSearchParams();
      params.set("q", query);
      params.set("page", String(page));
      params.set("page_size", String(pageSize));

      if (filters.start_date) {
        params.set("start_date", filters.start_date);
      }
      if (filters.end_date) {
        params.set("end_date", filters.end_date);
      }
      if (filters.tags && filters.tags.length > 0) {
        params.set("tags", filters.tags.join(","));
      }

      const response = await fetch(
        `${baseUrl}/clips/search?${params.toString()}`
      );
      return handleResponse<PagedResult>(response);
    },

    async getClip(id: string): Promise<Clip> {
      const response = await fetch(`${baseUrl}/clips/${id}`);
      return handleResponse<Clip>(response);
    },

    async createClip(
      content: string,
      tags: string[] = [],
      additionalNotes?: string
    ): Promise<Clip> {
      const body: Record<string, unknown> = {
        content,
        tags,
      };
      if (additionalNotes) {
        body.additional_notes = additionalNotes;
      }

      const response = await fetch(`${baseUrl}/clips`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(body),
      });
      return handleResponse<Clip>(response);
    },

    async uploadFile(
      file: File,
      tags: string[] = [],
      additionalNotes?: string
    ): Promise<Clip> {
      const formData = new FormData();
      formData.append("file", file);
      if (tags.length > 0) {
        formData.append("tags", tags.join(","));
      }
      if (additionalNotes) {
        formData.append("additional_notes", additionalNotes);
      }

      const response = await fetch(`${baseUrl}/clips/upload`, {
        method: "POST",
        body: formData,
      });
      return handleResponse<Clip>(response);
    },

    async updateClip(
      id: string,
      tags?: string[],
      additionalNotes?: string | null
    ): Promise<Clip> {
      const body: Record<string, unknown> = {};
      if (tags !== undefined) {
        body.tags = tags;
      }
      if (additionalNotes !== undefined) {
        body.additional_notes = additionalNotes;
      }

      const response = await fetch(`${baseUrl}/clips/${id}`, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(body),
      });
      return handleResponse<Clip>(response);
    },

    async deleteClip(id: string): Promise<void> {
      const response = await fetch(`${baseUrl}/clips/${id}`, {
        method: "DELETE",
      });
      if (!response.ok) {
        const text = await response.text();
        throw new Error(text || `HTTP ${response.status}`);
      }
    },

    getFileUrl(clipId: string): string {
      return `${baseUrl}/clips/${clipId}/file`;
    },

    async copyToClipboard(content: string): Promise<void> {
      await navigator.clipboard.writeText(content);
    },

    async downloadFile(clipId: string, filename: string): Promise<void> {
      const link = document.createElement("a");
      link.href = this.getFileUrl(clipId);
      link.download = filename;
      link.target = "_blank";
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    },
  };
}
