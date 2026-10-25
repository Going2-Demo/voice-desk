# Retrospectiva — Épico 1 · Ditado de ponta a ponta

Produto: Voice Desk · Épico: ENG-58 · Período: 2026-10-24 a 2026-10-25 · Escrita por Helena - Scrum Master

## Resumo
O épico entregou as oito histórias planejadas (ENG-65 a ENG-72) e oito bugs: dois achados na conferência visual (ENG-93 e ENG-94), quatro no teste nativo do BD (ENG-95 a ENG-98) e dois no segundo teste no Mac (ENG-99 e ENG-100). Os 16 cards estão em `done`. Ditado com atalho global, transcrição no Groq, colagem no programa em foco, chaves no cofre, ícone na bandeja, primeiro uso guiado e instaladores públicos existem em dev. O principal destaque foi o fluxo paralelo com três Devs, sem história sobrepondo áreas. O principal problema foi a reprovação recorrente de CI na primeira história (ENG-65), que consumiu a maior parte do ciclo dela.

## Entregue vs planejado
| História | Planejada? | Status | Dev | Retornos | Ciclo (todo → done) |
|---|---|---|---|---|---|
| E1.1 · Fundação percorrível (ENG-65) | sim | entregue | — | 1 | 13 h |
| E1.2 · Gravar com atalho global e gravador flutuante (ENG-66) | sim | entregue | — | 0 | 5,4 h |
| E1.3 · Transcrição na nuvem Groq (ENG-67) | sim | entregue | Roberval | 0 | 1,6 h |
| E1.4 · Colar no programa em foco (ENG-68) | sim | entregue | Bia | 0 | 1,5 h |
| E1.5 · Chaves de API no cofre do sistema (ENG-69) | sim | entregue | — | 0 | 5,3 h |
| E1.6 · Ícone na bandeja e iniciar com o sistema (ENG-70) | sim | entregue | Davi | 1 | 8,1 h |
| E1.7 · Primeiro uso guiado (ENG-71) | sim | entregue | Roberval | 0 | 4,7 h |
| E1.8 · Instaladores públicos e página do projeto (ENG-72) | sim | entregue | — | 0 | 4,1 h |
| Bug · rolagem horizontal abaixo de 720 px (ENG-93) | acrescentado (conferência visual) | entregue | Roberval | — | 7 min |
| Bug · casca não recolhe a lateral abaixo de 720 px (ENG-94) | acrescentado (conferência visual) | entregue | — | 0 | 1,2 h |
| Bug · ditado não cola no Windows (ENG-95) | acrescentado (teste nativo do BD) | entregue | Roberval | 0 | 36 min |
| Bug · tela inicial vazia após a instalação (ENG-96) | acrescentado (teste nativo do BD) | entregue | Davi | 0 | 20 min |
| Bug · Mac não pede permissão de microfone (ENG-98) | acrescentado (teste nativo do BD) | entregue | Bia | 0 | 26 min |
| Bug · instalador do Mac bloqueado pelo Gatekeeper (ENG-97) | acrescentado (teste nativo do BD) | entregue com passo manual documentado | Bia | 0 | 27 min |
| Bug · indicador de permissão do microfone não atualiza (ENG-99) | acrescentado (segundo teste no Mac) | entregue | Davi | 0 | 32 min |
| Bug · ditado no Mac não cola o texto automaticamente (ENG-100) | acrescentado (segundo teste no Mac) | entregue | Roberval | 0 | 52 min |

A coluna Dev mostra "—" onde o card não registra o Dev de origem. Nenhuma história foi cancelada.

Requisitos do PRD cobertos: F-01 a F-06 do Épico 1 (E-01), com F-02 (ENG-67), F-03 (ENG-68) e F-06 (ENG-71) entregues no mesmo dia. Requisitos previstos e não entregues: nenhum.

## O que o BD pode verificar em dev
- Releases de pré-release em https://github.com/going2-demo/voice-desk/releases (último build: `dev-dcc6111`, gerado pelo `build-dev`). Passo a passo: instalar no Mac e no Windows, seguir o primeiro uso, ditar 20 s em português num modo com Groq e conferir a colagem no e-mail e no chat, e ver o ícone na bandeja ou barra de menus.
- Janela de 390 px: Configurações e a lateral da casca devem rolar verticalmente, sem rolagem horizontal (ENG-93 e ENG-94).

## Pendências
- **Decisão do BD:** o Mac do instalador segue sem assinatura nem notarização. ENG-97 entregou o passo manual (aviso e "Abrir Assim Mesmo" no `site/index.html`). Assinar e notarizar exige conta Apple Developer e segredos no CI; fica para decisão do BD, sem card aberto.
- Nenhuma pendência de história. Sem roteiro nativo pendente em `tests/nativo/pendencias.md` (arquivo ausente).
- Mac: a permissão do microfone (ENG-99) e a colagem automática (ENG-100) estão corrigidas em `done`, mas ainda não foram confirmadas pelo BD no Mac. Esse é o teste que falta antes da aprovação.
- Para depois, sem card aberto: avaliar se a checagem de formatação entre Windows e macOS deve rodar antes do PR (ver ações).

## Retornos e causas
Total: 2 retornos em 2 histórias (ENG-65 e ENG-70). Além disso, o Igor reprovou ENG-65 três vezes na revisão de código sem somar Retornos.

| Causa | Quantas vezes | Exemplos |
|---|---|---|
| Critério de aceite não atendido | 1 | ENG-65: instalador do Mac gerado só para arm64; corrigido no PR #2 com build universal |
| Divergência visual do protótipo (Nina) | 1 | ENG-70: ícone e menu da bandeja fora do protótipo; corrigido e aprovado depois do Retorno 1 |
| Regressão em funcionalidade anterior | 0 | — |
| Critério ambíguo na história | 0 | — |
| Ambiente de dev instável | 0 | — |
| Reprovação de CI na revisão de código (fora de Retornos) | 3 rodadas em ENG-65 | Windows: `pnpm biome ci` com diferenças de formatação, depois CRLF e `package.json` |

## Reinícios de agentes
Total: 6, pelo metadado `reinicios_total` do vigia. ENG-66 (1), ENG-69 (1), ENG-70 (3, todos com o card visto em `em_teste`) e ENG-72 (1).

## Tempo de ciclo
Médio: 3,0 h (de `todo` a `done`, nos 16 cards). Os bugs do teste nativo e do segundo teste no Mac levaram entre 20 e 52 min cada.
Mais rápido: ENG-93, 7 min. Mais lento: ENG-65, 13 h, por três rodadas de CI reprovado no Windows e a correção do build universal do Mac.
Vazão no período: S43 (2026-10-24 a 2026-10-25): 16 cards concluídos, sendo 8 histórias e 8 bugs.

## Aprendizados
- O que funcionou: três Devs em paralelo com WIP 1 e dependências explícitas; nenhuma colisão de áreas; a conferência visual pela Nina pegou o desvio do ícone da bandeja antes do `done`.
- O que atrapalhou: a primeira história fez a fundação passar por três rodadas de CI reprovado, o que travou as outras por horas; o instalador do Mac só gerou uma arquitetura, o que a Marta só viu no teste; o teste simulado não pegou os quatro defeitos nativos (permissão de microfone, Gatekeeper, captura no Windows e tela inicial), que só apareceram no teste do BD.

## Ações para o próximo épico
| Ação | Responsável | Como saberemos que funcionou |
|---|---|---|
| Incluir no AGENTS.md o comando local de formatação e lint que o CI roda, para rodar em Windows e macOS antes de abrir o PR | Otávio | Zero reprovações de CI na primeira revisão do Igor |
| Histórias que geram instalador precisam de critério explícito de arquitetura alvo (Mac universal e Windows) | Helena (quebra) | Zero retornos da Marta por arquitetura de instalador |
| Histórias com tela incluem checagem visual contra o protótipo antes de `em_teste` | Nina e Devs | Zero retornos visuais no próximo épico |
| Antes de `in_review` do épico, roteiro nativo de instalação e permissões em Mac e Windows, rodado pela Marta | Helena (retro) e Marta | Zero bugs de instalação ou permissão achados pelo BD |

## Decisão do BD
Aguardando aprovação do BD.
