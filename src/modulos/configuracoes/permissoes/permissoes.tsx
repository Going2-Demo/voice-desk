import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { SituacaoApp } from "../../../ipc/bindings";
import { consultas } from "../../../ipc/consultas";
import { type ApiPermissoes, api as apiReal, type Permissao } from "./api";
import "./permissoes.css";

const SELO: Record<SituacaoApp, { icone: string; texto: string; classe: string }> = {
  concedida: { icone: "✓", texto: "Concedida", classe: "ok" },
  negada: { icone: "✕", texto: "Negada", classe: "erro" },
  pendente: { icone: "○", texto: "Pendente", classe: "alerta" },
  nao_se_aplica: { icone: "–", texto: "Não necessária", classe: "" },
};

type Descricao = {
  permissao: Permissao;
  titulo: string;
  para: string;
  /** O que deixa de funcionar e como conceder, mostrado quando está negada ou pendente. */
  semela: string;
  onde: string;
};

function descricoes(mac: boolean): Descricao[] {
  return [
    {
      permissao: "microfone",
      titulo: "Microfone",
      para: "Para gravar a sua voz.",
      semela: "Sem o microfone o ditado não grava nada.",
      onde: mac
        ? "Abra Ajustes do Sistema › Privacidade e Segurança › Microfone e ligue o Voice Desk."
        : "Abra Configurações do Windows › Privacidade e segurança › Microfone e ligue o acesso de aplicativos de desktop.",
    },
    ...(mac
      ? [
          {
            permissao: "acessibilidade" as const,
            titulo: "Acessibilidade",
            para: "Para colar o texto no programa em foco e cancelar o ditado com Esc duplo.",
            semela:
              "Sem a Acessibilidade o texto não é colado sozinho (use Cmd+V) e o Esc duplo não cancela.",
            onde: "Abra Ajustes do Sistema › Privacidade e Segurança › Acessibilidade e ligue o Voice Desk.",
          },
        ]
      : []),
  ];
}

type Props = { api?: ApiPermissoes };

/** Lista de permissões com o estado de cada uma; usada no primeiro uso e em Configurações › Permissões. */
export function ListaPermissoes({ api = apiReal }: Props) {
  const cliente = useQueryClient();
  const consulta = useQuery({
    queryKey: consultas.permissoes.estado,
    queryFn: api.estado,
    // O usuário concede na tela do sistema e volta: acompanha sem pedir para recarregar.
    refetchInterval: 3000,
  });
  const atualizar = (novo: Awaited<ReturnType<ApiPermissoes["estado"]>>) =>
    cliente.setQueryData(consultas.permissoes.estado, novo);
  const pedir = useMutation({ mutationFn: (p: Permissao) => api.pedir(p), onSuccess: atualizar });
  const abrir = useMutation({ mutationFn: (p: Permissao) => api.abrirConfiguracoes(p) });

  if (consulta.isPending) return <p role="status">Carregando…</p>;
  if (consulta.isError || !consulta.data) {
    return <p role="alert">Não consegui ler as permissões. Feche e abra o Voice Desk.</p>;
  }
  const estado = consulta.data;
  const mac = estado.plataforma === "macos";

  return (
    <div className="permissoes">
      <ul className="lista-permissoes">
        {descricoes(mac).map((d) => {
          const situacao = estado[d.permissao];
          const selo = SELO[situacao];
          const faltando = situacao === "negada" || situacao === "pendente";
          return (
            <li key={d.permissao} className="permissao">
              <div className="texto">
                <b>{d.titulo}</b>
                <span className="dica">{d.para}</span>
                {faltando && (
                  <span className="dica aviso-permissao" role="note">
                    {d.semela} {d.onde}
                  </span>
                )}
              </div>
              <span
                className={`selo ${selo.classe}`}
                role="status"
                aria-label={`${d.titulo}: ${selo.texto}`}
              >
                <span aria-hidden="true">{selo.icone}</span> {selo.texto}
              </span>
              <div className="acoes-permissao">
                {faltando && (
                  <button
                    type="button"
                    className="btn pequeno primario"
                    disabled={pedir.isPending}
                    onClick={() => pedir.mutate(d.permissao)}
                  >
                    Conceder {d.titulo.toLowerCase()}
                  </button>
                )}
                {situacao === "negada" && (
                  <button
                    type="button"
                    className="btn pequeno"
                    onClick={() => abrir.mutate(d.permissao)}
                  >
                    Abrir configurações do sistema
                  </button>
                )}
              </div>
            </li>
          );
        })}
      </ul>
      {!mac && (
        <p className="dica">
          Para colar em outros programas o Windows não exige permissão; ele pode pedir confirmação
          na primeira colagem.
        </p>
      )}
      {(pedir.isError || abrir.isError) && (
        <p role="alert" className="erro-campo">
          {((pedir.error ?? abrir.error) as Error).message}
        </p>
      )}
    </div>
  );
}
