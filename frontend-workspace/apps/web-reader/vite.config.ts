import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import viteCompression from "vite-plugin-compression";

// https://vite.dev/config/
export default defineConfig(({ mode }) => ({
  resolve: {
    alias: {
      // MarkdownEditor in ui-shared conditionally imports Tauri — provide an empty shim for web-reader
      "@tauri-apps/api/core": new URL("./src/shims/tauri-api-core.ts", import.meta.url).pathname,
    },
  },
  plugins: [
    vue(),
    // Gzip / Brotli 压缩（Android 模式跳过，避免 Capacitor 资源冲突）
    mode !== "android" && viteCompression({
      algorithm: "gzip",
      threshold: 1024,
      deleteOriginFile: false,
    }),
    mode !== "android" && viteCompression({
      algorithm: "brotliCompress",
      threshold: 1024,
      deleteOriginFile: false,
    }),
  ].filter(Boolean),
  build: {
    // 精确分包：核心框架 / 图谱引擎 / 虚拟滚动 分离
    rollupOptions: {
      output: {
        // Keep existing chunks and add new ones for long-term caching
        manualChunks(id) {
          if (id.includes("node_modules")) {
            // Core Vue stack
            if (id.includes("vue") || id.includes("pinia")) {
              return "vendor-core";
            }
            // Graph / graphology / potpack / dagre
            // Note: graphology and potpack should be grouped with the graph vendor
            if (
              id.includes("@vue-flow") ||
              id.includes("dagre") ||
              id.includes("graphology") ||
              id.includes("potpack")
            ) {
              return "vendor-graph";
            }
            // Icons (Lucide)
            if (id.includes("lucide-vue-next")) {
              return "vendor-icons";
            }
            // Axios / HTTP client
            if (id.includes("axios")) {
              return "vendor-http";
            }
          }
        },
        // Long-term caching for chunks
        chunkFileNames: "assets/[name]-[hash].js",
      },
    },
    cssCodeSplit: true,
    target: "esnext",
  },
}));
