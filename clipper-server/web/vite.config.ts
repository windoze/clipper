import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    dedupe: ["react", "react-dom"],
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    chunkSizeWarningLimit: 1000,
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (id.indexOf("node_modules") !== -1) {
            if (id.indexOf("react-dom") !== -1 || id.indexOf("/react/") !== -1) {
              return "react";
            }
            if (id.indexOf("highlight.js") !== -1) {
              return "hljs";
            }
          }
        },
      },
    },
  },
  server: {
    port: 5173,
    proxy: {
      "/clips": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
      "/health": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
      "/auth": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
      "/version": {
        target: "http://localhost:3000",
        changeOrigin: true,
      },
      "/ws": {
        target: "http://localhost:3000",
        changeOrigin: true,
        ws: true,
      },
    },
    fs: {
      // Allow serving files from the parent packages directory
      allow: [
        path.resolve(__dirname, "../.."),
      ],
    },
  },
});
