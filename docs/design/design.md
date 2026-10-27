# Design: Voice Desk

> **Escopo v3 (ENG-104, decisão do BD):** os Épicos 3 (modos e refinamento por IA), 4 (histórico), 5 (mesma configuração nas duas máquinas) e 6 (uso sem internet) estão **fora da v1**. As telas Modos, Editor de modo, Histórico, Sincronização, Modelo local e Dados e retenção saíram do protótipo (a rota mostra "Fora da v1") e as seções deste documento que as descrevem valem só como referência histórica, marcadas com ~~fora da v1~~. O Épico 7 segue inteiro. Há um único jeito de ditar (o "Padrão"), sem modos, sem IA e sem histórico; o único provedor é o Groq.

> Status: rascunho para aprovação · Versão: 1 · Atualizado em: 2026-10-23
> Fontes: docs/produto/prd.md (v2), docs/produto/resumo.md, relatório de engenharia reversa (só como referência de fluxo; nenhuma marca, texto ou visual do VoiceInk foi usado)
> Protótipo: docs/design/prototipo/index.html (abrir com duplo clique)
> Em caso de conflito entre este documento e o protótipo, este documento vence.

## Parte 1 — Identidade visual

### 1.1 Marca e estilo
Voice Desk é uma **ferramenta de bancada**: fica quieta na barra de menus ou na bandeja, e quando aparece precisa ser rápida, legível e confiável. A direção é "papel quente e tinta": fundo off-white levemente amarelado (não o branco frio de template), texto em grafite quase preto, um único acento verde-petróleo para ações, e o vermelho-coral reservado ao que é **gravação**. O gravador flutuante é a única peça escura e densa, para se destacar sobre qualquer programa. Sem gradientes, sem ilustrações decorativas, sem roxo; emojis só nos modos (são dados do usuário). Referência: utilitários de sistema bem acabados. Anti-referência: painel SaaS genérico e visual do VoiceInk (não copiar).

### 1.2 Cores
| Token | Hex (claro) | Hex (escuro) | Uso | Não usar para |
|---|---|---|---|---|
| --cor-fundo | #F6F4EF | #14181A | fundo da janela | cartões |
| --cor-superficie | #FFFFFF | #1D2326 | cartões, campos, tabelas | fundo da janela |
| --cor-superficie-2 | #EEEBE4 | #262D31 | barra de título, lateral, cabeçalho de tabela, blocos de texto | texto |
| --cor-texto | #1D2327 | #ECEFF0 | texto principal | — |
| --cor-texto-suave | #5A6368 | #A3ADB2 | textos de apoio, dicas | informação crítica |
| --cor-borda | #D9D5CB | #333B40 | divisórias e bordas de cartões | bordas de campos |
| --cor-borda-forte | #78838A | #6B767C | bordas de campos, interruptores desligados (≥ 3:1) | divisórias |
| --cor-primaria | #0F6B5C | #4FC3AC | ações principais, links, item ativo | fundos grandes |
| --cor-primaria-hover | #0B5649 | #6FD3BF | hover da primária | — |
| --cor-primaria-suave | #DCEFEA | #17332E | item ativo da lateral, selo informativo | texto |
| --cor-sobre-primaria | #FFFFFF | #0B1412 | texto sobre a primária | — |
| --cor-erro / --cor-erro-suave | #B3261E / #F9E3E1 | #FF8A80 / #3A1F1D | falha, validação, ação destrutiva | gravação |
| --cor-sucesso / --cor-sucesso-suave | #1E7A3C / #DDF1E3 | #6FD38B / #1B3324 | colado, chave válida, em dia | — |
| --cor-alerta / --cor-alerta-suave | #8A5A00 / #FBEFD3 | #F2B84B / #3A2E12 | colado sem refinamento, permissão pendente | — |
| --cor-gravador-fundo | #1B2124 | #1B2124 | gravador flutuante (igual nos dois temas) | outras telas |
| --cor-gravador-texto | #F2F4F5 | #F2F4F5 | texto do gravador | — |
| --cor-gravando | #FF5A47 | #FF5A47 | ponto de gravação e ícone gravando | erro comum |
| --cor-foco | #0F6B5C | #8FE0CF | anel de foco (3 px) | — |

Contrastes verificados (WCAG): texto/fundo 14,5:1; texto suave/fundo 5,6:1; branco/primária 6,4:1; primária/fundo 5,8:1; erro/superfície 6,5:1; sucesso 5,4:1; alerta 5,9:1; texto do gravador/fundo do gravador 14,8:1; ponto de gravação/fundo do gravador 5,3:1. Escuro: texto 15,5:1; primária/fundo 8,3:1; erro 7,0:1; borda forte 3,8:1. Bordas de campos e interruptores ≥ 3:1. Estado nunca depende só da cor (sempre símbolo ou texto junto).

### 1.3 Tipografia
Fonte do sistema (Inter se instalada): **-apple-system / Segoe UI Variable / Segoe UI**, para parecer nativo em cada plataforma e funcionar sem rede. Mono para atalhos e códigos: ui-monospace / Cascadia Mono / Consolas.
| Papel | Família | Tamanho | Peso | Altura de linha |
|---|---|---|---|---|
| Título 1 (tela) | base | 26px | 700 | 1.2 |
| Título 2 (cartão) | base | 19px | 650 | 1.2 |
| Título 3 | base | 15px | 650 | 1.2 |
| Corpo | base | 15px | 400 | 1.5 |
| Apoio / dica | base | 13px | 400 | 1.5 |
| Rótulo de campo | base | 14px | 600 | 1.5 |
| Atalho (kbd) | mono | 12,5px | 600 | 1 |
| Página do projeto, título | base | 34px | 700 | 1.2 |

### 1.4 Espaçamento, grid e pontos de quebra
Escala: 4 / 8 / 12 / 16 / 24 / 32 / 48 (`--espaco-1` a `--espaco-7`). Janela do app: largura máxima 1180 px, lateral fixa de 232 px e conteúdo fluido com 32 px de margem interna; cartões em grade de 1 ou 2 colunas (`minmax(260px, 1fr)`). Quebras: **≥ 1024** (lateral 232 px), **720–1023** (lateral 200 px, formulários em 1 coluna), **< 720** (lateral vira barra horizontal rolável no topo; tabelas viram lista de blocos; alvos de toque ≥ 44 px). Página do projeto: coluna única de até 1040 px.

### 1.5 Elevação, bordas e raios
Sombras: `--sombra-1` (cartões, 1 px suave), `--sombra-2` (menu da bandeja, gravador, toast), `--sombra-3` (janela, modal). Raios: `--raio-sm` 6, `--raio-md` 10 (campos, botões), `--raio-lg` 16 (cartões, modais), `--raio-pilula` 999 (selos, gravador, chips). Borda padrão 1 px em `--cor-borda`; campos em `--cor-borda-forte`.

### 1.6 Componentes (visual)
| Componente | Anatomia | Variantes | Estados visuais |
|---|---|---|---|
| Botão (`.btn`) | altura 40 px (32 pequeno, 44 no celular), raio md, texto 14/600 | primário (cheio verde), secundário (contorno), perigo (contorno/cheio vermelho), discreto (só texto) | hover escurece/clareia o fundo; foco anel 3 px; desabilitado 50% e cursor bloqueado; erro: variante perigo |
| Campo de texto / select / textarea | rótulo acima, campo 40 px, dica e erro abaixo | texto, busca, select, área de texto | hover borda mais escura; foco anel; erro borda 2 px vermelha + mensagem; desabilitado fundo cinza |
| Interruptor (`.interruptor`) | trilho 46×28 com bolinha; rótulo ao lado | — | ligado verde; desligado cinza (≥ 3:1); foco anel; desabilitado 50% |
| Cartão (`.cartao`) | título 2, dica, conteúdo; raio lg, sombra 1 | cartão de modo (clicável, com selos) | hover do cartão de modo: borda primária |
| Tabela (`.tabela-wrap`) | cabeçalho em caixa-alta suave, linhas de 48 px | histórico, substituições, máquinas | hover de linha; no celular vira lista de blocos |
| Selo (`.selo`) | pílula 12,5/600 | neutro, ok, alerta, erro, info | — |
| Chip (`.chip`) | pílula com “×” de remover (alvo 28 px) | palavra, programa, gatilho | hover do × em vermelho suave |
| Aviso (`.aviso`) | ícone + texto + ações opcionais, fundo suave e borda | info, ok, alerta, erro | erro e alerta usam `role="alert"` |
| Modal | caixa 520 px sobre fundo escurecido | confirmação, formulário | foco vai para o primeiro campo; Esc fecha |
| Toast | caixa escura no canto inferior direito, filete colorido à esquerda | normal, alerta, erro | some em ~5 s; `aria-live="polite"` |
| Abas (configurações) | links com sublinhado de 3 px no ativo | — | ativo: texto e sublinhado primários |
| Lateral | itens de 40 px com emoji-ícone | — | ativo com fundo `--cor-primaria-suave`; hover; foco |
| Gravador flutuante (`.gravador`) | pílula escura: ponto de estado, emoji+nome do modo, nível de áudio, mensagem, tempo | gravando, processando, ok, aviso, erro | gravando: ponto coral pulsando e barras de nível; processando: âmbar; ok: verde; aviso/erro: âmbar. Com movimento reduzido, sem animação |
| Menu da bandeja | caixa de 300 px: iniciar/parar, modo padrão (com ✓), caixa rápida de vocabulário, histórico, configurações, sair | Mac (barra de menus, no topo) e Windows (bandeja, embaixo) | item com hover primário suave; foco visível |
| Ícone da bandeja | símbolo de microfone | parado 🎙, gravando 🔴, processando ⏳, erro ⚠ (símbolo + dica de texto) | — |
| Barra de progresso | trilho 10 px com preenchimento | download do modelo | `role="progressbar"` com valor |
| Esqueleto de carregamento | blocos de 56 px com brilho | listas | animação desligada com movimento reduzido |

### 1.7 Faça e não faça
- Faça: reservar o coral à gravação; mostrar sempre o nome e o emoji do modo no gravador; escrever mensagens que dizem o que aconteceu e o que fazer; manter atalhos em `kbd`.
- Não faça: usar cor sozinha para estado; copiar texto, ícone, som ou marca do VoiceInk; colocar chave de API inteira na tela; criar telas fora do PRD.

## Parte 2 — Experiência

### 2.1 Fundamentos
Aplicativo de **desktop para macOS 14+ e Windows 10 (22H2)/11**, mesma experiência nas duas plataformas; diferem só as teclas (⌘/⌥/⌃/⇧ no Mac; Ctrl/Alt/Shift no Windows) e o local do ícone (barra de menus no topo; bandeja embaixo). Interface em **português do Brasil**. Não há login, conta, perfis nem administração: um único perfil (BD e usuário compartilhado usam o mesmo app, cada um na própria instalação). Biblioteca de componentes: decisão do Otávio; os tokens acima valem em qualquer uma. O app tem três superfícies: **gravador flutuante** (sobre qualquer programa, sem tirar o foco), **menu do ícone** e **janela de configuração** (testar ditado, texto e configurações). Mais a **página pública do projeto** (web).

### 2.2 Arquitetura de informação
```
Ícone (barra de menus / bandeja) → Menu: iniciar/parar, vocabulário rápido, configurações, sair
Janela
 ├─ Ditado: Testar ditado (só protótipo: simula o ditado e os estados do gravador)
 ├─ Texto: Vocabulário · Substituições · Limpeza do texto
 └─ App: Configurações (Geral · Atalhos · Chave do Groq · Permissões) · Ícone e menu · Primeiro uso · Sobre
Página do projeto (web): baixar · instalar · requisitos · versões · código aberto
```
A tela **Testar ditado** existe no protótipo para o BD ver o fluxo; no produto o ditado acontece nos outros programas. Se o time quiser uma tela assim no produto, é decisão nova (não está no PRD).

| Tela | Rota no protótipo | Perfis | Funcionalidades (F-xx) |
|---|---|---|---|
| Gravador flutuante | `app.html#/ditado` | todos | F-01, F-02, F-03, F-23.2, F-26 |
| Vocabulário | `app.html#/vocabulario` | todos | F-07 |
| Substituições | `app.html#/substituicoes` | todos | F-08 |
| Limpeza do texto | `app.html#/limpeza` | todos | F-09 |
| Config. Geral (**inclui o bloco "Texto do ditado"**: idioma, remover preenchimento, formatar, Enter depois de colar) | `app.html#/config/geral` | todos | F-02, F-03, F-05.2, F-09, F-23, F-24, F-25, F-26 |
| Config. Atalhos | `app.html#/config/atalhos` | todos | F-01.1, F-01.4 |
| Config. Chave do Groq | `app.html#/config/provedores` | todos | F-04 |
| Config. Permissões | `app.html#/config/permissoes` | todos | F-03.4, F-06.2 |
| Ícone e menu | `app.html#/bandeja` | todos | F-05, F-07.3 |
| Primeiro uso | `app.html#/primeiro-uso` | todos | F-06 |
| Sobre | `app.html#/sobre` | todos | F-27.4 |
| Página do projeto | `projeto.html` | qualquer pessoa | F-27 |

### 2.3 Fluxos principais
#### Fluxo UJ-1 — BD responde um e-mail no Windows ditando
Personagem: BD, no Outlook com a resposta aberta.
1. Aciona `Ctrl+Alt+Espaço` com o cursor no corpo da resposta (gravador aparece em ≤ 300 ms, foco não sai do campo).
2. Gravador mostra “✍️ Padrão”, nível de áudio e tempo; ele fala.
3. Aciona o atalho de novo: gravador “Processando…” e as etapas transcrever → limpar → colar.
4. O texto limpo é colado no corpo do e-mail; gravador “Texto colado” e some.
Momento de valor: texto pronto em até 6 s. · Caminho de erro: sem conexão → o gravador mostra “Falhou: sem conexão” e o áudio fica guardado na máquina. · No protótipo: `app.html?plat=win#/ditado` (cenário “Sem internet na transcrição”).

#### Fluxo UJ-2 — BD manda uma mensagem rápida no Slack (Mac)
Personagem: BD conversando com a Tâmara Quintanilha.
1. Atalho com o cursor na caixa do Slack; gravador “✍️ Padrão”.
2. Fala a frase com o nome incomum; atalho de novo.
3. Nome certo pelo vocabulário, texto colado e, **se o BD ligou “Apertar Enter depois de colar” em Configurações › Geral** (desligado por padrão, vale para todo programa), Enter enviado (marcado “↵”).
Caminho de erro: desiste → `Esc` duas vezes em 1,5 s cancela; nada é colado, enviado ou guardado. `Esc` uma vez passa para o programa. · No protótipo: `app.html?plat=mac#/ditado`, programa Slack.

#### ~~Fluxo UJ-3 — Termo novo no Mac vale no Windows~~ (fora da v1)
Dependia de sincronização, histórico ou transcrição local (Épicos 3 a 6). Removido do protótipo.

#### ~~Fluxo UJ-4 — BD recupera um ditado que não colou~~ (fora da v1)
Dependia de sincronização, histórico ou transcrição local (Épicos 3 a 6). Removido do protótipo.

#### ~~Fluxo UJ-5 — BD dita sem internet~~ (fora da v1)
Dependia de sincronização, histórico ou transcrição local (Épicos 3 a 6). Removido do protótipo.

#### Fluxo UJ-6 — BD instala numa máquina nova
1. Primeiro uso: Boas-vindas → Permissões → Atalho → Chave do Groq → Teste.
2. Cada permissão traz uma frase do porquê e o estado; qualquer passo pode ser pulado.
3. Cola a chave do Groq e faz o ditado de teste.
Caminho de erro: permissão negada → aviso “o que deixa de funcionar” e como conceder depois (Configurações › Permissões). · No protótipo: `app.html?plat=win#/primeiro-uso`.

#### Fluxo UJ-7 — Colega da Going2 começa a usar
Personagem: Ana, do time comercial.
1. Baixa o instalador na página do projeto e instala (aviso de segurança explicado).
2. No primeiro uso, cadastra a própria chave do Groq.
3. Dita um e-mail.
Caminho de erro: sem chave → o passo explica como obter a chave; não há alternativa local na v1 (sem chave não há ditado). · No protótipo: `projeto.html`, `#/primeiro-uso`.

### 2.4 Estados por tela
| Tela | Vazio | Carregando | Erro | Sem permissão | Outros |
|---|---|---|---|---|---|
| Vocabulário | “Seu vocabulário está vazio” + campo | esqueletos | “Tentar de novo” | — | busca sem resultado; duplicata e limite de 500 com mensagem |
| Substituições | explicação + formulário | esqueletos | “Tentar de novo” | — | duplicata recusada; desativada fica esmaecida |
| Limpeza | “Lista vazia” | esqueletos | “Tentar de novo” | — | prévia ao vivo |
| Config. Chave do Groq | sem chave: formulário + link | esqueletos | chave inválida | — | testando a chave (≤ 10 s) |
| Config. Permissões | — | — | — | microfone/acessibilidade negados com “o que deixa de funcionar” | concedida |
| Gravador | — | processando | falhou; modelo ausente; sem permissão do microfone | texto copiado (colagem recusada) | colado sem refinamento; nenhuma fala; microfone padrão em uso; aguarde |
| Primeiro uso | — | — | — | permissão pendente com aviso | passo pulado |

### 2.5 Padrões de interação e componentes (comportamento)
| Componente/padrão | Regra de comportamento |
|---|---|
| Formulário | Validação ao enviar; o erro aparece abaixo do campo, em texto, com borda vermelha e o foco volta ao campo; mensagem diz como corrigir. |
| Interruptor | Efeito imediato e toast curto “Preferência salva”; nunca precisa de “Salvar”. |
| Ação destrutiva | Modal com o nome do item e botão vermelho com verbo (“Excluir modo”); apagar tudo repete a quantidade. |
| Atalho | Precisa de modificador; atalho do sistema é recusado; vale sem reiniciar o app. |
| Chave de API | Salvar testa no provedor (≤ 10 s) e mostra válida/inválida; depois só os 4 últimos caracteres; trocar e apagar (com confirmação). |
| Busca (vocabulário, histórico) | Filtra ao digitar; “nenhum resultado” com ação de limpar. |
| Importar configuração | Mostra “Agora × Depois” por item e pede confirmação; arquivo inválido é recusado sem alterar nada. |
| Cancelar ditado | `Esc` duas vezes em 1,5 s; uma só é repassada ao programa e o app explica. |
| Gravador | Aparece sem roubar o foco; mostra modo, nível e tempo; avisa aos 9:30; durante o processamento o atalho responde “Aguarde”. |
| Colagem recusada | Texto fica na área de transferência e o aviso diz a tecla de colar da plataforma. |
| Toast | 5 s, no canto inferior direito, não bloqueia; erros importantes também ficam na tela. |
| Modal | Foco no primeiro campo, `Esc` fecha, foco volta ao gatilho. |

### 2.6 Voz e textos
Tom direto, calmo, sem jargão e sem culpar. Botões são verbos (“Salvar modo”, “Baixar modelo”, “Transcrever de novo”). Erros dizem o que houve **e** o que fazer: “Sem conexão com a internet. O áudio foi guardado; você pode transcrever de novo.” Sucessos são curtos: “Texto colado”, “Modo salvo.”. Dados do usuário são tratados por “você”. Nunca “oops”. Nunca textos do VoiceInk.

### 2.7 Acessibilidade
Meta **WCAG 2.1 AA** (RNF-11) nas telas de configuração e histórico: contraste conforme 1.2; todas as ações por teclado, na ordem visual; foco sempre visível (anel de 3 px); rótulos em todos os campos; interruptores com `role="switch"` e nome; avisos de erro com `role="alert"`; resultados dinâmicos (toasts, gravador) em regiões `aria-live`; alvos ≥ 44 px no celular/estreito e ≥ 32 px no desktop; estado nunca só por cor; `prefers-reduced-motion` desliga pulso, nível e esqueleto. O gravador flutuante não toma foco por design; seu estado também é anunciado ao leitor de tela.

### 2.8 Responsividade
O produto é de desktop, mas a janela pode ser redimensionada e o protótipo é testado em 1440 px e 390 px. ≥ 1024: lateral de 232 px; 720–1023: lateral 200 px e formulários em 1 coluna; < 720: lateral vira barra horizontal rolável, tabelas viram blocos, botões de 44 px, página do projeto em coluna única.

## Premissas e perguntas em aberto
- [PREMISSA] Cor de acento verde-petróleo e fundo quente como identidade própria, sem marca prévia do BD. Se houver logo ou cor definida, troco só os tokens.
- [PREMISSA] Nenhum logotipo foi criado; o produto usa o nome em texto e o símbolo de microfone do sistema como ícone provisório. Ícone do app e dos instaladores precisam de definição (fora do PRD).
- [PREMISSA] A tela “Testar ditado” é só do protótipo (para simular o fluxo). Não faz parte da v1 do PRD.
- [PREMISSA] Aviso de segurança do instalador na página do projeto assume instalador não assinado (pergunta 6 do PRD).
- [PREMISSA] Licença exibida em “Sobre” como “a definir” até a pergunta 5 do PRD ser decidida.
- [PREMISSA] No Windows, a permissão de colagem aparece como “Colar em outros programas” apenas como orientação; o comportamento real depende do Otávio.
- Divergência com o PRD: nenhuma. O PRD não pede uma tela de “início/painel”, por isso a janela abre em Modos/Testar ditado e o uso diário acontece pelo atalho e pelo ícone.
- [PREMISSA] (ENG-104) Idioma, remoção de preenchimento, formatação e Enter depois de colar ficam em **Configurações › Geral, bloco "Texto do ditado"**, valendo para todos os ditados. A lista de palavras de preenchimento continua em "Limpeza do texto". Enter vem **desligado**.
- [PREMISSA] (ENG-104) O menu do ícone da v1 tem: iniciar/parar ditado, adicionar palavra ao vocabulário, abrir configurações e sair. O primeiro uso tem 5 passos: Boas-vindas, Permissões, Atalho, Chave do Groq e Teste. Não há alternativa sem chave.
- [PREMISSA] (ENG-104) Sem histórico nem retenção, o app não oferece tela para ver ou apagar áudios e textos guardados; é decisão para o BD/Otávio (ver premissa do PRD). Se o BD quiser, basta um botão "Apagar dados guardados" em Configurações › Geral.
- Os componentes de modo (cartão de modo, editor, emoji do modo no gravador, linha de histórico) nas seções 1.6, 2.5 e 2.6 ficam sem uso na v1; o gravador mostra apenas "✍️ Padrão".
