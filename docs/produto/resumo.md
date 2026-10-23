# Resumo do produto: Voice Desk

> Status: rascunho para aprovação · Versão: 1 · Atualizado em: 2026-10-23
> Forma de entrada: engenharia reversa (VoiceInk 2.13, o "VoiceLNK" do pedido)
> Fontes: [relatório de engenharia reversa](engenharia-reversa/relatorio.md), card-mãe ENG-50, card ENG-51

## Resumo executivo

O Voice Desk é um app de **ditado por voz para desktop**, para **Mac e Windows**. Com um atalho de teclado, o BD fala em qualquer programa (e-mail, chat, editor, navegador) e o texto aparece pronto onde o cursor está. Se quiser, o texto vem antes refinado por IA conforme o contexto: formal num e-mail, enxuto num chat.

Hoje o BD usa o VoiceInk, que resolve isso bem, mas **só existe para macOS**. Ele depende de vários recursos exclusivos da Apple ([relatório §8](engenharia-reversa/relatorio.md#8-integrações-externas)). O Voice Desk reproduz o **comportamento** que importa no dia a dia do VoiceInk, com experiência e configuração iguais nas duas plataformas, em produto próprio (sem reaproveitar código nem marca do VoiceInk, que é GPL-3.0).

## O problema

O BD dita textos no Mac com o VoiceInk, com modos e dicionário próprios. Quando está no Windows, perde o atalho, os modos, o vocabulário e o histórico. Ou digita tudo, ou usa outra ferramenta sem as mesmas regras. Cada máquina vira um jeito diferente de trabalhar, e o que ele configurou num lugar não vale no outro.

## A solução

Um app que fica discreto na bandeja ou barra de menus e funciona igual no Mac e no Windows:
1. Atalho global → gravador flutuante com indicação de áudio → o BD fala → atalho de novo (ou Esc para cancelar).
2. A fala vira texto, com transcrição na nuvem ou na própria máquina.
3. O texto passa pelo dicionário (vocabulário e substituições) e, se o **modo** pedir, por um refinamento com IA.
4. O resultado é **colado no programa em foco**.
5. Tudo fica num **histórico** pesquisável, com retenção configurável.

O **modo** é escolhido automaticamente pelo programa em uso (e, se der, pelo site aberto).

## O que diferencia

Não é inovação de categoria: é **o mesmo fluxo de ditado que o BD já aprova, nas duas plataformas, com a configuração dele valendo nas duas**, sob controle dele (chaves de API próprias, sem licença de terceiros). A alternativa "continuar no VoiceInk" não atende o Windows.

## Para quem

| Perfil | Quem é | O que precisa | Sucesso para ele |
|---|---|---|---|
| BD (usuário principal) | Usa Mac e Windows no trabalho, dita muito texto e já domina modos e dicionário do VoiceInk | Ditar em qualquer programa nas duas máquinas, com os mesmos modos, vocabulário e atalho | Trocar de máquina sem mudar o jeito de ditar |

**Não é para (v1):** equipes, venda a terceiros, celular. [PREMISSA] O Voice Desk é para uso próprio do BD.

## Jornadas principais

_Programas citados são exemplos ilustrativos._

- **E-mail no Windows:** o BD abre o Outlook, aciona o atalho, dita a resposta em tom informal e solta. O modo "E-mail", ativado porque o programa em foco é o Outlook, entrega o texto formatado e corrigido no corpo da mensagem em poucos segundos.
- **Chat rápido no Mac:** no Slack, o BD dita uma frase curta; o modo "Ditado" cola o texto quase literal, com os nomes de clientes escritos certo graças ao vocabulário.
- **Mesma configuração nas duas máquinas:** o BD cadastra um termo novo no dicionário no Mac; no dia seguinte, ditando no Windows, o termo já é reconhecido. [PREMISSA] Sincronizar a configuração entre máquinas entra na v1 (ver perguntas).

## Forma do produto

App de desktop instalável para **macOS e Windows**, com ícone na barra de menus ou bandeja, atalho global e gravador flutuante. Funciona com internet (transcrição e IA na nuvem); [PREMISSA] a transcrição local, sem internet, entra na v1 com pelo menos um modelo. Interface em português do Brasil.

## Critérios de sucesso

- O BD usa o Voice Desk **no lugar do VoiceInk** no Mac e no Windows durante 30 dias seguidos. [PREMISSA] (prazo a confirmar)
- O fluxo central (FL-01 do relatório) funciona igual nas duas plataformas: atalho, gravar, transcrever, refinar, colar no programa em foco.
- Texto colado em até poucos segundos depois de parar a gravação, num ditado de até 1 minuto. [PREMISSA] (meta exata a definir com o BD no PRD)
- Modos, vocabulário e substituições configurados uma vez valem nas duas máquinas.

## Escopo da primeira versão

O que copiar, mudar e deixar de fora, com base no [mapa de funcionalidades](engenharia-reversa/relatorio.md#3-mapa-de-funcionalidades):

**Copiar o comportamento (v1):**
- **Gravação:** atalho global para gravar e parar; Esc cancela; gravador flutuante com indicação de áudio; escolha do microfone; sons de início e fim; pausar a mídia durante a gravação (ER-01 a ER-07).
- **Transcrição:** na nuvem com chave do usuário; idioma e dica de vocabulário (ER-09, ER-11).
- **Texto:** vocabulário personalizado, substituição de palavras, remoção de palavras de preenchimento, formatação (ER-14, ER-20, ER-21).
- **Saída:** colar no programa em foco, com opção de enviar Enter depois (ER-13).
- **Modos:** prompt e modelo próprios; refinamento por IA com chave do usuário; contexto do texto selecionado e da área de transferência; troca automática pelo programa em foco (ER-15 a ER-18, sem o OCR da tela).
- **Histórico:** texto original e refinado, áudio, busca, retenção automática, exportação CSV (ER-23 a ER-25).
- **Sistema:** chaves de API guardadas no cofre seguro do sistema; iniciar com o sistema; primeiro uso guiado com permissões (ER-27 a ER-29).

**Mudar:**
- **Mac e Windows com a mesma experiência**: é a razão do produto.
- **Configuração sincronizada entre as máquinas** (modos, prompts, dicionário), em vez do iCloud só para o dicionário e só entre Macs. [PREMISSA]
- **Conjunto enxuto de provedores** na v1 (2 ou 3 de transcrição e 2 ou 3 de IA, os que o BD usa), em vez de cerca de 20. [PREMISSA] Hoje a instalação dele tem modelo escolhido para Groq e Ollama e o Whisper large-v3-turbo baixado ([relatório EV-12/EV-13](engenharia-reversa/relatorio.md#12-índice-de-evidências)).
- **Sem licença, teste ou ativação** (uso próprio). [PREMISSA]

**Fora (por enquanto):**
- **Contexto por OCR da tela** e **troca de modo pelo site do navegador**: dependem de recursos que diferem muito entre Mac e Windows; avaliar na fase 2.
- **Assistente/chat**, **perguntar** e **voz sintetizada (TTS)**: não fazem parte do ditado; o TTS nem está confirmado como função do VoiceInk (ER-19, ER-32).
- **Transcrição em tempo real (streaming)** e **transcrição de arquivo**: úteis, mas não essenciais ao fluxo central (ER-10, ER-12).
- **Painel de estatísticas** (ER-26).
- **Gravador no "notch"** do MacBook, ações do app Atalhos da Apple, comando local ("Local CLI").
- **Recursos comerciais do VoiceInk**: licença, afiliados, avisos remotos, convite para dar estrela no GitHub.

## Restrições e integrações

- **Licença e marca:** o VoiceInk é GPL-3.0. O Voice Desk **não reaproveita código, textos, ícones nem marca** dele; só o comportamento descrito no relatório. Qualquer reuso de código depende de parecer do Otávio antes.
- **Permissões do sistema:**
  - microfone;
  - acessibilidade, para colar e ler a seleção;
  - no Mac, as permissões equivalentes às do VoiceInk.

  O primeiro uso precisa guiar isso nas duas plataformas.
- **Dados sensíveis:** áudios e textos ditados podem conter informação de clientes. Ficam na máquina, com retenção configurável, e só saem para o provedor escolhido pelo BD. [PREMISSA] Na sincronização entre máquinas, só a configuração viaja, nunca os áudios.
- **Provedores externos:** usam a chave do próprio BD; custo de uso por conta dele.

## Riscos

- **Colar no programa em foco e ler a seleção funcionam de forma diferente em cada sistema.** Esse é o coração do produto. Reduzir: validar cedo, nas duas plataformas, com os programas que o BD mais usa.
- **Modelos locais rápidos no VoiceInk usam aceleração específica da Apple.** No Windows, o desempenho local pode ser pior. Reduzir: v1 com nuvem como padrão e local como opção.
- **Permissões e segurança do Windows** (antivírus, teclas simuladas): podem bloquear a colagem. Reduzir: testar em máquina Windows real do BD desde o início.
- **Escopo inflar** até a paridade total com o VoiceInk. Reduzir: manter o "fora por enquanto" acima e só abrir com aprovação do BD.

## Visão

Se a v1 cumprir o papel, o Voice Desk vira a ferramenta única de ditado do BD em qualquer máquina. Depois podem vir contexto da tela, troca de modo por site, transcrição em tempo real e, se fizer sentido, uso por outras pessoas da Going2.

## Premissas a confirmar

- [PREMISSA] O Voice Desk é para uso próprio do BD, sem venda nem licença (Para quem; Escopo).
- [PREMISSA] Sincronizar a configuração (modos, prompts, dicionário) entre Mac e Windows entra na v1 (Jornadas; Escopo).
- [PREMISSA] A transcrição local, sem internet, entra na v1 com pelo menos um modelo (Forma do produto).
- [PREMISSA] A v1 terá 2 ou 3 provedores de transcrição e 2 ou 3 de IA, começando pelos que o BD já usa (Groq, Ollama, Whisper local) (Escopo).
- [PREMISSA] Na sincronização, só a configuração viaja, nunca áudio nem histórico (Restrições).
- [PREMISSA] O critério de sucesso é usar o Voice Desk no lugar do VoiceInk por 30 dias; a meta de tempo até colar será definida no PRD (Critérios de sucesso).

## Perguntas em aberto

1. Quais provedores de transcrição e de IA o BD usa de fato no dia a dia? (BD; define a lista da v1 no PRD.)
2. O histórico (textos e áudios) também deve aparecer na outra máquina, ou fica em cada uma? (BD.)
3. O Windows de uso do BD é pessoal ou corporativo, com restrições de instalação ou antivírus? (BD; afeta o risco de colagem.)
