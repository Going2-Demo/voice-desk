import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Atalho } from "../../ipc/bindings";
import type { Estado, PonteDitado } from "../../ipc/ponte-ditado";
import type { ApiPermissoes } from "../configuracoes/permissoes/api";
import type { ApiChaves } from "../configuracoes/provedores/api";
import type { ApiPrimeiroUso } from "./api";
import Pagina from "./pagina";

vi.mock("../configuracoes/permissoes/api", () => ({ api: undefined }));

const atalho: Atalho = {
  combinacao: "Ctrl+Alt+Space",
  exibicao: "Ctrl+Alt+Espaço",
  padrao: "Ctrl+Alt+Space",
};

let ouvinte: (e: Estado) => void;
function ponte(extra: Partial<PonteDitado> = {}): PonteDitado {
  return {
    obterAtalho: vi.fn().mockResolvedValue(atalho),
    salvarAtalho: vi.fn().mockResolvedValue({ ok: true, atalho }),
    alternarDitado: vi.fn().mockResolvedValue(undefined),
    abrirChaves: vi.fn().mockResolvedValue(undefined),
    aoEstado: vi.fn().mockImplementation(async (f: (e: Estado) => void) => {
      ouvinte = f;
      return () => undefined;
    }),
    aoNivel: vi.fn().mockResolvedValue(() => undefined),
    ...extra,
  };
}

function apiPU(): ApiPrimeiroUso {
  const e = { concluido: false, pulados: [] as string[] };
  return {
    estado: vi.fn().mockResolvedValue(e),
    pular: vi.fn().mockImplementation(async (p: string) => ({ ...e, pulados: [p] })),
    cumprir: vi.fn().mockResolvedValue(e),
    concluir: vi.fn().mockResolvedValue({ ...e, concluido: true }),
  };
}

function chaves(extra: Partial<ApiChaves> = {}): ApiChaves {
  return {
    estado: vi.fn(),
    salvar: vi.fn().mockResolvedValue({
      provedor: "groq",
      salva: true,
      mascara: "••••abcd",
      validade: "valida",
      motivo: null,
    }),
    testar: vi.fn(),
    apagar: vi.fn(),
    abrirPagina: vi.fn().mockResolvedValue(undefined),
    ...extra,
  };
}

const permissoesOk: ApiPermissoes = {
  estado: vi.fn().mockResolvedValue({
    plataforma: "windows",
    microfone: "concedida",
    acessibilidade: "nao_se_aplica",
  }),
  pedir: vi.fn(),
  abrirConfiguracoes: vi.fn(),
};

vi.mock("../configuracoes/permissoes/permissoes", async () => {
  const { ListaPermissoes } = await vi.importActual<
    typeof import("../configuracoes/permissoes/permissoes")
  >("../configuracoes/permissoes/permissoes");
  return { ListaPermissoes: () => <ListaPermissoes api={permissoesOk} /> };
});

function desenhar(props: { api?: ApiPrimeiroUso; ponte?: PonteDitado; chaves?: ApiChaves } = {}) {
  const a = props.api ?? apiPU();
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <MemoryRouter initialEntries={["/primeiro-uso"]}>
        <Routes>
          <Route
            path="/primeiro-uso"
            element={
              <Pagina api={a} ponte={props.ponte ?? ponte()} chaves={props.chaves ?? chaves()} />
            }
          />
          <Route path="/inicio" element={<h1>Início</h1>} />
          <Route path="/config/local" element={<h1>Modelo local</h1>} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  );
  return a;
}

const seguir = (vezes: number) => {
  for (let i = 0; i < vezes; i++)
    fireEvent.click(screen.getByRole("button", { name: "Continuar" }));
};

beforeEach(() => sessionStorage.clear());

describe("primeiro uso guiado (F-06)", () => {
  it("começa nas boas-vindas e mostra os seis passos", () => {
    desenhar();
    expect(screen.getByRole("heading", { name: "Bem-vindo ao Voice Desk" })).toBeInTheDocument();
    expect(screen.getAllByRole("listitem").length).toBeGreaterThanOrEqual(6);
    expect(screen.getByText(/Boas-vindas/)).toHaveAttribute("aria-current", "step");
  });

  it("pular um passo o marca como pulado e segue (RF-06.2)", async () => {
    const a = desenhar();
    seguir(1);
    fireEvent.click(screen.getByRole("button", { name: "Pular este passo" }));
    await waitFor(() => expect(a.pular).toHaveBeenCalledWith("permissoes"));
    expect(screen.getByRole("heading", { name: "Escolha o atalho" })).toBeInTheDocument();
  });

  it("pulando a chave do Groq o primeiro uso termina e vai para o app (RF-06.2)", async () => {
    const a = desenhar();
    seguir(3);
    expect(screen.getByRole("heading", { name: "Como transcrever?" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Pular este passo" }));
    await waitFor(() => expect(a.pular).toHaveBeenCalledWith("provedor"));
    seguir(1);
    fireEvent.click(screen.getByRole("button", { name: "Concluir" }));
    await waitFor(() => expect(a.concluir).toHaveBeenCalled());
    expect(await screen.findByRole("heading", { name: "Início" })).toBeInTheDocument();
  });

  it("não deixa pular as boas-vindas nem o teste", () => {
    desenhar();
    expect(screen.queryByRole("button", { name: "Pular este passo" })).toBeNull();
    seguir(5);
    expect(screen.queryByRole("button", { name: "Pular este passo" })).toBeNull();
  });

  it("mostra o passo das permissões", async () => {
    desenhar();
    seguir(1);
    expect(await screen.findByText("Para gravar a sua voz.")).toBeInTheDocument();
  });

  it("salva o atalho escolhido", async () => {
    const p = ponte();
    const a = desenhar({ ponte: p });
    seguir(2);
    const campo = await screen.findByLabelText("Atalho global");
    fireEvent.change(campo, { target: { value: "Ctrl+Shift+D" } });
    fireEvent.click(screen.getByRole("button", { name: "Salvar atalho" }));
    await waitFor(() => expect(p.salvarAtalho).toHaveBeenCalledWith("Ctrl+Shift+D"));
    await waitFor(() => expect(a.cumprir).toHaveBeenCalledWith("atalho"));
  });

  it("mostra o motivo quando o atalho é recusado", async () => {
    desenhar({
      ponte: ponte({
        salvarAtalho: vi
          .fn()
          .mockResolvedValue({ ok: false, mensagem: "Esse atalho é do sistema." }),
      }),
    });
    seguir(2);
    await screen.findByLabelText("Atalho global");
    fireEvent.click(screen.getByRole("button", { name: "Salvar atalho" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Esse atalho é do sistema.");
  });

  it("guarda a chave do Groq e explica como obter uma (RF-06.4)", async () => {
    const c = chaves();
    const a = desenhar({ chaves: c });
    seguir(3);
    fireEvent.click(screen.getByRole("button", { name: /Como obter uma chave/ }));
    expect(c.abrirPagina).toHaveBeenCalledWith("groq");
    fireEvent.change(screen.getByLabelText("Chave de API do Groq"), {
      target: { value: "gsk_123abcd" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Salvar e testar chave" }));
    await waitFor(() => expect(c.salvar).toHaveBeenCalledWith("groq", "gsk_123abcd"));
    expect(await screen.findByText("Chave guardada neste computador.")).toBeInTheDocument();
    expect(a.cumprir).toHaveBeenCalledWith("provedor");
    expect(screen.getByLabelText("Chave de API do Groq")).toHaveValue("");
  });

  it("chave recusada pelo Groq não conta como passo cumprido", async () => {
    const c = chaves({
      salvar: vi.fn().mockResolvedValue({
        provedor: "groq",
        salva: true,
        mascara: "••••",
        validade: "invalida",
        motivo: "x",
      }),
    });
    const a = desenhar({ chaves: c });
    seguir(3);
    fireEvent.change(screen.getByLabelText("Chave de API do Groq"), {
      target: { value: "gsk_ruim" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Salvar e testar chave" }));
    expect(await screen.findByText(/o Groq a recusou/)).toBeInTheDocument();
    expect(a.cumprir).not.toHaveBeenCalled();
  });

  it("pede a chave quando o campo está vazio e mostra o erro do cofre", async () => {
    const c = chaves({ salvar: vi.fn().mockRejectedValue(new Error("Cofre indisponível.")) });
    desenhar({ chaves: c });
    seguir(3);
    fireEvent.click(screen.getByRole("button", { name: "Salvar e testar chave" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Cole a chave de API do Groq.");
    fireEvent.change(screen.getByLabelText("Chave de API do Groq"), { target: { value: "gsk_x" } });
    fireEvent.click(screen.getByRole("button", { name: "Salvar e testar chave" }));
    await waitFor(() => expect(screen.getByRole("alert")).toHaveTextContent("Cofre indisponível."));
  });

  it("sem chave, a transcrição local segue o passo a passo (RF-06.4)", async () => {
    const a = desenhar();
    seguir(3);
    fireEvent.click(screen.getByRole("button", { name: "Usar transcrição local" }));
    await waitFor(() => expect(a.pular).toHaveBeenCalledWith("provedor"));
    expect(screen.getByRole("heading", { name: "Sincronização (opcional)" })).toBeInTheDocument();
  });

  it("no teste, o texto ditado aparece na caixa e o ditado começa com o foco nela (RF-06.3)", async () => {
    const p = ponte();
    desenhar({ ponte: p });
    seguir(5);
    const caixa = screen.getByLabelText("Caixa de teste");
    fireEvent.click(screen.getByRole("button", { name: /Iniciar ou parar o ditado de teste/ }));
    expect(p.alternarDitado).toHaveBeenCalled();
    expect(caixa).toHaveFocus();
    fireEvent.change(caixa, { target: { value: "olá mundo" } });
    expect(caixa).toHaveValue("olá mundo");
  });

  it("se o ditado falhar, mostra o motivo (RF-06.3)", async () => {
    desenhar();
    seguir(5);
    await waitFor(() => expect(ouvinte).toBeDefined());
    ouvinte({
      fase: "erro",
      mensagem: "Sem acesso ao microfone.",
      segundos: 0,
      programa: null,
      acao: null,
    });
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Não deu certo: Sem acesso ao microfone.",
    );
  });

  it("lembra o passo ao voltar de outra tela", () => {
    sessionStorage.setItem("primeiro-uso.passo", "4");
    desenhar();
    expect(screen.getByRole("heading", { name: "Sincronização (opcional)" })).toBeInTheDocument();
  });

  it("volta ao passo anterior", () => {
    desenhar();
    seguir(1);
    fireEvent.click(screen.getByRole("button", { name: "Voltar" }));
    expect(screen.getByRole("heading", { name: "Bem-vindo ao Voice Desk" })).toBeInTheDocument();
  });

  it("mostra o erro se não conseguir concluir", async () => {
    const a = apiPU();
    a.concluir = vi.fn().mockRejectedValue(new Error("Não gravou."));
    desenhar({ api: a });
    seguir(5);
    fireEvent.click(screen.getByRole("button", { name: "Concluir" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Não gravou.");
  });
});
