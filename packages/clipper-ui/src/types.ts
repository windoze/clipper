export interface Clip {
  id: string;
  content: string;
  created_at: string;
  tags: string[];
  additional_notes?: string;
  file_attachment?: string;
  original_filename?: string;
  /** Highlighted content with search terms wrapped by highlight markers.
   * Only present in search results when highlight params are provided. */
  highlighted_content?: string;
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

/**
 * Auto-cleanup configuration from the server.
 * Used to calculate time-to-autoclean for visual aging effect.
 */
export interface CleanupConfig {
  /** Whether auto-cleanup is enabled */
  enabled: boolean;
  /** Retention period in days */
  retentionDays?: number;
}

/**
 * Server configuration from the /version endpoint.
 * Used to conditionally show features like sharing.
 */
export interface ServerConfig {
  /** Whether short URL sharing is enabled */
  shortUrlEnabled: boolean;
  /** Base URL for short URLs (if enabled) */
  shortUrlBase?: string;
  /** Short URL expiration in hours (0 = no expiration) */
  shortUrlExpirationHours?: number;
}

/**
 * Calculate the age ratio of a clip (0 = new, 1 = about to be cleaned up).
 * Returns null if cleanup is disabled or clip has meaningful tags.
 *
 * @param clip The clip to calculate age ratio for
 * @param cleanupConfig The server's cleanup configuration
 * @returns A number between 0 and 1, or null if not applicable
 */
export function calculateAgeRatio(clip: Clip, cleanupConfig: CleanupConfig | null): number | null {
  if (!cleanupConfig?.enabled || !cleanupConfig.retentionDays) {
    return null;
  }

  // Check if clip has meaningful tags (not just $host:* tags or no tags)
  // Clips with meaningful tags are protected from auto-cleanup
  const meaningfulTags = clip.tags.filter(
    (tag) => !tag.startsWith("$host:") && tag !== FAVORITE_TAG
  );
  if (meaningfulTags.length > 0) {
    return null;
  }

  // Favorites are also protected
  if (isFavorite(clip)) {
    return null;
  }

  const createdAt = new Date(clip.created_at);
  const now = new Date();
  const ageMs = now.getTime() - createdAt.getTime();
  const retentionMs = cleanupConfig.retentionDays * 24 * 60 * 60 * 1000;

  // Calculate ratio (0 = just created, 1 = at retention limit)
  const ratio = Math.min(1, Math.max(0, ageMs / retentionMs));
  return ratio;
}
