import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
vi.hoisted(() => {
  import.meta.env.VITE_IPC = "simulado";
});

import { Secao as TextoDoDitado } from "../configuracoes/geral/secao-texto-do-ditado";
import Pagina from "./pagina";
import { reiniciarSimulado } from "./simulado";

function desenhar(ui = <Pagina />) {
  const cliente = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(<QueryClientProvider client={cliente}>{ui}</QueryClientProvider>);
}

beforeEach(() => reiniciarSimulado(["hã", "tipo", "é…"]));

describe("Limpeza do texto (F-09)", () => {
  it("mostra a lista e a prévia do texto de exemplo", async () => {
    desenhar();
    const lista = await screen.findByRole("list", { name: "Palavras de preenchimento" });
    expect(within(lista).getByText("tipo")).toBeInTheDocument();
    expect(
      await screen.findByText("Então assim, preciso falar com o Pereira né sobre o contrato."),
    ).toBeInTheDocument();
  });

  it("a prévia acompanha o que o usuário digita", async () => {
    desenhar();
    const campo = await screen.findByLabelText("Texto falado");
    await userEvent.clear(campo);
    await userEvent.type(campo, "hã… chego em dez minutos");
    expect(await screen.findByText("Chego em dez minutos.")).toBeInTheDocument();
  });

  it("palavra adicionada passa a ser removida na prévia", async () => {
    desenhar();
    await userEvent.type(await screen.findByLabelText("Nova palavra"), "basicamente");
    await userEvent.click(screen.getByRole("button", { name: "Adicionar" }));
    const lista = await screen.findByRole("list", { name: "Palavras de preenchimento" });
    expect(await within(lista).findByText("basicamente")).toBeInTheDocument();
    const campo = screen.getByLabelText("Texto falado");
    await userEvent.clear(campo);
    await userEvent.type(campo, "basicamente funciona");
    expect(await screen.findByText("Funciona.")).toBeInTheDocument();
  });

  it("duplicada e vazia mostram o erro no campo", async () => {
    desenhar();
    await userEvent.type(await screen.findByLabelText("Nova palavra"), "TIPO{Enter}");
    expect(await screen.findByText("“TIPO” já está na lista.")).toBeInTheDocument();
    await userEvent.clear(screen.getByLabelText("Nova palavra"));
    await userEvent.click(screen.getByRole("button", { name: "Adicionar" }));
    expect(await screen.findByText("Digite uma palavra ou expressão.")).toBeInTheDocument();
  });

  it("remover tira a palavra da lista e a prévia deixa de cortá-la", async () => {
    desenhar();
    await userEvent.click(await screen.findByRole("button", { name: "Remover hã" }));
    const lista = await screen.findByRole("list", { name: "Palavras de preenchimento" });
    expect(within(lista).queryByText("hã")).not.toBeInTheDocument();
    const campo = screen.getByLabelText("Texto falado");
    await userEvent.clear(campo);
    await userEvent.type(campo, "hã sim");
    expect(await screen.findByText("Hã sim.")).toBeInTheDocument();
  });

  it("lista vazia mostra o estado vazio", async () => {
    reiniciarSimulado([]);
    desenhar();
    expect(await screen.findByText(/Lista vazia/)).toBeInTheDocument();
  });

  it("avisa que o texto sai sem alteração com tudo desligado", async () => {
    reiniciarSimulado(["hã"], { remover_preenchimento: false, formatar: false });
    desenhar();
    expect(await screen.findByText(/sai sem alteração/)).toBeInTheDocument();
  });
});

describe("Configurações › Geral › Texto do ditado (RF-09.1)", () => {
  it("liga e desliga remoção e formatação separadamente", async () => {
    desenhar(<TextoDoDitado />);
    const remover = await screen.findByRole("switch", { name: /Remover palavras/ });
    const formatar = screen.getByRole("switch", { name: /Formatar o texto/ });
    await waitFor(() => expect(remover).toBeChecked());
    expect(formatar).toBeChecked();
    await userEvent.click(remover);
    expect(await screen.findByText("Preferência salva.")).toBeInTheDocument();
    expect(screen.getByRole("switch", { name: /Remover palavras/ })).not.toBeChecked();
    expect(screen.getByRole("switch", { name: /Formatar o texto/ })).toBeChecked();
  });

  it("Enter vem desligado e o idioma muda", async () => {
    desenhar(<TextoDoDitado />);
    const enter = await screen.findByRole("switch", { name: /Apertar Enter/ });
    await waitFor(() => expect(enter).toBeEnabled());
    expect(enter).not.toBeChecked();
    await userEvent.selectOptions(screen.getByLabelText("Idioma do ditado"), "en");
    expect(await screen.findByDisplayValue("Inglês")).toBeInTheDocument();
  });

  it("o que foi salvo vale na prévia da tela de limpeza", async () => {
    const { unmount } = desenhar(<TextoDoDitado />);
    await userEvent.click(await screen.findByRole("switch", { name: /Formatar o texto/ }));
    await screen.findByText("Preferência salva.");
    unmount();
    desenhar();
    const campo = await screen.findByLabelText("Texto falado");
    await userEvent.clear(campo);
    await userEvent.type(campo, "hã… chego");
    expect(await screen.findByText("chego")).toBeInTheDocument();
  });
});
