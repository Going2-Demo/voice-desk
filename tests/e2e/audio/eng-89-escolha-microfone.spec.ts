import { expect, test } from "@playwright/test";
import { emitir, instalarTauri } from "../ditado/tauri-mock";
import { irPara } from "../helpers/navegar";

const estado = (fase: string, mensagem: string, segundos = 0) => ({ fase, mensagem, segundos });

test.describe("ENG-89 · F-23 escolha do microfone (interface, núcleo simulado)", () => {
  test("RF-23.1 lista os dispositivos conectados e 'Padrão do sistema'; a escolha é salva", async ({
    page,
  }) => {
    await irPara(page, "/config/geral");
    await expect(page.getByRole("heading", { level: 2, name: "Microfone" })).toBeVisible();
    const campo = page.getByLabel("Microfone de entrada");
    await expect(campo).toHaveValue("");
    await expect(campo.getByRole("option")).toHaveText([
      "Padrão do sistema",
      "MacBook Pro — Microfone",
      "Headset USB Logitech H390",
    ]);

    await campo.selectOption("Headset USB Logitech H390");
    await expect(page.getByText("Microfone salvo")).toBeVisible();
    await expect(campo).toHaveValue("Headset USB Logitech H390");
    await expect(page.getByRole("status").filter({ hasText: "Conectado" })).toBeVisible();

    await campo.selectOption("");
    await expect(campo).toHaveValue("");
    await expect(page.getByRole("status").filter({ hasText: "Conectado" })).toHaveCount(0);
  });

  test("RF-23.1 a escolha persiste ao reabrir a tela (mesma sessão)", async ({ page }) => {
    await irPara(page, "/config/geral");
    await page.getByLabel("Microfone de entrada").selectOption("MacBook Pro — Microfone");
    await expect(page.getByText("Microfone salvo")).toBeVisible();
    await page.evaluate(() => {
      window.location.hash = "#/modos";
    });
    await page.evaluate(() => {
      window.location.hash = "#/config/geral";
    });
    await expect(page.getByLabel("Microfone de entrada")).toHaveValue("MacBook Pro — Microfone");
  });

  test("critério: com o headset em uso, o nível do gravador reage à voz", async ({ page }) => {
    await instalarTauri(page, {});
    await page.goto("/gravador.html");
    await emitir(page, "ditado-estado", estado("gravando", "Gravando…", 2));
    const nivel = page.getByRole("meter", { name: "Nível do áudio" });
    await emitir(page, "ditado-nivel", 0.2);
    await expect(nivel).toHaveAttribute("value", "20");
    await emitir(page, "ditado-nivel", 0.8);
    await expect(nivel).toHaveAttribute("value", "80");
  });

  test("RF-23.2 headset desconectado: o gravador avisa qual microfone está em uso", async ({
    page,
  }) => {
    await instalarTauri(page, {});
    await page.goto("/gravador.html");
    await emitir(
      page,
      "ditado-estado",
      estado(
        "gravando",
        "Headset desconectado. Usando o padrão do sistema: MacBook Pro — Microfone",
        1,
      ),
    );
    await expect(page.getByRole("status")).toContainText("padrão do sistema");
    await expect(page.getByRole("status")).toContainText("MacBook Pro");
  });

  test("critério: sem nenhum microfone, o gravador avisa e não grava", async ({ page }) => {
    await instalarTauri(page, {});
    await page.goto("/gravador.html");
    await emitir(page, "ditado-estado", estado("erro", "Nenhum microfone encontrado."));
    await expect(page.getByText("Nenhum microfone encontrado.")).toBeVisible();
    await expect(page.getByRole("meter", { name: "Nível do áudio" })).toHaveCount(0);
  });
});
