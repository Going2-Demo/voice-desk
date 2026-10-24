import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ApiChaves, EstadoChave } from "./api";
import { ErroChave } from "./erro";

const sem: EstadoChave = {
  provedor: "groq",
  salva: false,
  mascara: null,
  validade: null,
  motivo: null,
};
const valida: EstadoChave = { ...sem, salva: true, mascara: "••••abcd", validade: "valida" };

const fake = vi.hoisted(() => ({
  estado: vi.fn(),
  salvar: vi.fn(),
  testar: vi.fn(),
  apagar: vi.fn(),
  abrirPagina: vi.fn(),
}));
vi.mock("./api", () => ({ api: fake satisfies ApiChaves }));

import Pagina from "./pagina";

function desenhar() {
  const cliente = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={cliente}>
      <Pagina />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  for (const f of Object.values(fake)) f.mockReset();
  fake.abrirPagina.mockResolvedValue(undefined);
});

describe("Provedores e chaves (F-04)", () => {
  it("sem chave mostra o formulário e o link para obter a chave", async () => {
    fake.estado.mockResolvedValue([sem]);
    desenhar();
    expect(await screen.findByLabelText("Chave de API")).toBeInTheDocument();
    expect(screen.getByText("Sem chave")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: /Onde consigo uma chave/ }));
    expect(fake.abrirPagina).toHaveBeenCalledWith("groq");
  });

  it("o campo não deixa a chave legível e recusa vazio sem chamar o núcleo", async () => {
    fake.estado.mockResolvedValue([sem]);
    desenhar();
    const campo = await screen.findByLabelText("Chave de API");
    expect(campo).toHaveAttribute("type", "password");
    await userEvent.click(screen.getByRole("button", { name: "Salvar e testar" }));
    expect(await screen.findByText("Cole a chave de API do Groq.")).toBeInTheDocument();
    expect(campo).toHaveAttribute("aria-invalid", "true");
    expect(fake.salvar).not.toHaveBeenCalled();
  });

  it("salvar chave válida mostra 'válida' e só os 4 últimos caracteres", async () => {
    fake.estado.mockResolvedValue([sem]);
    fake.salvar.mockResolvedValue(valida);
    desenhar();
    await userEvent.type(await screen.findByLabelText("Chave de API"), "gsk_segredo_abcd");
    await userEvent.click(screen.getByRole("button", { name: "Salvar e testar" }));
    expect(await screen.findByText(/Chave válida/)).toBeInTheDocument();
    expect(fake.salvar).toHaveBeenCalledWith("groq", "gsk_segredo_abcd");
    expect(screen.getByTestId("mascara")).toHaveTextContent("••••abcd");
    expect(document.body.textContent).not.toContain("gsk_segredo");
    expect(screen.queryByLabelText("Chave de API")).not.toBeInTheDocument();
  });

  it("chave inventada fica 'inválida' e a mensagem traz o motivo do provedor", async () => {
    fake.estado.mockResolvedValue([sem]);
    fake.salvar.mockResolvedValue({
      ...valida,
      validade: "invalida",
      motivo: "Invalid API Key",
    });
    desenhar();
    await userEvent.type(await screen.findByLabelText("Chave de API"), "gsk_inventada");
    await userEvent.click(screen.getByRole("button", { name: "Salvar e testar" }));
    expect(await screen.findByText(/Chave inválida/)).toBeInTheDocument();
    expect(screen.getByText(/Invalid API Key/)).toBeInTheDocument();
  });

  it("mostra o estado 'testando' enquanto o provedor responde", async () => {
    fake.estado.mockResolvedValue([sem]);
    let responder: (e: EstadoChave) => void = () => {};
    fake.salvar.mockReturnValue(new Promise<EstadoChave>((r) => (responder = r)));
    desenhar();
    await userEvent.type(await screen.findByLabelText("Chave de API"), "gsk_qualquer_1");
    await userEvent.click(screen.getByRole("button", { name: "Salvar e testar" }));
    expect(await screen.findByRole("button", { name: /Testando a chave/ })).toBeDisabled();
    responder(valida);
    expect(await screen.findByText(/Chave válida/)).toBeInTheDocument();
  });

  it("erro de formato vindo do núcleo aparece no campo", async () => {
    fake.estado.mockResolvedValue([sem]);
    fake.salvar.mockRejectedValue(
      new ErroChave("chave_formato", "Essa chave não parece completa.", "chave"),
    );
    desenhar();
    await userEvent.type(await screen.findByLabelText("Chave de API"), "curta");
    await userEvent.click(screen.getByRole("button", { name: "Salvar e testar" }));
    expect(await screen.findByText("Essa chave não parece completa.")).toBeInTheDocument();
  });

  it("trocar chave abre o formulário e cancelar volta à máscara", async () => {
    fake.estado.mockResolvedValue([valida]);
    desenhar();
    await userEvent.click(await screen.findByRole("button", { name: "Trocar chave" }));
    expect(screen.getByLabelText("Chave de API")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Cancelar" }));
    expect(screen.queryByLabelText("Chave de API")).not.toBeInTheDocument();
    expect(screen.getByTestId("mascara")).toHaveTextContent("••••abcd");
  });

  it("apagar pede confirmação e só então remove a chave", async () => {
    fake.estado.mockResolvedValue([valida]);
    fake.apagar.mockResolvedValue(sem);
    desenhar();
    await userEvent.click(await screen.findByRole("button", { name: "Apagar" }));
    expect(fake.apagar).not.toHaveBeenCalled();
    await userEvent.click(screen.getByRole("button", { name: "Apagar chave" }));
    await waitFor(() => expect(fake.apagar).toHaveBeenCalledWith("groq"));
    expect(await screen.findByText("Sem chave")).toBeInTheDocument();
    expect(screen.getByLabelText("Chave de API")).toBeInTheDocument();
  });

  it("cancelar a confirmação não apaga", async () => {
    fake.estado.mockResolvedValue([valida]);
    desenhar();
    await userEvent.click(await screen.findByRole("button", { name: "Apagar" }));
    await userEvent.click(screen.getByRole("button", { name: "Cancelar" }));
    expect(fake.apagar).not.toHaveBeenCalled();
    expect(screen.getByTestId("mascara")).toBeInTheDocument();
  });

  it("chave não testada oferece testar de novo", async () => {
    fake.estado.mockResolvedValue([
      {
        ...valida,
        validade: "nao_testada",
        motivo: "O Groq não respondeu em 10 segundos. Tente de novo.",
      },
    ]);
    fake.testar.mockResolvedValue(valida);
    desenhar();
    expect(await screen.findByText(/não respondeu em 10 segundos/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Testar de novo" }));
    expect(await screen.findByText(/Chave válida/)).toBeInTheDocument();
  });
});
