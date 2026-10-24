import { expect, test } from "@playwright/test";

// ENG-70 / F-05 (modo simulado). O ícone na bandeja e o reinício são nativos: ver tests/nativo/pendencias.md.

test.describe("F-05 ícone e menu", () => {
  test("a página lista os itens do menu e os quatro estados do ícone (RF-05.1, RF-05.3)", async ({
    page,
  }) => {
    await page.goto("/#/bandeja");
    await expect(page.getByRole("heading", { name: "Ícone e menu" })).toBeVisible();
    const menu = page.getByRole("navigation", { name: "Menu do Voice Desk" });
    for (const item of [
      "Iniciar ditado",
      "Modo padrão",
      "Abrir histórico",
      "Abrir configurações",
      "Sair do Voice Desk",
    ]) {
      await expect(menu.getByText(item)).toBeVisible();
    }
    const icone = page.getByRole("img", { name: /Ícone do Voice Desk/ });
    for (const estado of ["Parado", "Gravando", "Processando", "Erro"]) {
      await page.getByRole("button", { name: estado, exact: true }).click();
      await expect(icone).toHaveAccessibleName(new RegExp(estado, "i"));
    }
    await page.getByRole("button", { name: "Gravando", exact: true }).click();
    await expect(menu.getByText("Parar ditado")).toBeVisible();
  });

  test("a página está na lateral e abre pelo link", async ({ page }) => {
    await page.goto("/#/config/geral");
    await page.getByRole("link", { name: "Ícone e menu" }).click();
    await expect(page).toHaveURL(/#\/bandeja$/);
  });
});

test.describe("F-05 iniciar com o sistema (RF-05.2)", () => {
  for (const caminho of ["/bandeja", "/config/geral"]) {
    test(`vem ligado por padrão, desliga e religa em ${caminho}`, async ({ page }) => {
      await page.goto(`/#${caminho}`);
      const chave = page.getByRole("switch", { name: /iniciar com o sistema/i });
      await expect(chave).toBeChecked();
      await chave.uncheck();
      await expect(chave).not.toBeChecked();
      await chave.check();
      await expect(chave).toBeChecked();
      await expect(page.getByRole("alert")).toHaveCount(0);
    });
  }

  test("a escolha vale nas duas telas na mesma sessão", async ({ page }) => {
    await page.goto("/#/bandeja");
    await page.getByRole("switch", { name: /iniciar com o sistema/i }).uncheck();
    await page.getByRole("link", { name: "Geral" }).click();
    await expect(page.getByRole("switch", { name: /iniciar com o sistema/i })).not.toBeChecked();
  });
});
