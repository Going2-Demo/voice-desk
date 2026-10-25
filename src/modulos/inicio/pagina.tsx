import { useMutation, useQuery } from "@tanstack/react-query";
import { Link } from "react-router";
import { consultas } from "../../ipc/consultas";
import { type PonteDitado, ponteReal } from "../../ipc/ponte-ditado";
import "./inicio.css";

export function PaginaInicio({ ponte = ponteReal }: { ponte?: PonteDitado }) {
  const atalho = useQuery({ queryKey: consultas.atalhos.global, queryFn: ponte.obterAtalho });
  const testar = useMutation({ mutationFn: ponte.alternarDitado });

  return (
    <section className="inicio">
      <h1>Início</h1>
      <div className="cartao">
        <h2>Como ditar</h2>
        <ol>
          <li>Clique no campo de texto de qualquer programa.</li>
          <li>
            Pressione {atalho.data ? <kbd>{atalho.data.exibicao}</kbd> : "o atalho de ditado"} e
            fale.
          </li>
          <li>Pressione de novo para terminar: o texto é colado onde o cursor está.</li>
        </ol>
        {atalho.isError && (
          <p role="alert" className="erro-campo">
            Não foi possível ler o atalho atual. Feche e abra o Voice Desk.
          </p>
        )}
        <p className="dica">
          O Voice Desk fica no ícone da bandeja; fechar esta janela não encerra o app.
        </p>
        <p>
          <button type="button" onClick={() => testar.mutate()}>
            Testar ditado
          </button>
        </p>
        {testar.isError && (
          <p role="alert" className="erro-campo">
            Não foi possível iniciar o teste. Tente de novo.
          </p>
        )}
      </div>
      <div className="cartao">
        <h2>Primeiros passos</h2>
        <ul>
          <li>
            <Link to="/primeiro-uso">Guia de primeiro uso</Link>: permissões, atalho e provedor
          </li>
          <li>
            <Link to="/config/provedores">Provedores e chaves</Link>
          </li>
          <li>
            <Link to="/config/atalhos">Atalhos</Link>
          </li>
        </ul>
      </div>
    </section>
  );
}

export default function Pagina() {
  return <PaginaInicio />;
}
