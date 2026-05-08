import { defineConfig } from "vitest/config";
import path from "path";

export default defineConfig({
  test: {
    environment: "happy-dom",
    include: ["__tests__/**/*.test.js"],
  },
  resolve: {
    alias: {
      "@ui": path.resolve(__dirname, "ui"),
    },
  },
});
