import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const { obter, definir } = vi.hoisted(() => ({ obter: vi.fn(), definir: vi.fn() }));
vi.mock("./api", () => ({
  api: { bandejaIniciarComOSistema: obter, bandejaDefinirIniciarComOSistema: definir },
}));

import { Secao } from "../configuracoes/geral/secao-iniciar-com-o-sistema";
import Pagina from "./pagina";

beforeEach(() => {
  obter.mockReset().mockResolvedValue({ status: "ok", data: true });
  definir.mockReset();
});

describe("iniciar com o sistema (RF-05.2)", () => {
  it("vem ligado por padrão", async () => {
    render(<Secao />);
    const chave = await screen.findByRole("switch", { name: /iniciar com o sistema/i });
    await waitFor(() => expect(chave).toBeChecked());
  });

  it("desliga e grava a escolha", async () => {
    definir.mockResolvedValue({ status: "ok", data: false });
    render(<Secao />);
    const chave = await screen.findByRole("switch", { name: /iniciar com o sistema/i });
    await waitFor(() => expect(chave).toBeEnabled());
    fireEvent.click(chave);
    expect(definir).toHaveBeenCalledWith(false);
    await waitFor(() => expect(chave).not.toBeChecked());
    expect(await screen.findByText("Preferência salva")).toBeInTheDocument();
  });

  it("mostra o erro e mantém o estado quando o sistema recusa", async () => {
    definir.mockResolvedValue({
      status: "error",
      error: { codigo: "inicio_automatico_indisponivel", mensagem: "Não foi possível mudar." },
    });
    render(<Secao />);
    const chave = await screen.findByRole("switch", { name: /iniciar com o sistema/i });
    await waitFor(() => expect(chave).toBeEnabled());
    fireEvent.click(chave);
    expect(await screen.findByRole("alert")).toHaveTextContent("Não foi possível mudar.");
    expect(chave).toBeChecked();
  });
});

describe("página Ícone e menu", () => {
  it("lista os itens do menu com atalho e modo padrão marcado", async () => {
    render(<Pagina />);
    const menu = screen.getByRole("navigation", { name: "Menu do Voice Desk" });
    for (const t of [
      "Iniciar ditado",
      "Abrir histórico",
      "Abrir configurações",
      "Sair do Voice Desk",
    ]) {
      expect(menu).toHaveTextContent(t);
    }
    expect(menu.querySelector("kbd")).not.toBeNull();
    expect(menu).toHaveTextContent("✓");
    expect(
      await screen.findByRole("switch", { name: /iniciar com o sistema/i }),
    ).toBeInTheDocument();
  });

  it("o seletor troca o estado do ícone, com símbolo e texto", async () => {
    render(<Pagina />);
    const grupo = screen.getByRole("group", { name: "Estado do ícone" });
    expect(grupo.querySelectorAll("button")).toHaveLength(4);
    for (const [rotulo, simbolo] of [
      ["Gravando", "🔴"],
      ["Processando", "⏳"],
      ["Erro", "⚠️"],
      ["Parado", "🎙️"],
    ]) {
      fireEvent.click(screen.getByRole("button", { name: rotulo }));
      expect(screen.getByRole("button", { name: rotulo })).toHaveAttribute("aria-pressed", "true");
      expect(screen.getByRole("img", { name: new RegExp(rotulo, "i") })).toHaveTextContent(simbolo);
    }
  });

  it("o item de ditado vira Parar ditado quando está gravando", () => {
    render(<Pagina />);
    fireEvent.click(screen.getByRole("button", { name: "Gravando" }));
    expect(screen.getByRole("navigation", { name: "Menu do Voice Desk" })).toHaveTextContent(
      "Parar ditado",
    );
  });
});
