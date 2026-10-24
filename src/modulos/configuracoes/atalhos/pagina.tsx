import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useState } from "react";
import { consultas } from "../../../ipc/consultas";
import { type PonteDitado, ponteReal } from "../../../ipc/ponte-ditado";
import "./atalhos.css";

export function PaginaAtalhos({ ponte = ponteReal }: { ponte?: PonteDitado }) {
  const atual = useQuery({ queryKey: consultas.atalhos.global, queryFn: ponte.obterAtalho });
  return (
    <section className="atalhos">
      <h1>Configurações · Atalhos</h1>
      {atual.isError ? (
        <p role="alert" className="erro-campo">
          Não foi possível ler o atalho atual. Feche e abra o Voice Desk.
        </p>
      ) : atual.data ? (
        <Conteudo inicial={atual.data.exibicao} ponte={ponte} />
      ) : (
        <p className="dica" role="status">
          Carregando…
        </p>
      )}
    </section>
  );
}

function Conteudo({ inicial, ponte }: { inicial: string; ponte: PonteDitado }) {
  const cliente = useQueryClient();
  const [valor, setValor] = useState(inicial);
  const [erro, setErro] = useState("");
  const [salvo, setSalvo] = useState(false);

  const salvar = useMutation({
    mutationFn: (combinacao: string) => ponte.salvarAtalho(combinacao),
    onSuccess: (r) => {
      if (r.ok) {
        setErro("");
        setSalvo(true);
        setValor(r.atalho.exibicao);
        cliente.setQueryData(consultas.atalhos.global, r.atalho);
      } else {
        setSalvo(false);
        setErro(r.mensagem);
      }
    },
  });

  function enviar(e: FormEvent) {
    e.preventDefault();
    setSalvo(false);
    if (!valor.trim()) {
      setErro("Digite uma combinação, por exemplo Ctrl+Alt+Espaço.");
      return;
    }
    salvar.mutate(valor.trim());
  }

  return (
    <>
      <div className="cartao">
        <h2>Atalho global</h2>
        <p className="dica">
          Funciona em qualquer programa. Acione para começar a gravar e de novo para parar.
        </p>
        <form onSubmit={enviar} noValidate>
          <div className="campo">
            <label htmlFor="atalho-global">Atalho</label>
            <input
              id="atalho-global"
              type="text"
              value={valor}
              autoComplete="off"
              spellCheck={false}
              aria-invalid={erro ? "true" : undefined}
              aria-describedby="atalho-erro atalho-dica"
              onChange={(e) => {
                setValor(e.target.value);
                setSalvo(false);
              }}
            />
            <span className="erro-campo" id="atalho-erro" role={erro ? "alert" : undefined}>
              {erro}
            </span>
            <span className="dica" id="atalho-dica">
              Use pelo menos uma tecla modificadora (Ctrl, Alt, Shift ou ⌘/Win). Atalhos do sistema
              são recusados.
            </span>
          </div>
          <button type="submit" className="btn primario" disabled={salvar.isPending}>
            Salvar atalho
          </button>
          <p className="salvo" role={salvo ? "status" : undefined}>
            {salvo ? "Atalho salvo. Já vale, sem reiniciar o app." : ""}
          </p>
        </form>
      </div>
      <div className="cartao">
        <h2>Outros atalhos</h2>
        <div className="linha-opcao">
          <div className="texto">
            <b>Cancelar o ditado</b>
            <span className="dica">
              Aperte duas vezes em até 1,5 s enquanto grava. Uma vez só passa para o programa em
              foco.
            </span>
          </div>
          <span>
            <kbd>Esc</kbd> <kbd>Esc</kbd>
          </span>
        </div>
      </div>
    </>
  );
}

export default function Pagina() {
  return <PaginaAtalhos />;
}
