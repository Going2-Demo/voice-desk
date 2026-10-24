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
    const chave = await screen.findByRole("checkbox", { name: /iniciar com o sistema/i });
    await waitFor(() => expect(chave).toBeChecked());
  });

  it("desliga e grava a escolha", async () => {
    definir.mockResolvedValue({ status: "ok", data: false });
    render(<Secao />);
    const chave = await screen.findByRole("checkbox", { name: /iniciar com o sistema/i });
    await waitFor(() => expect(chave).toBeEnabled());
    fireEvent.click(chave);
    expect(definir).toHaveBeenCalledWith(false);
    await waitFor(() => expect(chave).not.toBeChecked());
  });

  it("mostra o erro e mantém o estado quando o sistema recusa", async () => {
    definir.mockResolvedValue({
      status: "error",
      error: { codigo: "inicio_automatico_indisponivel", mensagem: "Não foi possível mudar." },
    });
    render(<Secao />);
    const chave = await screen.findByRole("checkbox", { name: /iniciar com o sistema/i });
    await waitFor(() => expect(chave).toBeEnabled());
    fireEvent.click(chave);
    expect(await screen.findByRole("alert")).toHaveTextContent("Não foi possível mudar.");
    expect(chave).toBeChecked();
  });
});

describe("página Ícone e menu", () => {
  it("lista os itens do menu e os quatro estados do ícone", async () => {
    render(<Pagina />);
    for (const t of [
      "Iniciar ou parar o ditado",
      "Escolher o modo padrão",
      "Abrir o histórico",
      "Abrir as configurações",
      "Sair do Voice Desk",
    ]) {
      expect(screen.getByText(t)).toBeInTheDocument();
    }
    for (const e of ["Parado", "Gravando", "Processando", "Erro"]) {
      expect(screen.getByText(e)).toBeInTheDocument();
    }
    expect(await screen.findByRole("checkbox")).toBeInTheDocument();
  });
});
