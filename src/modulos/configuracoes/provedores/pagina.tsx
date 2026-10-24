import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import type { Validade } from "../../../ipc/bindings";
import { consultas } from "../../../ipc/consultas";
import { api, type EstadoChave } from "./api";
import { ErroChave } from "./erro";
import "./provedores.css";

const SELO: Record<"sem_chave" | Validade, { icone: string; texto: string; classe: string }> = {
  sem_chave: { icone: "○", texto: "Sem chave", classe: "" },
  valida: { icone: "✓", texto: "Chave válida", classe: "ok" },
  invalida: { icone: "✕", texto: "Chave inválida", classe: "erro" },
  nao_testada: { icone: "?", texto: "Chave não testada", classe: "alerta" },
};

export default function Pagina() {
  const consulta = useQuery({ queryKey: consultas.chaves.estado, queryFn: api.estado });
  const groq = consulta.data?.find((e) => e.provedor === "groq");

  return (
    <section className="provedores">
      <h1>Provedores e chaves</h1>
      {consulta.isPending && <p role="status">Carregando…</p>}
      {consulta.isError && <p role="alert">Não consegui ler o estado das chaves. Tente de novo.</p>}
      {groq && <CartaoGroq estado={groq} />}
    </section>
  );
}

function CartaoGroq({ estado }: { estado: EstadoChave }) {
  const cliente = useQueryClient();
  const [trocando, setTrocando] = useState(false);
  const [confirmando, setConfirmando] = useState(false);
  const [chave, setChave] = useState("");
  const [erroCampo, setErroCampo] = useState("");
  const [erroGeral, setErroGeral] = useState("");

  const atualizar = (novo: EstadoChave) => {
    cliente.setQueryData<EstadoChave[]>(consultas.chaves.estado, (antigo) =>
      (antigo ?? []).map((e) => (e.provedor === novo.provedor ? novo : e)),
    );
  };
  const falhou = (e: unknown) => {
    if (e instanceof ErroChave && e.campo === "chave") setErroCampo(e.message);
    else setErroGeral(e instanceof Error ? e.message : "Algo deu errado. Tente de novo.");
  };

  const salvar = useMutation({
    mutationFn: (valor: string) => api.salvar("groq", valor),
    onSuccess: (novo) => {
      atualizar(novo);
      setChave("");
      setTrocando(false);
    },
    onError: falhou,
  });
  const testar = useMutation({
    mutationFn: () => api.testar("groq"),
    onSuccess: atualizar,
    onError: falhou,
  });
  const apagar = useMutation({
    mutationFn: () => api.apagar("groq"),
    onSuccess: (novo) => {
      atualizar(novo);
      setConfirmando(false);
      setTrocando(false);
    },
    onError: (e) => {
      setConfirmando(false);
      falhou(e);
    },
  });

  const selo = SELO[estado.salva ? (estado.validade ?? "nao_testada") : "sem_chave"];
  const mostraForm = !estado.salva || trocando;

  const enviar = (e: React.FormEvent) => {
    e.preventDefault();
    setErroCampo("");
    setErroGeral("");
    if (!chave.trim()) {
      setErroCampo("Cole a chave de API do Groq.");
      return;
    }
    salvar.mutate(chave);
  };

  return (
    <section className="cartao" aria-labelledby="titulo-groq">
      <h2 id="titulo-groq">
        Groq{" "}
        <span className={`selo ${selo.classe}`}>
          <span aria-hidden="true">{selo.icone} </span>
          {selo.texto}
        </span>
      </h2>
      <p className="dica">
        Transcrição e refinamento na nuvem. O custo de uso é seu, na sua conta. A chave fica no
        cofre do sistema; nunca em arquivo, na exportação nem na sincronização.
      </p>

      {estado.salva && (
        <div className="linha-chave">
          <div>
            <b className="mono" data-testid="mascara">
              {estado.mascara}
            </b>
            <span className="dica">Só os 4 últimos caracteres aparecem.</span>
          </div>
          <div className="acoes">
            <button type="button" className="btn" onClick={() => setTrocando((t) => !t)}>
              Trocar chave
            </button>
            <button type="button" className="btn perigo" onClick={() => setConfirmando(true)}>
              Apagar
            </button>
          </div>
        </div>
      )}

      {estado.salva && estado.validade === "invalida" && (
        <div className="aviso erro" role="alert">
          O Groq recusou esta chave{estado.motivo ? `: ${estado.motivo}` : "."} Confira se copiou
          inteira ou crie outra.
        </div>
      )}
      {estado.salva && estado.validade === "nao_testada" && (
        <div className="aviso alerta" role="alert">
          {estado.motivo ?? "Ainda não testei esta chave."}{" "}
          <button
            type="button"
            className="btn"
            onClick={() => testar.mutate()}
            disabled={testar.isPending}
          >
            {testar.isPending ? "Testando…" : "Testar de novo"}
          </button>
        </div>
      )}

      {mostraForm && (
        <form onSubmit={enviar} noValidate>
          <div className="campo">
            <label htmlFor="chave-groq">Chave de API</label>
            <input
              id="chave-groq"
              type="password"
              autoComplete="off"
              spellCheck={false}
              placeholder="gsk_…"
              value={chave}
              onChange={(e) => setChave(e.target.value)}
              aria-invalid={erroCampo ? true : undefined}
              aria-describedby="erro-chave-groq"
            />
            <span className="erro-campo" id="erro-chave-groq" role="alert">
              {erroCampo}
            </span>
          </div>
          <div className="acoes">
            <button type="submit" className="btn primario" disabled={salvar.isPending}>
              {salvar.isPending ? "Testando a chave no Groq…" : "Salvar e testar"}
            </button>
            {trocando && (
              <button
                type="button"
                className="btn"
                onClick={() => {
                  setTrocando(false);
                  setChave("");
                  setErroCampo("");
                }}
              >
                Cancelar
              </button>
            )}
          </div>
        </form>
      )}

      <p className="dica">
        <button type="button" className="link" onClick={() => api.abrirPagina("groq")}>
          Onde consigo uma chave? ↗
        </button>
      </p>

      {erroGeral && (
        <div className="aviso erro" role="alert">
          {erroGeral}
        </div>
      )}

      {confirmando && (
        <ConfirmarApagar
          carregando={apagar.isPending}
          aoConfirmar={() => apagar.mutate()}
          aoCancelar={() => setConfirmando(false)}
        />
      )}
    </section>
  );
}

function ConfirmarApagar(props: {
  carregando: boolean;
  aoConfirmar: () => void;
  aoCancelar: () => void;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const d = ref.current;
    if (d && !d.open) {
      if (typeof d.showModal === "function") d.showModal();
      else d.setAttribute("open", "");
    }
  }, []);
  return (
    <dialog ref={ref} aria-labelledby="titulo-apagar" onCancel={props.aoCancelar}>
      <h3 id="titulo-apagar">Apagar a chave do Groq?</h3>
      <p>
        Ela sai do cofre do sistema. Modos que usam o Groq vão falhar até você cadastrar outra ou
        trocar o provedor.
      </p>
      <div className="acoes">
        <button type="button" className="btn" onClick={props.aoCancelar}>
          Cancelar
        </button>
        <button
          type="button"
          className="btn perigo"
          onClick={props.aoConfirmar}
          disabled={props.carregando}
        >
          Apagar chave
        </button>
      </div>
    </dialog>
  );
}
