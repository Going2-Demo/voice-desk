import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useRef, useState } from "react";
import { consultas } from "../../ipc/consultas";
import { api, type Palavra } from "./api";
import { ErroVocabulario } from "./erro";
import "./vocabulario.css";

const mensagemDe = (e: unknown) =>
  e instanceof Error ? e.message : "Algo deu errado. Tente de novo.";

export default function Pagina() {
  const cliente = useQueryClient();
  const [busca, setBusca] = useState("");
  const [nova, setNova] = useState("");
  const [erroNova, setErroNova] = useState("");
  const [aviso, setAviso] = useState("");
  const [editando, setEditando] = useState<string | null>(null);
  const campoNova = useRef<HTMLInputElement>(null);

  const lista = useQuery({
    queryKey: [...consultas.vocabulario.lista, busca],
    queryFn: () => api.listar(busca.trim() || null),
    placeholderData: (anterior) => anterior,
  });
  const recarregar = () => cliente.invalidateQueries({ queryKey: consultas.vocabulario.lista });

  const adicionar = useMutation({
    mutationFn: (p: string) => api.adicionar(p),
    onSuccess: (p) => {
      setNova("");
      setErroNova("");
      setAviso(`“${p.palavra}” adicionada.`);
      recarregar();
      campoNova.current?.focus();
    },
    onError: (e) => setErroNova(mensagemDe(e)),
  });
  const remover = useMutation({
    mutationFn: (p: Palavra) => api.remover(p.id).then(() => p),
    onSuccess: (p) => {
      setAviso(`“${p.palavra}” removida.`);
      recarregar();
    },
    onError: (e) => setAviso(mensagemDe(e)),
  });

  const enviar = (e: FormEvent) => {
    e.preventDefault();
    if (!nova.trim()) {
      setErroNova("Digite uma palavra ou nome.");
      return;
    }
    adicionar.mutate(nova);
  };

  const dados = lista.data;
  const vazio = dados?.total === 0;

  return (
    <section className="vocabulario">
      <div className="cabecalho">
        <h1>Vocabulário</h1>
        {dados && (
          <span className="selo">
            {dados.total} de {dados.limite} palavras
          </span>
        )}
      </div>
      <p className="suave">
        Nomes, siglas e termos que a transcrição deve escrever certo. Vale para todos os ditados.
      </p>
      {lista.isPending && <p role="status">Carregando…</p>}
      {lista.isError && (
        <p role="alert">
          Não consegui ler o vocabulário.{" "}
          <button type="button" onClick={() => lista.refetch()}>
            Tentar de novo
          </button>
        </p>
      )}
      {dados && (
        <>
          <div className="cartao">
            <form onSubmit={enviar} noValidate>
              <div className="campo">
                <label htmlFor="v-nova">{vazio ? "Primeira palavra" : "Adicionar palavra"}</label>
                <input
                  id="v-nova"
                  ref={campoNova}
                  type="text"
                  autoComplete="off"
                  placeholder="Ex.: Oncoclínica Vale do Sol"
                  value={nova}
                  aria-invalid={erroNova ? true : undefined}
                  aria-describedby="v-erro"
                  onChange={(e) => {
                    setNova(e.target.value);
                    setErroNova("");
                  }}
                />
                <span className="erro-campo" id="v-erro" role="alert">
                  {erroNova}
                </span>
              </div>
              <button type="submit" className="primario" disabled={adicionar.isPending}>
                Adicionar
              </button>
            </form>
          </div>
          {vazio ? (
            <div className="vazio">
              <h2>Seu vocabulário está vazio</h2>
              <p className="suave">
                Cadastre nomes de clientes, siglas e termos técnicos para a transcrição acertar de
                primeira.
              </p>
            </div>
          ) : (
            <>
              {dados.na_dica < dados.total && (
                <p className="dica" role="status">
                  Só as {dados.na_dica} palavras mais recentes vão como dica a cada ditado; o
                  provedor aceita uma dica curta.
                </p>
              )}
              <div className="campo">
                <label htmlFor="v-busca">Buscar no vocabulário</label>
                <input
                  id="v-busca"
                  type="search"
                  placeholder="Digite para filtrar"
                  value={busca}
                  onChange={(e) => setBusca(e.target.value)}
                />
              </div>
              {dados.palavras.length === 0 ? (
                <p className="suave" role="status">
                  Nenhuma palavra encontrada para “{busca}”.{" "}
                  <button type="button" onClick={() => setBusca("")}>
                    Limpar busca
                  </button>
                </p>
              ) : (
                <ul className="lista-chips" aria-label="Palavras do vocabulário">
                  {dados.palavras.map((p) =>
                    editando === p.id ? (
                      <li key={p.id}>
                        <EditarPalavra
                          palavra={p}
                          aoFechar={(salvou) => {
                            setEditando(null);
                            if (salvou) recarregar();
                          }}
                        />
                      </li>
                    ) : (
                      <li key={p.id} className="chip">
                        <span>{p.palavra}</span>
                        <button
                          type="button"
                          aria-label={`Editar ${p.palavra}`}
                          onClick={() => setEditando(p.id)}
                        >
                          ✎
                        </button>
                        <button
                          type="button"
                          aria-label={`Remover ${p.palavra}`}
                          onClick={() => remover.mutate(p)}
                        >
                          ×
                        </button>
                      </li>
                    ),
                  )}
                </ul>
              )}
            </>
          )}
          <p className="so-leitor" role="status" aria-live="polite">
            {aviso}
          </p>
        </>
      )}
    </section>
  );
}

function EditarPalavra({
  palavra,
  aoFechar,
}: {
  palavra: Palavra;
  aoFechar: (salvou: boolean) => void;
}) {
  const [valor, setValor] = useState(palavra.palavra);
  const [erro, setErro] = useState("");
  const salvar = useMutation({
    mutationFn: () => api.editar(palavra.id, valor),
    onSuccess: () => aoFechar(true),
    onError: (e) => setErro(e instanceof ErroVocabulario ? e.message : mensagemDe(e)),
  });
  return (
    <form
      className="editar"
      onSubmit={(e) => {
        e.preventDefault();
        salvar.mutate();
      }}
      noValidate
    >
      <label className="so-leitor" htmlFor={`ve-${palavra.id}`}>
        Palavra
      </label>
      <input
        id={`ve-${palavra.id}`}
        type="text"
        value={valor}
        aria-invalid={erro ? true : undefined}
        aria-describedby={`ve-erro-${palavra.id}`}
        // biome-ignore lint/a11y/noAutofocus: o campo nasce quando o usuário pede para editar
        autoFocus
        onChange={(e) => {
          setValor(e.target.value);
          setErro("");
        }}
        onKeyDown={(e) => e.key === "Escape" && aoFechar(false)}
      />
      <button type="submit" className="primario" disabled={salvar.isPending}>
        Salvar
      </button>
      <button type="button" onClick={() => aoFechar(false)}>
        Cancelar
      </button>
      <span className="erro-campo" id={`ve-erro-${palavra.id}`} role="alert">
        {erro}
      </span>
    </form>
  );
}
