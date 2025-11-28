import { Clip, PagedResult, SearchFilters } from "../types";

// Get base URL - use relative path for production, proxy for development
function getBaseUrl(): string {
  // In development, Vite proxy handles /api -> localhost:3000
  // In production, we serve from the same origin
  return "";
}

async function handleResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || `HTTP ${response.status}`);
  }
  return response.json();
}

export async function listClips(
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

  const response = await fetch(`${getBaseUrl()}/clips?${params.toString()}`);
  return handleResponse<PagedResult>(response);
}

export async function searchClips(
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
    `${getBaseUrl()}/clips/search?${params.toString()}`
  );
  return handleResponse<PagedResult>(response);
}

export async function getClip(id: string): Promise<Clip> {
  const response = await fetch(`${getBaseUrl()}/clips/${id}`);
  return handleResponse<Clip>(response);
}

export async function updateClip(
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

  const response = await fetch(`${getBaseUrl()}/clips/${id}`, {
    method: "PUT",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(body),
  });
  return handleResponse<Clip>(response);
}

export async function deleteClip(id: string): Promise<void> {
  const response = await fetch(`${getBaseUrl()}/clips/${id}`, {
    method: "DELETE",
  });
  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || `HTTP ${response.status}`);
  }
}

export function getFileUrl(clipId: string): string {
  return `${getBaseUrl()}/clips/${clipId}/file`;
}

export async function healthCheck(): Promise<boolean> {
  try {
    const response = await fetch(`${getBaseUrl()}/health`);
    return response.ok;
  } catch {
    return false;
  }
}
