# Impacto: corte dos Épicos 3, 4, 5 e 6 da v1

> Card: ENG-104 · Atualizado em: 2026-10-27
> Fontes: [prd.md](../produto/prd.md) v3 (seções 7.3 e 11), [design.md](../design/design.md) (escopo v3), [arquitetura.md](arquitetura.md) v2 e o código da `main` em `28459ed`

## 1. Resumo

O corte não pede migração nem tabela nova. O esquema inicial já tem tudo o que a v1 usa, e as tabelas dos épicos cortados ficam vazias, prontas para quando eles voltarem. O que muda é: (1) o que o app guarda de cada ditado, agora que não há histórico nem retenção (ADR-11); (2) onde ficam as opções de cada ditado, agora que não há modos; (3) o mapa de dependências, porque F-05, F-09 e F-06 dependiam de funcionalidades cortadas; e (4) três ajustes no código do Épico 1, que já está pronto.

## 2. Dados

### 2.1 O que fica guardado de cada ditado (ADR-11, decisão do BD)

Hoje o código guarda só os ditados que **falharam** (áudio em `audios/` e item `falhou` em `historico_item`, `transcricao/falha.rs`). Os ditados concluídos não deixam registro (a etapa `registrar` ainda é provisória). A premissa do PRD v3 é guardar áudio e texto de todos, sem apagar nunca. Recomendo uma regra mais enxuta:

| Caso | Item em `historico_item` | Áudio em `audios/` | Textos |
|---|---|---|---|
| Colado ou não colado | Sim, só dados técnicos: estado, programa, duração, provedor, modelo, tempos | Apagado assim que o texto é colado ou copiado | Não guarda (`texto_transcrito`, `texto_limpo` e `texto_final` ficam vazios) |
| Falhou (RF-02.3) ou app fechou no meio (RNF-08) | Sim, `falhou`, com o motivo | Fica guardado | Não existe texto nesse caso |
| Cancelado durante o processamento | Sim, `cancelado` | Apagado | Não guarda |
| Cancelado durante a gravação (Esc duplo) | Não (premissa já registrada na arquitetura) | Não é gravado | — |
| Sem fala (RF-26.2) | Não | Apagado | — |

**Por quê:** sem tela de histórico, o texto e o áudio dos ditados concluídos não servem para nada ao usuário e acumulam dados de clientes na máquina para sempre (RNF-07). Os dados técnicos bastam para RF-03.5, para as métricas M-1 e M-2 e para os testes. Falhas são raras e pequenas (cerca de 1 MB por minuto) e são o único caso em que o áudio ainda pode ser útil, quando o histórico e o "transcrever de novo" voltarem.

**Alternativas:**
- Guardar áudio e texto de todos, sem apagar (premissa atual do PRD): o mais simples, mas o volume e os dados pessoais crescem sem limite e sem jeito de apagar pela interface.
- Trazer só a retenção (F-17) de volta: resolve o acúmulo, mas devolve uma tela e uma história ao escopo.

**Efeitos:**
- O botão "Apagar dados guardados" (premissa da Nina) deixa de ser necessário na v1: só sobram os áudios das falhas.
- Os testes que pediam "ouvir o áudio no histórico" (RF-23.1 com headset, RF-24.2 sem o som de início) passam a usar um ditado com falha forçada (chave inválida ou sem internet), cujo áudio fica em `audios/AAAA/MM/<id>.flac`. O estado e os tempos dos outros ditados são conferidos no log local (RNF-09) ou consultando `voice-desk.db` com `sqlite3`, sem tela para o usuário.
- O áudio passa a ser gravado em `audios/` antes de transcrever (como a seção 3 da arquitetura já previa), para cumprir a RNF-08. Hoje isso só acontece depois da falha.
- Se o BD preferir a premissa do PRD, a regra muda só na etapa `registrar`; o esquema é o mesmo.

### 2.2 Opções de cada ditado sem modos

Idioma, provedor e modelo (RF-02.1, RF-02.2), Enter depois de colar (RF-03.3) e limpeza e formatação (RF-09.1) continuam nas colunas da tabela `modo`, na linha do **modo padrão "Ditado"**, que o F-00 já cria na primeira abertura e o `transcricao/config.rs` já lê. A interface não mostra modos: o bloco "Texto do ditado" de Configurações › Geral (design v3) lê e grava essa linha.

**Por quê:** zero migração; o código pronto do Épico 1 continua valendo; quando o Épico 3 voltar, o modo padrão vira o primeiro modo da lista sem conversão de dados. **Alternativa:** mover as opções para `preferencia_local`. Exigiria migrar o que já foi gravado e desfazer isso quando os modos voltassem.

Regra: o núcleo sempre usa o modo padrão. A escolha por programa, atalho ou palavra-gatilho não existe na v1.

### 2.3 Tabelas sem uso na v1

`modo_programa`, `palavra_gatilho`, `atalho` com escopo `modo:`, `historico_busca`, `modelo_local` e `sincronizacao_estado` ficam no esquema, vazias. As colunas de sincronização (`atualizado_em`, `atualizado_por`, `excluido_em`) continuam sendo preenchidas por `configuracao::gravar()`. Isso custa nada e evita migração quando os épicos voltarem. Nenhuma tabela é apagada.

## 3. Lacunas abertas pelo corte (para a Helena)

1. **Bloco "Texto do ditado" em Configurações › Geral.** O idioma (RF-02.2) e o Enter depois de colar (RF-03.3) seriam escolhidos no editor de modo (F-10), que saiu. Hoje não há tela para nenhum dos dois. O lugar mais natural é a E2.3 (F-09), que já cria as opções de limpeza: ela passa a entregar o bloco inteiro (idioma, remover preenchimento, formatar, Enter).
2. **Registro do ditado (ADR-11).** A etapa `registrar` (área ditado, F-01) segue provisória: falta gravar o áudio antes de transcrever, criar o item `processando`, atualizar o resultado, apagar o áudio dos concluídos e marcar `falhou` ao reabrir (RNF-08). Cabe numa história técnica curta, antes ou junto da E7.4 (que mexe no mesmo trecho).
3. **Ajustes no que o Épico 1 já entregou** (história técnica de limpeza, área bandeja e fundação):
   - Menu do ícone: tirar "modo padrão" e "Abrir histórico" (`src-tauri/src/bandeja/menu.rs`, `src/janelas/menu`).
   - Rotas Modos, Histórico, Sincronização, Modelo local e Dados: mostrar "Fora da v1", como o protótipo (`src/app/rotas.tsx`). As pastas em `src/modulos` ficam.
   - Primeiro uso (E1.7, pronta): hoje tem o passo "Sincronização (opcional)" e o caminho "sem chave → transcrição local" (`src/modulos/primeiro-uso/pagina.tsx`). Passa aos 5 passos do design v3: Boas-vindas, Permissões, Atalho, Chave do Groq e Teste.

A Helena decide como empacotar; os três itens não mudam o PRD.

## 4. Ordem sugerida do que fica

Sem o F-10, as funcionalidades restantes dependem só do que já está pronto (Épico 1):

- **Agora, em paralelo:** E2.1 (F-07), E2.2 (F-08), E2.3 (F-09 + bloco "Texto do ditado") e a limpeza do item 3.3.
- **Depois ou junto, sem juntar duas que mexem na captura:** E7.1 (F-23), E7.2 (F-24), E7.3 (F-25), E7.4 (F-26) e o registro do ditado (item 3.2).

Pares que não devem rodar ao mesmo tempo:
- F-23 × F-24 × F-26 × registro do ditado: mexem em `src-tauri/src/audio` ou na gravação do áudio.
- F-24 × F-25: ordem entre som e pausa de mídia (ver arquitetura, seção 10).
- F-07 × limpeza do menu: os dois mexem no menu do ícone (F-07 usa a caixa de vocabulário).
- F-08 × F-09: etapas `substituir` e `limpar` em arquivos separados; podem rodar juntas.

## 5. Riscos

- **Áudio das falhas sem apagamento:** cresce só com falhas; aceitável na v1. Volta a ter limpeza quando a F-17 voltar.
- **Métrica M-1 por 30 dias:** o log local guarda só 7 dias (RNF-09); por isso a contagem por dia vem de `historico_item`, que guarda os dados técnicos sem prazo.
- **Volta dos épicos:** nenhuma decisão aqui fecha porta. Modos, histórico, sincronização e modelo local voltam sobre o mesmo esquema.
