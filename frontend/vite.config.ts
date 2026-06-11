import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vitest/config";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  test: {
    setupFiles: ["./src/locales/test-setup.ts"],
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          const packagePath = id.split("node_modules/")[1];
          if (!packagePath) {
            return undefined;
          }
          const [scopeOrName, name] = packagePath.split("/");
          const packageName = scopeOrName.startsWith("@") ? `${scopeOrName}/${name}` : scopeOrName;

          if (
            packageName === "react" ||
            packageName === "react-dom" ||
            packageName === "scheduler"
          ) {
            return "react-vendor";
          }

          if (packageName === "katex") {
            return "math-vendor";
          }

          if (packageName === "parse5" || packageName === "entities") {
            return "html-parser-vendor";
          }

          if (packageName === "i18next" || packageName === "react-i18next") {
            return "i18n-vendor";
          }

          if (packageName === "chroma-js") {
            return "color-vendor";
          }

          return undefined;
        },
      },
    },
  },
}));
