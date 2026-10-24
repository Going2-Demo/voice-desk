import { Secao as IniciarComOSistema } from "../configuracoes/geral/secao-iniciar-com-o-sistema";

const estados = [
  ["Parado", "O Voice Desk está pronto para ditar."],
  ["Gravando", "O microfone está captando; o ícone ganha um ponto vermelho."],
  ["Processando", "A fala está sendo transcrita e refinada."],
  ["Erro", "Algo falhou no último ditado; abra o menu ou o histórico para ver."],
] as const;

const itens = [
  "Iniciar ou parar o ditado",
  "Escolher o modo padrão",
  "Abrir o histórico",
  "Abrir as configurações",
  "Sair do Voice Desk",
];

export default function Pagina() {
  return (
    <section>
      <h1>Ícone e menu</h1>
      <p>
        O Voice Desk vive no ícone da barra de menus (Mac) ou da bandeja (Windows). O ícone mostra o
        estado e o menu dá acesso rápido sem abrir a janela.
      </p>
      <h2>Menu do ícone</h2>
      <ul>
        {itens.map((i) => (
          <li key={i}>{i}</li>
        ))}
      </ul>
      <h2>Estados do ícone</h2>
      <dl>
        {estados.map(([nome, texto]) => (
          <div key={nome}>
            <dt>{nome}</dt>
            <dd>{texto}</dd>
          </div>
        ))}
      </dl>
      <IniciarComOSistema />
    </section>
  );
}
