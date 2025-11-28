export interface Clip {
  id: string;
  content: string;
  created_at: string;
  tags: string[];
  additional_notes?: string;
  file_attachment?: string;
  original_filename?: string;
}

export interface PagedResult {
  items: Clip[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export interface SearchFilters {
  start_date?: string;
  end_date?: string;
  tags?: string[];
}

export const FAVORITE_TAG = "$favorite";

export function isFavorite(clip: Clip): boolean {
  return clip.tags.includes(FAVORITE_TAG);
}
