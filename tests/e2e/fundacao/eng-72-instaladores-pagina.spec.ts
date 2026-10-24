import { readFileSync } from "node:fs";
import { expect, test } from "@playwright/test";

const html = readFileSync("site/index.html", "utf8");
const versoes = JSON.parse(readFileSync("site/versoes.json", "utf8"));

test.describe("RF-27.4 tela Sobre (modo simulado)", () => {
  test("mostra versão, licença, link do projeto e aviso do VoiceInk", async ({ page }) => {
    await page.goto("/#/sobre");
    await expect(page.getByRole("heading", { name: "Sobre o Voice Desk" })).toBeVisible();
    await expect(page.getByText("Versão instalada")).toBeVisible();
    await expect(page.getByText("0.1.0")).toBeVisible();
    await expect(page.getByText("MIT")).toBeVisible();
    await expect(page.getByText("não tem relação com o VoiceInk")).toBeVisible();
  });

  test("o link abre a página do projeto", async ({ page }) => {
    await page.goto("/#/sobre");
    await page.route("https://going2-demo.github.io/**", (r) => r.fulfill({ body: "ok" }));
    const popup = page.waitForEvent("popup");
    await page.getByRole("button", { name: /Página do projeto/ }).click();
    expect((await popup).url()).toContain("going2-demo.github.io/voice-desk");
  });
});

test.describe("RF-27.1 a 27.3 página do projeto (site/)", () => {
  test("baixa Mac e Windows sem cadastro, a partir do latest", async ({ page }) => {
    await page.goto(`file://${process.cwd()}/site/index.html`);
    const mac = page.locator('a[data-instalador="mac"]');
    const win = page.locator('a[data-instalador="windows"]');
    await expect(mac).toHaveAttribute("href", /releases\/latest\/download\/.*\.dmg$/);
    await expect(win).toHaveAttribute("href", /releases\/latest\/download\/.*\.exe$/);
    await expect(page.getByRole("textbox")).toHaveCount(0);
  });

  test("conteúdo em português: requisitos, instalação, permissões, Groq, sem chave, licença e código", () => {
    expect(html).toContain('lang="pt-BR"');
    for (const t of [
      "Requisitos mínimos",
      "Como instalar",
      "chave do Groq",
      "Código aberto",
      "MIT",
    ]) {
      expect(html).toContain(t);
    }
    expect(html.toLowerCase()).toContain("conceda");
    expect(html.toLowerCase()).toMatch(/sem chave|transcrição local/);
  });

  test("versões: a atual tem número, data e lista de mudanças", async ({ page }) => {
    const atual = versoes[0];
    expect(atual.versao && atual.data && atual.mudancas.length > 0).toBeTruthy();
    await page.goto(`file://${process.cwd()}/site/index.html`);
    await expect(page.locator("#versao-atual")).toHaveText(atual.versao);
    await expect(page.locator("#t-versoes")).toBeVisible();
  });
});

test("RF-27.1 (público): a página pública responde sem login", async ({ request }) => {
  test.skip(!process.env.SITE_URL, "SITE_URL não definido: repositório privado/Pages desativado");
  const r = await request.get(process.env.SITE_URL as string);
  expect(r.status()).toBe(200);
});
