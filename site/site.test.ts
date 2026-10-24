// @vitest-environment node
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const ler = (nome: string) => readFileSync(new URL(nome, import.meta.url), "utf8");
const html = ler("./index.html");
const versoes = JSON.parse(ler("./versoes.json")) as {
  versao: string;
  data: string;
  mudancas: string[];
}[];
const brData = (iso: string) => iso.split("-").reverse().join("/");

describe("página do projeto (F-27)", () => {
  it("RF-27.1: baixa os dois instaladores da última versão sem login", () => {
    const base = "https://github.com/going2-demo/voice-desk/releases/latest/download/";
    expect(html).toContain(`data-instalador="mac" href="${base}voice-desk-mac.dmg"`);
    expect(html).toContain(`data-instalador="windows" href="${base}voice-desk-windows.exe"`);
  });

  it("RF-27.2: traz em português os tópicos obrigatórios", () => {
    expect(html).toContain('lang="pt-BR"');
    for (const t of [
      "Requisitos mínimos",
      "No Mac",
      "No Windows",
      "Microfone",
      "Acessibilidade",
      "chave do Groq",
      "Usar sem chave",
      "Código aberto",
      "licença",
      "não tem relação com o VoiceInk",
    ]) {
      expect(html).toContain(t);
    }
  });

  it("RF-27.3: cada versão tem número, data ISO e lista do que mudou", () => {
    expect(versoes.length).toBeGreaterThan(0);
    for (const v of versoes) {
      expect(v.versao).toMatch(/^\d+\.\d+\.\d+$/);
      expect(v.data).toMatch(/^\d{4}-\d{2}-\d{2}$/);
      expect(v.mudancas.length).toBeGreaterThan(0);
    }
  });

  it("a versão mais recente (primeira) é a do app e aparece no HTML sem JavaScript", () => {
    const pacote = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8"));
    const atual = versoes[0];
    expect(atual.versao).toBe(pacote.version);
    expect(html).toContain(`<td><b>${atual.versao}</b></td><td>${brData(atual.data)}</td>`);
    expect(html).toContain(`<span id="versao-atual">${atual.versao}</span>`);
  });

  it("não inclui scripts externos nem referências ao VoiceInk além do aviso", () => {
    expect(html).not.toMatch(/<script[^>]+src="https?:/);
    expect(html.match(/VoiceInk/g)?.length).toBe(1);
  });
});
