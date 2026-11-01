import { PaginaProvisoria } from "../../../compartilhado/ui/pagina-provisoria";
import { Secao as IniciarComOSistema } from "./secao-iniciar-com-o-sistema";
import { Secao as Microfone } from "./secao-microfone";
import { Secao as Midia } from "./secao-midia";
import { Secao as SemFala } from "./secao-sem-fala";
import { Secao as Sons } from "./secao-sons";
import { Secao as TextoDoDitado } from "./secao-texto-do-ditado";

export default function Pagina() {
  return (
    <>
      <PaginaProvisoria titulo="Configurações · Geral" area="preferencias-gerais" />
      <IniciarComOSistema />
      <Microfone />
      <TextoDoDitado />
      <Sons />
      <Midia />
      <SemFala />
    </>
  );
}
