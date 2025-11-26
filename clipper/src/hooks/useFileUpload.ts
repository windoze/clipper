import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Clip } from "../types";

interface UseFileUploadState {
  uploading: boolean;
  error: string | null;
}

interface UseFileUploadReturn extends UseFileUploadState {
  uploadFile: (
    path: string,
    tags?: string[],
    additionalNotes?: string
  ) => Promise<Clip | null>;
  clearError: () => void;
}

export function useFileUpload(): UseFileUploadReturn {
  const [state, setState] = useState<UseFileUploadState>({
    uploading: false,
    error: null,
  });

  const uploadFile = useCallback(
    async (
      path: string,
      tags: string[] = [],
      additionalNotes?: string
    ): Promise<Clip | null> => {
      setState({ uploading: true, error: null });

      try {
        // Add $file tag if not already present
        const finalTags = tags.includes("$file") ? tags : ["$file", ...tags];

        const clip = await invoke<Clip>("upload_file", {
          path,
          tags: finalTags,
          additionalNotes: additionalNotes ?? null,
        });

        setState({ uploading: false, error: null });
        return clip;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setState({ uploading: false, error: errorMessage });
        return null;
      }
    },
    []
  );

  const clearError = useCallback(() => {
    setState((prev) => ({ ...prev, error: null }));
  }, []);

  return {
    ...state,
    uploadFile,
    clearError,
  };
}
