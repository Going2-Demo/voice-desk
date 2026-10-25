import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useRef, useState } from "react";
import { Link, useNavigate } from "react-router";
import { consultas } from "../../ipc/consultas";
import { type PonteDitado, ponteReal } from "../../ipc/ponte-ditado";
import { ListaPermissoes } from "../configuracoes/permissoes/permissoes";
import { type ApiChaves, api as chavesReal } from "../configuracoes/provedores/api";
import { type ApiPrimeiroUso, api as apiReal, type Passo } from "./api";
import "./primeiro-uso.css";

const PASSOS: { titulo: string; passo?: Passo }[] = [
  { titulo: "Boas-vindas" },
  { titulo: "Permissões", passo: "permissoes" },
  { titulo: "Atalho", passo: "atalho" },
  { titulo: "Provedor", passo: "provedor" },
  { titulo: "Sincronização", passo: "sincronizacao" },
  { titulo: "Teste" },
];
const ULTIMO = PASSOS.length - 1;
const CHAVE_POSICAO = "primeiro-uso.passo";

type Props = { api?: ApiPrimeiroUso; ponte?: PonteDitado; chaves?: ApiChaves };

/** Lembra o passo ao sair para outra tela (ligar a sincronização, por exemplo) e voltar. */
function lerPosicao(): number {
  const n = Number(sessionStorage.getItem(CHAVE_POSICAO));
  return Number.isInteger(n) && n >= 0 && n <= ULTIMO ? n : 0;
}

export default function Pagina(props: Props) {
  const { api = apiReal, ponte = ponteReal, chaves = chavesReal } = props;
  const cliente = useQueryClient();
  const navegar = useNavigate();
  const [posicao, setPosicao] = useState(lerPosicao);

  const estado = useQuery({ queryKey: consultas.primeiroUso.estado, queryFn: api.estado });
  const pulados = estado.data?.pulados ?? [];

  const guardar = (novo: Awaited<ReturnType<ApiPrimeiroUso["estado"]>>) =>
    cliente.setQueryData(consultas.primeiroUso.estado, novo);
  const pular = useMutation({ mutationFn: (p: Passo) => api.pular(p), onSuccess: guardar });
  const cumprir = useMutation({ mutationFn: (p: Passo) => api.cumprir(p), onSuccess: guardar });
  const concluir = useMutation({
    mutationFn: () => api.concluir(),
    onSuccess: (novo) => {
      guardar(novo);
      sessionStorage.removeItem(CHAVE_POSICAO);
      navegar("/inicio");
    },
  });

  const ir = (n: number) => {
    const alvo = Math.min(ULTIMO, Math.max(0, n));
    sessionStorage.setItem(CHAVE_POSICAO, String(alvo));
    setPosicao(alvo);
  };

  const atual = PASSOS[posicao];
  const pularAtual = () => {
    if (atual.passo) pular.mutate(atual.passo);
    ir(posicao + 1);
  };

  return (
    <section className="primeiro-uso">
      <h1>Primeiro uso</h1>
      <p className="dica">
        Em até 10 minutos você faz o primeiro ditado. Qualquer passo pode ser pulado e retomado
        depois em Configurações.
      </p>
      <ol className="trilha" aria-label="Passos">
        {PASSOS.map((p, i) => (
          <li
            key={p.titulo}
            aria-current={i === posicao ? "step" : undefined}
            className={i < posicao ? "feito" : undefined}
          >
            {i < posicao ? (p.passo && pulados.includes(p.passo) ? "↷ " : "✓ ") : `${i + 1}. `}
            {p.titulo}
            {p.passo && pulados.includes(p.passo) ? (
              <span className="so-leitor"> (pulado)</span>
            ) : null}
          </li>
        ))}
      </ol>

      <div className="cartao">
        {/* Ao trocar de passo o título é remontado e recebe o foco. */}
        <h2 key={posicao} tabIndex={-1} ref={(el) => el?.focus()}>
          {TITULOS[posicao]}
        </h2>
        {posicao === 0 && <BoasVindas />}
        {posicao === 1 && <ListaPermissoes />}
        {posicao === 2 && <PassoAtalho ponte={ponte} aoSalvar={() => cumprir.mutate("atalho")} />}
        {posicao === 3 && (
          <PassoProvedor
            chaves={chaves}
            aoSalvar={() => cumprir.mutate("provedor")}
            aoUsarLocal={pularAtual}
          />
        )}
        {posicao === 4 && <PassoSincronizacao />}
        {posicao === 5 && <PassoTeste api={api} ponte={ponte} />}
      </div>

      {concluir.isError && (
        <p role="alert" className="erro-campo">
          {(concluir.error as Error).message}
        </p>
      )}
      <div className="rodape-passos">
        <div>
          {posicao > 0 && (
            <button type="button" className="btn" onClick={() => ir(posicao - 1)}>
              Voltar
            </button>
          )}
        </div>
        <div className="acoes">
          {atual.passo && (
            <button type="button" className="btn discreto" onClick={pularAtual}>
              Pular este passo
            </button>
          )}
          {posicao < ULTIMO ? (
            <button type="button" className="btn primario" onClick={() => ir(posicao + 1)}>
              Continuar
            </button>
          ) : (
            <button
              type="button"
              className="btn primario"
              disabled={concluir.isPending}
              onClick={() => concluir.mutate()}
            >
              Concluir
            </button>
          )}
        </div>
      </div>
    </section>
  );
}

const TITULOS = [
  "Bem-vindo ao Voice Desk",
  "Permissões",
  "Escolha o atalho",
  "Como transcrever?",
  "Sincronização (opcional)",
  "Faça um ditado de teste",
];

function BoasVindas() {
  return (
    <>
      <p>
        Aperte um atalho em qualquer programa, fale e solte. O texto aparece onde o cursor estava.
        Gratuito, de código aberto, e a sua configuração e as suas chaves ficam com você.
      </p>
      <p className="aviso info">
        <span aria-hidden="true">🔒</span> Seu áudio e seus textos ficam nesta máquina. Só vão ao
        provedor que você escolher em cada modo.
      </p>
    </>
  );
}

function PassoAtalho({ ponte, aoSalvar }: { ponte: PonteDitado; aoSalvar: () => void }) {
  const atual = useQuery({ queryKey: consultas.atalhos.global, queryFn: ponte.obterAtalho });
  const cliente = useQueryClient();
  const [valor, setValor] = useState<string | null>(null);
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
        aoSalvar();
      } else {
        setSalvo(false);
        setErro(r.mensagem);
      }
    },
  });

  if (atual.isError) {
    return (
      <p role="alert">
        Não consegui ler o atalho atual. Você pode ajustar depois em Configurações › Atalhos.
      </p>
    );
  }
  if (!atual.data) return <p role="status">Carregando…</p>;
  const campo = valor ?? atual.data.exibicao;
  return (
    <form
      noValidate
      onSubmit={(e) => {
        e.preventDefault();
        setSalvo(false);
        if (!campo.trim()) {
          setErro("Digite uma combinação, por exemplo Ctrl+Alt+Espaço.");
          return;
        }
        salvar.mutate(campo.trim());
      }}
    >
      <p className="dica">Combinação que você aciona em qualquer programa.</p>
      <div className="campo">
        <label htmlFor="pu-atalho">Atalho global</label>
        <input
          id="pu-atalho"
          type="text"
          value={campo}
          autoComplete="off"
          spellCheck={false}
          aria-invalid={erro ? "true" : undefined}
          aria-describedby="pu-atalho-erro"
          onChange={(e) => {
            setValor(e.target.value);
            setSalvo(false);
          }}
        />
        <span className="erro-campo" id="pu-atalho-erro" role={erro ? "alert" : undefined}>
          {erro}
        </span>
      </div>
      <button type="submit" className="btn" disabled={salvar.isPending}>
        Salvar atalho
      </button>
      <p className="dica" role={salvo ? "status" : undefined}>
        {salvo
          ? "Atalho salvo. Já vale, sem reiniciar o app."
          : "Você pode mudar quando quiser em Configurações › Atalhos."}
      </p>
    </form>
  );
}

function PassoProvedor({
  chaves,
  aoSalvar,
  aoUsarLocal,
}: {
  chaves: ApiChaves;
  aoSalvar: () => void;
  aoUsarLocal: () => void;
}) {
  const [chave, setChave] = useState("");
  const [erro, setErro] = useState("");
  const [feito, setFeito] = useState<string | null>(null);
  const salvar = useMutation({
    mutationFn: (valor: string) => chaves.salvar("groq", valor),
    onSuccess: (e) => {
      setChave("");
      setErro("");
      setFeito(
        e.validade === "invalida"
          ? "A chave foi guardada, mas o Groq a recusou. Confira se copiou inteira."
          : "Chave guardada no cofre do sistema.",
      );
      if (e.validade !== "invalida") aoSalvar();
    },
    onError: (e: Error) => {
      setFeito(null);
      setErro(e.message);
    },
  });
  return (
    <>
      <div className="subcartao">
        <h3>☁️ Groq (nuvem) — rápido e preciso</h3>
        <p className="dica">
          Precisa de uma chave de API gratuita ou paga, criada por você no site do Groq.
        </p>
        <form
          noValidate
          onSubmit={(e) => {
            e.preventDefault();
            setFeito(null);
            if (!chave.trim()) {
              setErro("Cole a chave de API do Groq.");
              return;
            }
            salvar.mutate(chave);
          }}
        >
          <div className="campo">
            <label htmlFor="pu-chave">Chave de API do Groq</label>
            <input
              id="pu-chave"
              type="password"
              value={chave}
              placeholder="gsk_…"
              autoComplete="off"
              spellCheck={false}
              aria-invalid={erro ? "true" : undefined}
              aria-describedby="pu-chave-erro"
              onChange={(e) => setChave(e.target.value)}
            />
            <span className="erro-campo" id="pu-chave-erro" role={erro ? "alert" : undefined}>
              {erro}
            </span>
          </div>
          <div className="acoes">
            <button type="submit" className="btn" disabled={salvar.isPending}>
              {salvar.isPending ? "Testando…" : "Salvar e testar chave"}
            </button>
            <button type="button" className="link" onClick={() => void chaves.abrirPagina("groq")}>
              Como obter uma chave ↗
            </button>
          </div>
          <p className="dica" role={feito ? "status" : undefined}>
            {feito}
          </p>
        </form>
      </div>
      <div className="subcartao">
        <h3>🔒 Só local — sem chave e sem internet</h3>
        <p className="dica">
          Baixa o modelo no seu computador e transcreve sem enviar nada. O download fica em{" "}
          <Link to="/config/local">Configurações › Modelo local</Link>; dá para trocar depois.
        </p>
        <button type="button" className="btn" onClick={aoUsarLocal}>
          Usar transcrição local
        </button>
      </div>
    </>
  );
}

function PassoSincronizacao() {
  return (
    <>
      <p className="dica">
        Tem outra máquina? Ligue para ter os mesmos modos e vocabulário nas duas. Ou importe o
        arquivo de configuração de alguém.
      </p>
      <div className="acoes">
        <Link className="btn" to="/sincronizacao">
          Ligar a sincronização
        </Link>
        <Link className="btn" to="/sincronizacao">
          Importar arquivo de configuração…
        </Link>
      </div>
    </>
  );
}

function PassoTeste({ api, ponte }: { api: ApiPrimeiroUso; ponte: PonteDitado }) {
  const caixa = useRef<HTMLTextAreaElement>(null);
  const [texto, setTexto] = useState("");
  const [motivo, setMotivo] = useState("");
  const [atalho, setAtalho] = useState("");

  useEffect(() => {
    ponte.obterAtalho().then(
      (a) => setAtalho(a.exibicao),
      () => undefined,
    );
    const fim = ponte.aoEstado((e) => {
      if (e.fase === "erro" || e.fase === "aviso") setMotivo(e.mensagem);
      else if (e.fase === "gravando") setMotivo("");
    });
    return () => {
      fim.then((desligar) => desligar());
    };
  }, [ponte]);

  return (
    <>
      <p className="dica">
        Clique na caixa, acione o atalho
        {atalho ? (
          <>
            {" "}
            <kbd>{atalho}</kbd>
          </>
        ) : null}{" "}
        e fale uma frase.
      </p>
      <div className="campo">
        <label htmlFor="pu-teste">Caixa de teste</label>
        <textarea
          id="pu-teste"
          ref={caixa}
          value={texto}
          placeholder="O texto vai aparecer aqui"
          onChange={(e) => setTexto(e.target.value)}
        />
      </div>
      {motivo && (
        <p role="alert" className="erro-campo">
          Não deu certo: {motivo}
        </p>
      )}
      <div className="acoes">
        <button
          type="button"
          className="btn"
          onClick={() => {
            setMotivo("");
            // O texto é colado onde estava o foco: devolve o foco à caixa antes de gravar.
            caixa.current?.focus();
            void ponte.alternarDitado();
          }}
        >
          Iniciar ou parar o ditado de teste
        </button>
        {api.simularDitado && (
          <button
            type="button"
            className="btn"
            onClick={() => setTexto(api.simularDitado?.() ?? "")}
          >
            Simular ditado de teste
          </button>
        )}
      </div>
    </>
  );
}
