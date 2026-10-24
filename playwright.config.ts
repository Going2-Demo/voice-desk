import { defineConfig } from "@playwright/test";

const externa = process.env.BASE_URL;

export default defineConfig({
  testDir: "tests/e2e",
  reporter: "line",
  use: { baseURL: externa ?? "http://localhost:4173", trace: "retain-on-failure" },
  webServer: externa
    ? undefined
    : {
        command: "pnpm build:simulado && pnpm preview:simulado",
        url: "http://localhost:4173",
        reuseExistingServer: true,
        timeout: 180_000,
      },
});
