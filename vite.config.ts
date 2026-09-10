import { defineConfig } from "vitest/config";

export default defineConfig({
  root: "src",
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        about: new URL("./src/about.html", import.meta.url).pathname,
        capture: new URL("./src/index.html", import.meta.url).pathname,
        history: new URL("./src/history.html", import.meta.url).pathname,
      },
    },
  },
  test: {
    environment: "jsdom",
    include: ["**/*.test.ts"],
  },
});
