import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Estado, PonteDitado } from "../../ipc/ponte-ditado";
import { Gravador, formatarTempo } from "./gravador";

function ponteFalsa(abrirChaves = async () => {}) {
  let estado: (e: Estado) => void = () => {};
  let nivel: (n: number) => void = () => {};
  const ponte = {
    abrirChaves,
    aoEstado: async (f) => {
      estado = f;
      return () => {};
    },
    aoNivel: async (f) => {
      nivel = f;
      return () => {};
    },
  } as PonteDitado;
  return {
    ponte,
    estado: (e: Partial<Estado>) =>
      act(() => estado({ fase: "gravando", mensagem: "", segundos: 0, programa: null, ...e })),
    nivel: (n: number) => act(() => nivel(n)),
  };
}

describe("gravador flutuante", () => {
  it("formata o tempo como m:ss", () => {
    expect(formatarTempo(0)).toBe("0:00");
    expect(formatarTempo(7)).toBe("0:07");
    expect(formatarTempo(570)).toBe("9:30");
    expect(formatarTempo(600)).toBe("10:00");
  });

  it("fica vazio até chegar o primeiro estado", () => {
    const f = ponteFalsa();
    render(<Gravador ponte={f.ponte} />);
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("mostra gravando com mensagem, tempo e medidor de nível", async () => {
    const f = ponteFalsa();
    render(<Gravador ponte={f.ponte} />);
    await f.estado({
      fase: "gravando",
      mensagem: "Gravando… acione o atalho de novo para parar",
      segundos: 7,
    });
    expect(screen.getByRole("status")).toHaveTextContent(
      "Gravando… acione o atalho de novo para parar",
    );
    expect(screen.getByText("0:07")).toBeInTheDocument();
    const medidor = screen.getByRole("meter", { name: "Nível do áudio" });
    expect(medidor).toHaveValue(0);
    await f.nivel(0.8);
    expect(screen.getByRole("meter", { name: "Nível do áudio" })).toHaveValue(80);
    expect(document.querySelector(".gravador")).toHaveAttribute("data-fase", "gravando");
  });

  it("sem chave do Groq avisa antes de gravar e oferece o atalho para as chaves", async () => {
    const abrir = vi.fn(async () => {});
    const f = ponteFalsa(abrir);
    render(<Gravador ponte={f.ponte} />);
    await f.estado({
      fase: "erro",
      mensagem: "Cadastre a chave do Groq em Configurações › Provedores e chaves.",
      acao: "abrir_chaves",
    });
    expect(screen.getByRole("status")).toHaveTextContent("Cadastre a chave do Groq");
    await userEvent.click(screen.getByRole("button", { name: "Abrir chaves" }));
    expect(abrir).toHaveBeenCalledTimes(1);
  });

  it("não mostra o atalho das chaves em outros avisos", async () => {
    const f = ponteFalsa();
    render(<Gravador ponte={f.ponte} />);
    await f.estado({ fase: "erro", mensagem: "Sem conexão com o provedor." });
    expect(screen.queryByRole("button")).toBeNull();
  });

  it("no processamento mostra o aviso e esconde o medidor e o tempo", async () => {
    const f = ponteFalsa();
    render(<Gravador ponte={f.ponte} />);
    await f.estado({ fase: "processando", mensagem: "Processando…" });
    expect(screen.getByRole("status")).toHaveTextContent("Processando…");
    expect(screen.queryByRole("meter")).toBeNull();
    expect(screen.queryByText("0:00")).toBeNull();
  });

  it("mostra 'aguarde' quando o atalho é acionado durante o processamento", async () => {
    const f = ponteFalsa();
    render(<Gravador ponte={f.ponte} />);
    await f.estado({ fase: "processando", mensagem: "Aguarde: ainda processando" });
    expect(screen.getByRole("status")).toHaveTextContent("Aguarde: ainda processando");
  });

  it("mostra o erro de microfone com o caminho para conceder", async () => {
    const f = ponteFalsa();
    render(<Gravador ponte={f.ponte} />);
    await f.estado({
      fase: "erro",
      mensagem: "Sem acesso ao microfone. Conceda a permissão em Configurações › Permissões.",
    });
    expect(screen.getByRole("status")).toHaveTextContent("Sem acesso ao microfone");
    expect(screen.getByRole("status")).toHaveTextContent("Permissões");
    expect(document.querySelector(".gravador")).toHaveAttribute("data-fase", "erro");
  });

  it("zera o nível ao sair da gravação", async () => {
    const f = ponteFalsa();
    render(<Gravador ponte={f.ponte} />);
    await f.estado({ fase: "gravando", mensagem: "Gravando…" });
    await f.nivel(0.5);
    await f.estado({ fase: "processando", mensagem: "Processando…" });
    await f.estado({ fase: "gravando", mensagem: "Gravando…" });
    expect(screen.getByRole("meter")).toHaveValue(0);
  });
});
