import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type FormEvent, useRef, useState } from "react";
import { consultas } from "../../ipc/consultas";
import { api, type PalavraPreenchimento } from "./api";
import "../../compartilhado/tema/componentes.css";
import "../../compartilhado/tema/formulario.css";
import "./limpeza.css";

const TEXTO_DE_TESTE = "é… então assim, tipo, preciso falar com o Pereira né sobre o contrato";
const mensagemDe = (e: unknown) =>
  e instanceof Error ? e.message : "Algo deu errado. Tente de novo.";

export default function Pagina() {
  const cliente = useQueryClient();
  const [nova, setNova] = useState("");
  const [erroNova, setErroNova] = useState("");
  const [aviso, setAviso] = useState("");
  const [teste, setTeste] = useState(TEXTO_DE_TESTE);
  const campoNova = useRef<HTMLInputElement>(null);

  const lista = useQuery({
    queryKey: consultas.limpeza.lista,
    queryFn: () => api.listar(),
  });
  const opcoes = useQuery({
    queryKey: consultas.limpeza.opcoes,
    queryFn: () => api.opcoesObter(),
  });
  const previa = useQuery({
    queryKey: [...consultas.limpeza.previa, teste, lista.data?.palavras, opcoes.data],
    queryFn: () => api.previa(teste),
    enabled: lista.isSuccess && opcoes.isSuccess,
    placeholderData: (anterior) => anterior,
  });
  const recarregar = () => cliente.invalidateQueries({ queryKey: consultas.limpeza.lista });

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
    mutationFn: (p: PalavraPreenchimento) => api.remover(p.id).then(() => p),
    onSuccess: (p) => {
      setErroNova("");
      setAviso(`“${p.palavra}” removida.`);
      recarregar();
    },
    onError: (e) => setAviso(mensagemDe(e)),
  });

  const enviar = (e: FormEvent) => {
    e.preventDefault();
    if (!nova.trim()) {
      setErroNova("Digite uma palavra ou expressão.");
      return;
    }
    adicionar.mutate(nova);
  };

  const dados = lista.data;
  const vazio = dados?.palavras.length === 0;
  const tudoDesligado = opcoes.data && !opcoes.data.remover_preenchimento && !opcoes.data.formatar;

  const formulario = (
    <form onSubmit={enviar} noValidate>
      <div className="campo">
        <label htmlFor="f-nova">Nova palavra</label>
        <input
          id="f-nova"
          ref={campoNova}
          type="text"
          autoComplete="off"
          placeholder="Ex.: basicamente"
          value={nova}
          aria-invalid={erroNova ? true : undefined}
          aria-describedby="f-erro"
          onChange={(e) => {
            setNova(e.target.value);
            setErroNova("");
          }}
        />
        <span className="erro-campo" id="f-erro" role="alert">
          {erroNova}
        </span>
      </div>
      <button type="submit" className="btn primario" disabled={adicionar.isPending}>
        Adicionar
      </button>
    </form>
  );

  return (
    <section className="limpeza formulario-vd">
      <div className="cabecalho">
        <h1>Limpeza do texto</h1>
      </div>
      <p className="subtitulo">
        Etapa sem IA: tira as muletas da fala e arruma a pontuação. Em Configurações › Geral você
        liga ou desliga a remoção e a formatação.
      </p>
      {(lista.isPending || opcoes.isPending) && <p role="status">Carregando…</p>}
      {(lista.isError || opcoes.isError) && (
        <p role="alert">
          Não consegui ler a lista.{" "}
          <button
            type="button"
            className="btn"
            onClick={() => {
              lista.refetch();
              opcoes.refetch();
            }}
          >
            Tentar de novo
          </button>
        </p>
      )}
      {dados && (
        <>
          {vazio ? (
            <div className="vazio">
              <div className="grande" aria-hidden="true">
                🧹
              </div>
              <h2>Lista vazia</h2>
              <p>Sem palavras de preenchimento, a limpeza só arruma a pontuação.</p>
              {formulario}
            </div>
          ) : (
            <div className="cartao">
              <h2>Palavras de preenchimento</h2>
              <p className="dica">
                Em português e inglês. Edite a lista como quiser. Uma entrada com “…” (como “é…”) só
                sai quando a transcrição traz as reticências.
              </p>
              <ul className="lista-chips" aria-label="Palavras de preenchimento">
                {dados.palavras.map((p) => (
                  <li key={p.id} className="chip">
                    <span>{p.palavra}</span>
                    <button
                      type="button"
                      aria-label={`Remover ${p.palavra}`}
                      onClick={() => remover.mutate(p)}
                    >
                      ×
                    </button>
                  </li>
                ))}
              </ul>
              {formulario}
            </div>
          )}
          <div className="cartao">
            <h2>Teste a limpeza</h2>
            <p className="dica">Digite uma frase como se tivesse falado e veja o resultado.</p>
            {tudoDesligado && (
              <p className="dica" role="status">
                A remoção e a formatação estão desligadas em Configurações › Geral: o texto sai sem
                alteração.
              </p>
            )}
            <div className="campo">
              <label htmlFor="f-teste">Texto falado</label>
              <textarea
                id="f-teste"
                value={teste}
                maxLength={2000}
                onChange={(e) => setTeste(e.target.value)}
              />
            </div>
            <span className="rotulo" id="f-saida-rotulo">
              Depois da limpeza
            </span>
            <output className="texto-bloco" aria-labelledby="f-saida-rotulo" aria-live="polite">
              {previa.data ?? ""}
            </output>
          </div>
          <p className="so-leitor" role="status" aria-live="polite">
            {aviso}
          </p>
        </>
      )}
    </section>
  );
}
