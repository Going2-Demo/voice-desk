import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useId } from "react";
import { consultas } from "../../../ipc/consultas";
import "../../../compartilhado/tema/componentes.css";
import { Interruptor } from "../../../compartilhado/ui/interruptor";
import { api, type OpcoesTexto } from "../../limpeza/api";

const IDIOMAS: { valor: OpcoesTexto["idioma"]; rotulo: string }[] = [
  { valor: "pt-BR", rotulo: "Português do Brasil" },
  { valor: "en", rotulo: "Inglês" },
  { valor: "auto", rotulo: "Detectar automaticamente" },
];

/** Bloco "Texto do ditado" (RF-09.1, RF-02.2, RF-03.3): vale para todos os ditados. */
export function Secao() {
  const id = useId();
  const cliente = useQueryClient();
  const opcoes = useQuery({ queryKey: consultas.limpeza.opcoes, queryFn: () => api.opcoesObter() });
  const salvar = useMutation({
    mutationFn: (novas: OpcoesTexto) => api.opcoesSalvar(novas),
    onSuccess: (salvas) => cliente.setQueryData(consultas.limpeza.opcoes, salvas),
  });
  const atual = salvar.variables && salvar.isPending ? salvar.variables : opcoes.data;
  const mudar = (parte: Partial<OpcoesTexto>) => {
    if (opcoes.data) salvar.mutate({ ...opcoes.data, ...parte });
  };

  return (
    <section className="cartao" aria-labelledby={`${id}-titulo`}>
      <h2 id={`${id}-titulo`}>Texto do ditado</h2>
      <p className="dica">Vale para todos os ditados.</p>
      <div className="campo" style={{ maxWidth: 320 }}>
        <label htmlFor={`${id}-idioma`}>Idioma do ditado</label>
        <select
          id={`${id}-idioma`}
          value={atual?.idioma ?? "pt-BR"}
          disabled={!atual}
          onChange={(e) => mudar({ idioma: e.target.value as OpcoesTexto["idioma"] })}
        >
          {IDIOMAS.map((i) => (
            <option key={i.valor} value={i.valor}>
              {i.rotulo}
            </option>
          ))}
        </select>
      </div>
      <Opcao
        id={`${id}-limpeza`}
        titulo="Remover palavras de preenchimento"
        dica="Tira “é…”, “tipo”, “né” e parecidas. A lista é editável em Limpeza do texto."
        ligado={atual?.remover_preenchimento ?? false}
        desabilitado={!atual}
        aoMudar={(v) => mudar({ remover_preenchimento: v })}
      />
      <Opcao
        id={`${id}-formatar`}
        titulo="Formatar o texto"
        dica="Maiúscula no início, pontuação e espaços arrumados."
        ligado={atual?.formatar ?? false}
        desabilitado={!atual}
        aoMudar={(v) => mudar({ formatar: v })}
      />
      <Opcao
        id={`${id}-enter`}
        titulo="Apertar Enter depois de colar"
        dica="Desligado por padrão. Ligado, vale para todo programa, inclusive e-mail e documentos: ligue só se quiser enviar a mensagem na hora."
        ligado={atual?.enviar_enter ?? false}
        desabilitado={!atual}
        aoMudar={(v) => mudar({ enviar_enter: v })}
      />
      {(opcoes.isError || salvar.isError) && (
        <p role="alert" className="aviso-erro">
          Não foi possível salvar a preferência. Tente de novo.
        </p>
      )}
      <p className="so-leitor" role="status" aria-live="polite">
        {salvar.isSuccess ? "Preferência salva." : ""}
      </p>
    </section>
  );
}

function Opcao(props: {
  id: string;
  titulo: string;
  dica: string;
  ligado: boolean;
  desabilitado: boolean;
  aoMudar: (v: boolean) => void;
}) {
  return (
    <div className="linha-opcao">
      <div className="texto">
        <b id={`${props.id}-t`}>{props.titulo}</b>
        <span className="dica">{props.dica}</span>
      </div>
      <Interruptor
        id={props.id}
        ligado={props.ligado}
        desabilitado={props.desabilitado}
        aoMudar={props.aoMudar}
      />
    </div>
  );
}
