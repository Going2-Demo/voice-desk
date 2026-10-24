import { useMutation, useQuery } from "@tanstack/react-query";
import { consultas } from "../../ipc/consultas";
import { api } from "./api";
import "./sobre.css";

export default function Pagina() {
  const consulta = useQuery({ queryKey: consultas.app.sobre, queryFn: api.obter });
  const abrir = useMutation({ mutationFn: api.abrirProjeto });

  return (
    <section className="sobre">
      <h1>Sobre o Voice Desk</h1>
      {consulta.isError && <p role="alert">Não foi possível ler a versão instalada.</p>}
      {consulta.data && (
        <div className="cartao">
          <dl className="meta">
            <div>
              <dt>Versão instalada</dt>
              <dd>{consulta.data.versao}</dd>
            </div>
            <div>
              <dt>Licença</dt>
              <dd>{consulta.data.licenca}</dd>
            </div>
          </dl>
          <p>
            <button type="button" className="link" onClick={() => abrir.mutate()}>
              Página do projeto: instaladores, código-fonte e licença ↗
            </button>
          </p>
          {abrir.isError && (
            <p role="alert">Não foi possível abrir a página do projeto. Tente de novo.</p>
          )}
          <p className="aviso info">
            O Voice Desk é um projeto independente e <b>não tem relação com o VoiceInk</b>.
          </p>
        </div>
      )}
    </section>
  );
}
