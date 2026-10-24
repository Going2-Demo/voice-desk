import { useParams } from "react-router";
import { PaginaProvisoria } from "../../../compartilhado/ui/pagina-provisoria";

export default function Pagina() {
  const { id } = useParams();
  return <PaginaProvisoria titulo={`Editor de modo (${id})`} area="modos" />;
}
