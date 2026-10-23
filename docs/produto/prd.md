# PRD: Voice Desk

> Status: rascunho para aprovação · Versão: 1 · Atualizado em: 2026-10-23
> Modo: criado do zero a partir do resumo aprovado
> Fontes: [resumo do produto](resumo.md) (aprovado no ENG-51), [relatório de engenharia reversa do VoiceInk 2.13](engenharia-reversa/relatorio.md), cards ENG-50 (card-mãe), ENG-51 e ENG-53

## 0. Como ler este documento

Este PRD é o contrato da v1 do Voice Desk. Ele serve à Nina (design), ao Otávio (arquitetura), à Helena (quebra em histórias) e à Marta (testes). Diz **o que** o produto faz e **por quê**. A tecnologia fica com o Otávio. A organização é: épicos `E-xx` → funcionalidades `F-xx` (cada uma cabe numa história) → requisitos `RF-xx.y` com consequência testável. A origem `ER-xx` aponta para o [mapa de funcionalidades do relatório](engenharia-reversa/relatorio.md#3-mapa-de-funcionalidades). Itens não confirmados pelo BD estão marcados com `[PREMISSA]` e listados na seção 11.

## 1. Visão

O Voice Desk é um app de **ditado por voz para desktop, no Mac e no Windows**. O BD aciona um atalho em qualquer programa, fala e solta. O texto aparece onde o cursor estava, já com o vocabulário dele e, se o modo pedir, refinado por IA: formal num e-mail, quase literal num chat.

Hoje o BD tem esse fluxo no Mac com o VoiceInk 2.13, que só existe para macOS e depende de recursos exclusivos da Apple. No Windows ele perde o atalho, os modos, o dicionário e o histórico. O Voice Desk reproduz o **comportamento** que importa no dia a dia, com a mesma experiência nas duas máquinas e uma **configuração única que vale nas duas**. Não reaproveita código, textos, ícones nem marca do VoiceInk, que é GPL-3.0.

A v1 é enxuta de propósito: o fluxo central (gravar → transcrever → limpar o texto → refinar → colar), modos, dicionário, histórico e sincronização da configuração. Ficam de fora a leitura da tela, a troca de modo por site, o assistente e o tempo real. Sucesso é o BD **trocar o VoiceInk pelo Voice Desk nas duas máquinas por 30 dias seguidos**.

## 2. Usuários

### 2.1 Perfis

| Perfil | Descrição | Principais tarefas | Permissões em alto nível |
|---|---|---|---|
| BD (usuário único) | Usa um Mac e um Windows no trabalho e dita muito texto: e-mails, chats, documentos. Já domina modos e dicionário do VoiceInk. Tem chaves próprias dos provedores. | Ditar em qualquer programa; manter modos, vocabulário e substituições; consultar o histórico. | Tudo. Não há login, contas nem papéis: o app roda na máquina dele, para ele. |

### 2.2 Não usuários (v1)

- Outras pessoas da Going2, equipes e clientes: não há multiusuário, convite nem administração. [PREMISSA]
- Usuários de celular ou de navegador: o produto é só desktop.
- Compradores: não há venda, licença, teste grátis nem ativação. [PREMISSA]

### 2.3 Jornadas principais

_Os programas citados são exemplos._

- **UJ-1. BD responde um e-mail no Windows ditando.** Contexto: o BD está no Outlook com uma mensagem aberta para responder. · Ponto de partida: cursor no corpo da resposta. · Caminho: (1) aciona o atalho global; (2) o gravador aparece e ele fala em tom informal; (3) aciona o atalho de novo; (4) o Voice Desk aplica o modo "E-mail", ativado porque o Outlook está em foco, e refina o texto com IA. · Momento de valor: o texto formal e corrigido aparece no corpo da mensagem em até 6 s. · Caso de erro: o provedor de IA não responde; o Voice Desk cola a transcrição sem refino e avisa.
- **UJ-2. BD manda uma mensagem rápida no Slack, no Mac.** Contexto: conversa com um cliente cujo nome é incomum. · Ponto de partida: cursor na caixa de mensagem. · Caminho: (1) atalho; (2) fala uma frase curta com o nome do cliente; (3) atalho. · Momento de valor: em até 3 s o texto quase literal aparece, com o nome escrito certo graças ao vocabulário, e o Enter é enviado porque o modo "Chat" está configurado assim. · Caso de erro: o BD desiste no meio e cancela com Esc duas vezes; nada é colado nem enviado ao provedor.
- **UJ-3. BD cadastra um termo no Mac e ele vale no Windows.** Contexto: um projeto novo tem uma sigla que a transcrição erra. · Caminho: (1) no Mac, adiciona a sigla ao vocabulário e uma substituição; (2) no dia seguinte, abre o Windows; (3) dita uma frase com a sigla. · Momento de valor: a sigla sai certa no Windows sem ele configurar nada de novo. · Caso de erro: o Windows estava sem internet; quando reconecta, a configuração chega sozinha.
- **UJ-4. BD recupera um ditado que não colou.** Contexto: ditou um parágrafo longo, mas o cursor não estava num campo de texto. · Caminho: (1) recebe o aviso de que o texto ficou na área de transferência; (2) clica no campo certo e cola; ou (3) abre o histórico, busca por uma palavra e copia o texto. · Momento de valor: nenhum ditado se perde. · Caso de erro: a transcrição falhou por falta de internet; o áudio está no histórico e ele manda transcrever de novo.
- **UJ-5. BD dita sem internet.** Contexto: está num voo, com o Mac. · Caminho: (1) o modo em uso aponta para a transcrição local; (2) dita normalmente. · Momento de valor: o texto é colado sem depender da rede. · Caso de erro: o modelo local não foi baixado; o Voice Desk avisa antes de gravar e indica onde baixar.
- **UJ-6. BD instala o Voice Desk numa máquina nova.** Contexto: primeiro uso no Windows. · Caminho: (1) instala e abre; (2) o primeiro uso guiado pede a permissão do microfone e as demais necessárias, a escolha do atalho e as chaves de API; (3) ativa a sincronização. · Momento de valor: em até 10 minutos ele faz o primeiro ditado, com os modos e o dicionário que já tinha no Mac. · Caso de erro: uma permissão é negada; o Voice Desk explica o que deixa de funcionar e como conceder depois.

## 3. Glossário

- **Ditado** — uma gravação, do atalho de início até parar ou cancelar, e tudo o que acontece com ela (transcrição, limpeza, refinamento, colagem). Cada ditado concluído vira um item do histórico.
- **Atalho global** — combinação de teclas que funciona em qualquer programa para iniciar e parar o ditado.
- **Gravador** — janela flutuante pequena, sempre visível durante o ditado, que mostra o estado (gravando, processando, erro) e o nível do áudio.
- **Programa em foco** — o programa que estava ativo quando o ditado começou; é onde o texto será colado e o que decide o modo automático.
- **Transcrição** — conversão do áudio do ditado em texto, feita por um provedor de transcrição.
- **Provedor** — serviço que faz a transcrição ou o refinamento. Na nuvem (Groq) ou local, na própria máquina (Whisper local, Ollama).
- **Chave de API** — credencial do BD num provedor de nuvem; o custo de uso é dele.
- **Cofre do sistema** — armazenamento seguro de credenciais do sistema operacional (no Mac e no Windows).
- **Limpeza do texto** — etapa sem IA que aplica substituições, remove palavras de preenchimento e formata o texto transcrito.
- **Vocabulário** — lista de palavras e nomes que o BD quer que a transcrição reconheça.
- **Substituição** — par "original → substituto" aplicado ao texto transcrito; pode ser ativada ou desativada.
- **Palavras de preenchimento** — muletas da fala ("é…", "hã", "tipo", "né") removidas na limpeza do texto.
- **Modo** — conjunto de regras de um ditado: nome, emoji, idioma, provedor de transcrição, se refina com IA, provedor e modelo de IA, prompt, uso de contexto, envio de Enter, programas associados, atalho e palavra-gatilho.
- **Modo padrão** — modo usado quando nenhum outro se aplica ao programa em foco.
- **Refinamento** — etapa opcional do modo que envia o texto limpo (e o contexto) a um modelo de IA com o prompt do modo e usa a resposta como texto final.
- **Prompt** — instrução em texto que diz à IA como refinar (ex.: "transforme em e-mail formal").
- **Contexto** — texto selecionado no programa em foco e conteúdo da área de transferência no início do ditado, enviados ao refinamento quando o modo permite.
- **Palavra-gatilho** — palavra falada no início do ditado que escolhe um modo; cada palavra pertence a um único modo.
- **Colagem** — inserção do texto final no programa em foco, onde estava o cursor.
- **Histórico** — registro local dos ditados concluídos: áudio, texto transcrito, texto final, modo, provedores, programa em foco, tempos e resultado da colagem.
- **Retenção** — prazo depois do qual o áudio e os itens do histórico são apagados automaticamente; prazos separados para áudio e para texto.
- **Configuração sincronizável** — modos, prompts, vocabulário, substituições, lista de palavras de preenchimento e preferências de texto. Não inclui chaves de API, histórico, áudio, atalhos, microfone nem modelos baixados.
- **Máquina** — uma instalação do Voice Desk (o Mac ou o Windows do BD).

## 4. Épicos e funcionalidades (índice)

| ID | Funcionalidade | Épico | Prioridade | Jornadas | Origem |
|---|---|---|---|---|---|
| F-01 | Gravar com atalho global e gravador flutuante | E-01 Ditado de ponta a ponta | Essencial | UJ-1, UJ-2 | ER-01, ER-02, ER-03 |
| F-02 | Transcrição na nuvem (Groq) | E-01 | Essencial | UJ-1, UJ-2 | ER-09, ER-11 |
| F-03 | Colar no programa em foco | E-01 | Essencial | UJ-1, UJ-2, UJ-4 | ER-13 |
| F-04 | Chaves de API no cofre do sistema | E-01 | Essencial | UJ-1, UJ-6 | ER-29 |
| F-05 | Ícone na bandeja ou barra de menus e iniciar com o sistema | E-01 | Essencial | UJ-6 | ER-27 |
| F-06 | Primeiro uso guiado | E-01 | Importante | UJ-6 | ER-28 |
| F-07 | Vocabulário personalizado | E-02 Texto do jeito do BD | Essencial | UJ-2, UJ-3 | ER-20, ER-11 |
| F-08 | Substituição de palavras | E-02 | Importante | UJ-3 | ER-21 |
| F-09 | Remoção de palavras de preenchimento e formatação | E-02 | Importante | UJ-2 | ER-14 |
| F-10 | Modos | E-03 Modos e refinamento por IA | Essencial | UJ-1, UJ-2 | ER-17 |
| F-11 | Refinamento por IA (Groq e Ollama) | E-03 | Essencial | UJ-1 | ER-15 |
| F-12 | Troca automática de modo pelo programa em foco | E-03 | Essencial | UJ-1, UJ-2 | ER-18 |
| F-13 | Contexto do texto selecionado e da área de transferência | E-03 | Importante | UJ-1 | ER-16 |
| F-14 | Atalho por modo | E-03 | Desejável | UJ-1 | ER-01 |
| F-15 | Palavra-gatilho falada | E-03 | Desejável | UJ-1 | ER-18 |
| F-16 | Histórico com busca e detalhe | E-04 Histórico | Importante | UJ-4 | ER-23 |
| F-17 | Retenção automática | E-04 | Importante | UJ-4 | ER-24 |
| F-18 | Transcrever de novo | E-04 | Desejável | UJ-4 | FL-05 |
| F-19 | Exportar o histórico em CSV | E-04 | Desejável | UJ-4 | ER-25 |
| F-20 | Sincronização da configuração entre máquinas | E-05 Mesma configuração nas duas máquinas | Essencial | UJ-3, UJ-6 | resumo ("Mudar"); substitui ER-22 |
| F-21 | Exportar e importar a configuração | E-05 | Importante | UJ-3 | ER-25 |
| F-22 | Transcrição local (Whisper) | E-06 Uso sem internet | Importante | UJ-5 | ER-08 |
| F-23 | Escolha do microfone | E-07 Conforto na gravação | Importante | UJ-1, UJ-2 | ER-04 |
| F-24 | Sons de início e fim | E-07 | Desejável | UJ-2 | ER-05 |
| F-25 | Pausar a mídia durante o ditado | E-07 | Desejável | UJ-1 | ER-06 |
| F-26 | Descartar ditado sem fala | E-07 | Desejável | UJ-2 | ER-07 |

**Épicos** (na ordem sugerida de entrega)

- **E-01 Ditado de ponta a ponta** — o BD dita em qualquer programa, no Mac e no Windows, e o texto é colado onde estava o cursor. Ao fim do épico o produto já é utilizável. Funcionalidades: F-01 a F-06.
- **E-02 Texto do jeito do BD** — nomes, siglas e muletas tratados sem IA, em todo ditado. Funcionalidades: F-07, F-08, F-09.
- **E-03 Modos e refinamento por IA** — o texto sai no formato certo para cada programa. Funcionalidades: F-10 a F-15.
- **E-04 Histórico** — nenhum ditado se perde, e os dados não ficam guardados além do prazo. Funcionalidades: F-16 a F-19.
- **E-05 Mesma configuração nas duas máquinas** — configurou uma vez, vale no Mac e no Windows. Funcionalidades: F-20, F-21.
- **E-06 Uso sem internet** — ditar sem rede e sem enviar o áudio para fora. Funcionalidade: F-22.
- **E-07 Conforto na gravação** — microfone certo, sons e mídia pausada. Funcionalidades: F-23 a F-26.

## 5. Funcionalidades

### F-01 Gravar com atalho global e gravador flutuante

**Épico:** E-01 · **Prioridade:** Essencial · **Realiza:** UJ-1, UJ-2 · **Origem:** ER-01, ER-02, ER-03
**Descrição:** com o Voice Desk aberto (mesmo minimizado na bandeja), o BD aciona o atalho global em qualquer programa. O gravador aparece sem tirar o foco do programa, indica que está gravando e mostra o nível do áudio. O mesmo atalho para a gravação e inicia o processamento. Esc duas vezes cancela. O ditado guarda qual era o programa em foco no início.

**Requisitos funcionais**
- **RF-01.1** O BD pode definir o atalho global nas configurações, com pelo menos uma tecla modificadora.
  - Consequência testável: um atalho já usado pelo sistema ou só com uma letra é recusado, com mensagem; o atalho salvo passa a funcionar sem reiniciar o app.
- **RF-01.2** O BD pode iniciar o ditado pelo atalho global com qualquer programa em foco.
  - Consequência testável: o gravador aparece em até 300 ms e o programa em foco continua em foco (o cursor não sai do campo).
- **RF-01.3** O BD pode parar o ditado acionando o atalho de novo.
  - Consequência testável: o gravador passa a "processando" e, ao terminar, some sozinho.
- **RF-01.4** O BD pode cancelar o ditado apertando Esc duas vezes em até 1,5 s enquanto grava.
  - Consequência testável: nada é colado, nada é enviado a provedor e nada entra no histórico. Esc uma vez só não cancela e chega normalmente ao programa em foco.
- **RF-01.5** O ditado para sozinho ao atingir 10 minutos de gravação. [PREMISSA]
  - Consequência testável: aos 9 min 30 s o gravador mostra um aviso; aos 10 min o ditado segue para o processamento como se o BD tivesse parado.
- **RF-01.6** Enquanto um ditado está em processamento, o atalho global não inicia outro.
  - Consequência testável: acionar o atalho durante o processamento não abre nova gravação, e o gravador indica "aguarde".

**Critérios de aceite (alto nível)**
- Dado o Voice Desk na bandeja e o Bloco de Notas em foco, quando o BD aciona o atalho, fala e aciona de novo, então o gravador aparece, mostra o nível do áudio, passa a "processando" e some.
- Dado um ditado em andamento, quando o BD aperta Esc duas vezes em 1 s, então o gravador some e nada é colado nem registrado.
- Dado o microfone sem permissão, quando o BD aciona o atalho, então o gravador mostra "sem acesso ao microfone" com o caminho para conceder, e nada é gravado.

**Fora do escopo desta funcionalidade:** modo "segurar para falar" (gravar só enquanto o atalho estiver pressionado); gravador no entalhe ("notch") do MacBook; texto ao vivo durante a fala.
**Notas / perguntas:** o atalho padrão sugerido no primeiro uso é definido pela Nina, diferente para Mac e Windows.

### F-02 Transcrição na nuvem (Groq)

**Épico:** E-01 · **Prioridade:** Essencial · **Realiza:** UJ-1, UJ-2 · **Origem:** ER-09, ER-11
**Descrição:** ao parar o ditado, o áudio vai ao provedor de transcrição do modo ativo. Na v1, o provedor de nuvem é o Groq, com a chave do BD. O idioma vem do modo: português do Brasil, inglês ou automático.

**Requisitos funcionais**
- **RF-02.1** O BD pode escolher, por modo, o provedor de transcrição (Groq ou Whisper local, ver F-22) e o modelo do provedor.
  - Consequência testável: ditados feitos em modos diferentes registram no histórico o provedor e o modelo de cada modo.
- **RF-02.2** O BD pode escolher, por modo, o idioma do ditado: português do Brasil, inglês ou detecção automática.
  - Consequência testável: com o idioma "inglês", uma frase falada em inglês é transcrita em inglês, sem tradução.
- **RF-02.3** Quando a transcrição falha (sem internet, chave ausente ou inválida, erro do provedor ou mais de 30 s sem resposta), o Voice Desk não cola nada, guarda o áudio no histórico com o estado "falhou" e mostra o motivo.
  - Consequência testável: com a rede desligada, o ditado termina com aviso "sem conexão com o provedor", e o item aparece no histórico com áudio e estado "falhou".

**Critérios de aceite (alto nível)**
- Dado um modo com Groq e chave válida, quando o BD dita 20 s em português, então o texto transcrito é colado no programa em foco.
- Dado um modo com Groq sem chave cadastrada, quando o BD inicia o ditado, então o gravador avisa "cadastre a chave do Groq" antes de gravar, com atalho para a tela de chaves.

**Fora do escopo desta funcionalidade:** outros provedores de nuvem; transcrição em tempo real; transcrição de arquivo.

### F-03 Colar no programa em foco

**Épico:** E-01 · **Prioridade:** Essencial · **Realiza:** UJ-1, UJ-2, UJ-4 · **Origem:** ER-13
**Descrição:** o texto final do ditado é inserido no programa em foco, onde estava o cursor, como se o BD tivesse colado. Depois disso, o conteúdo que o BD tinha na área de transferência volta para ela. Se o modo pedir, o Voice Desk envia Enter depois de colar. Se não der para colar, o texto fica na área de transferência e o BD é avisado.

**Requisitos funcionais**
- **RF-03.1** O Voice Desk cola o texto final no programa em foco ao fim do processamento.
  - Consequência testável: nos programas da lista de compatibilidade (RNF-05), o texto aparece na posição do cursor, inclusive com acentos, emojis e quebras de linha.
- **RF-03.2** Depois de colar, o Voice Desk devolve à área de transferência o conteúdo que estava nela antes do ditado.
  - Consequência testável: com "abc" copiado antes do ditado, Ctrl+V ou Cmd+V depois da colagem cola "abc".
- **RF-03.3** O BD pode ligar, por modo, o envio de Enter depois de colar.
  - Consequência testável: num modo com Enter ligado, a mensagem ditada no chat é enviada sem o BD tocar no teclado; com Enter desligado, ela fica na caixa de texto.
- **RF-03.4** Quando o Voice Desk não consegue colar (permissão de acessibilidade negada ou o programa em foco recusa a colagem), o texto final fica na área de transferência e aparece um aviso "Texto copiado; cole com Ctrl+V/Cmd+V".
  - Consequência testável: com a permissão de acessibilidade negada, o ditado termina com o aviso e o texto pode ser colado manualmente; o item no histórico registra "não colado".
- **RF-03.5** Cada item do histórico registra o resultado da colagem: "colado", "não colado" ou "cancelado".
  - Consequência testável: os três estados aparecem no detalhe do histórico (F-16) conforme o caso.

**Critérios de aceite (alto nível)**
- Dado o cursor no corpo de um e-mail no Outlook (Windows), quando o ditado termina, então o texto aparece no corpo e a área de transferência volta ao conteúdo anterior.
- Dado o mesmo cenário no Mail ou no Slack (Mac), quando o ditado termina, então o comportamento é o mesmo.
- Dado nenhum campo de texto em foco, quando o ditado termina, então o texto fica na área de transferência e o aviso aparece.

**Fora do escopo desta funcionalidade:** digitar o texto letra a letra como alternativa à colagem (avaliar só se a validação no Windows mostrar necessidade).
**Notas / perguntas:** é o maior risco do produto (resumo, "Riscos"). Validar cedo, nas duas plataformas, com os programas da RNF-05.

### F-04 Chaves de API no cofre do sistema

**Épico:** E-01 · **Prioridade:** Essencial · **Realiza:** UJ-1, UJ-6 · **Origem:** ER-29
**Descrição:** o BD cadastra as chaves de API dos provedores de nuvem (na v1, Groq) numa tela própria. Elas ficam guardadas no cofre do sistema de cada máquina, nunca em arquivo comum, e não viajam na sincronização nem na exportação.

**Requisitos funcionais**
- **RF-04.1** O BD pode cadastrar, trocar e apagar a chave de cada provedor de nuvem.
  - Consequência testável: depois de apagar a chave do Groq, um ditado num modo com Groq avisa "cadastre a chave do Groq".
- **RF-04.2** Ao salvar uma chave, o Voice Desk testa a chave no provedor e mostra "válida" ou "inválida" em até 10 s.
  - Consequência testável: uma chave inventada é marcada "inválida", e a mensagem diz o motivo informado pelo provedor.
- **RF-04.3** A chave nunca aparece inteira na tela depois de salva: aparecem só os 4 últimos caracteres.
  - Consequência testável: a tela de chaves mostra "••••abcd"; a chave não aparece em logs, na exportação (F-21) nem na sincronização (F-20).
- **RF-04.4** A tela de cada provedor tem um link para a página do provedor onde se obtém a chave.
  - Consequência testável: o link do Groq abre o console do Groq no navegador padrão.

**Critérios de aceite (alto nível)**
- Dado o BD na tela de chaves, quando cola uma chave válida do Groq e salva, então aparece "válida" e os ditados com Groq passam a funcionar.
- Dado um arquivo exportado (F-21) ou a configuração sincronizada (F-20), quando alguém procura a chave nele, então ela não está lá.

### F-05 Ícone na bandeja ou barra de menus e iniciar com o sistema

**Épico:** E-01 · **Prioridade:** Essencial · **Realiza:** UJ-6 · **Origem:** ER-27
**Descrição:** o Voice Desk vive como ícone na barra de menus (Mac) ou na bandeja (Windows), sem janela principal aberta. O menu do ícone dá acesso rápido ao essencial. Pode iniciar junto com o sistema.

**Requisitos funcionais**
- **RF-05.1** O menu do ícone oferece: iniciar ou parar ditado, escolher o modo padrão, abrir o histórico, abrir as configurações e sair.
  - Consequência testável: cada item do menu executa a ação correspondente; escolher um modo padrão no menu muda o modo usado quando nenhum programa associado está em foco (F-12).
- **RF-05.2** O BD pode ligar ou desligar "iniciar com o sistema"; o padrão é ligado.
  - Consequência testável: com a opção ligada, depois de reiniciar a máquina e entrar na conta, o ícone aparece em até 5 s, sem abrir janela.
- **RF-05.3** O ícone indica o estado atual: parado, gravando, processando ou erro.
  - Consequência testável: durante a gravação o ícone muda de aparência e volta ao normal ao terminar.

**Critérios de aceite (alto nível)**
- Dado o Voice Desk instalado com "iniciar com o sistema" ligado, quando o BD reinicia a máquina, então o ícone aparece e o atalho global funciona sem abrir o app.
- Dado o BD clicando em "Sair", quando confirma, então o ícone some e o atalho global deixa de funcionar.

### F-06 Primeiro uso guiado

**Épico:** E-01 · **Prioridade:** Importante · **Realiza:** UJ-6 · **Origem:** ER-28
**Descrição:** na primeira abertura em cada máquina, um passo a passo leva o BD pelas permissões do sistema (microfone e acessibilidade, mais as equivalentes no Mac), pela escolha do atalho, pela chave do Groq e pela ativação da sincronização. Termina com um ditado de teste.

**Requisitos funcionais**
- **RF-06.1** O primeiro uso pede cada permissão necessária, explica em uma frase para que serve e mostra se foi concedida.
  - Consequência testável: com a permissão de microfone negada, o passo mostra "negada", o que deixa de funcionar e como conceder nas configurações do sistema.
- **RF-06.2** O BD pode pular qualquer passo e voltar a ele depois pelas configurações.
  - Consequência testável: pulando a chave do Groq, o primeiro uso termina, e a tela de chaves continua acessível pelo menu.
- **RF-06.3** O último passo é um ditado de teste numa caixa de texto do próprio primeiro uso.
  - Consequência testável: o texto ditado aparece na caixa; se falhar, aparece o motivo (F-02 RF-02.3).

**Critérios de aceite (alto nível)**
- Dado um Windows sem o Voice Desk, quando o BD instala e segue o primeiro uso concedendo tudo, então em até 10 minutos ele conclui o ditado de teste.
- Dado o primeiro uso já concluído, quando o BD reabre o app, então o primeiro uso não aparece de novo.

### F-07 Vocabulário personalizado

**Épico:** E-02 · **Prioridade:** Essencial · **Realiza:** UJ-2, UJ-3 · **Origem:** ER-20, ER-11
**Descrição:** lista de palavras, nomes e siglas que o provedor de transcrição deve reconhecer. O Voice Desk envia o vocabulário como dica a cada transcrição. Há uma forma rápida de adicionar uma palavra sem abrir as configurações.

**Requisitos funcionais**
- **RF-07.1** O BD pode adicionar, editar, remover e buscar palavras no vocabulário.
  - Consequência testável: uma palavra adicionada aparece na lista e na busca; uma palavra duplicada (sem diferenciar maiúsculas) é recusada com aviso.
- **RF-07.2** Todo ditado envia o vocabulário como dica ao provedor de transcrição.
  - Consequência testável: um nome incomum que a transcrição errava sem o vocabulário passa a sair escrito como cadastrado em pelo menos 4 de 5 ditados de teste.
- **RF-07.3** O BD pode adicionar uma palavra ao vocabulário pelo menu do ícone, numa caixa rápida, sem abrir as configurações.
  - Consequência testável: a palavra digitada na caixa rápida aparece na lista do vocabulário.
- **RF-07.4** O vocabulário aceita até 500 palavras. [PREMISSA]
  - Consequência testável: a 501ª palavra é recusada com a mensagem "limite de 500 palavras".

**Critérios de aceite (alto nível)**
- Dado "Going2" no vocabulário, quando o BD dita "mandei para a Going2", então o texto colado traz "Going2".
- Dado o vocabulário vazio, quando o BD dita, então a transcrição funciona normalmente, sem dica.

**Notas / perguntas:** o limite real de dica por provedor é verificado pelo Otávio; se for menor que 500 palavras, o PRD é ajustado.

### F-08 Substituição de palavras

**Épico:** E-02 · **Prioridade:** Importante · **Realiza:** UJ-3 · **Origem:** ER-21
**Descrição:** pares "original → substituto" aplicados ao texto transcrito, antes do refinamento. Servem para corrigir erros recorrentes e para atalhos de texto (ex.: "meu e-mail" → endereço completo).

**Requisitos funcionais**
- **RF-08.1** O BD pode criar, editar, remover, ativar e desativar substituições.
  - Consequência testável: uma substituição desativada não é aplicada; reativada, volta a ser.
- **RF-08.2** A substituição vale para palavras ou expressões inteiras, sem diferenciar maiúsculas e minúsculas.
  - Consequência testável: com "vois desk → Voice Desk", o texto "o Vois Desk" vira "o Voice Desk"; a palavra "voisdesking" não é alterada.
- **RF-08.3** Substituições são aplicadas antes do refinamento por IA (F-11).
  - Consequência testável: no histórico, o texto enviado ao refinamento já contém o substituto.

**Critérios de aceite (alto nível)**
- Dada a substituição "pix" → "PIX" ativa, quando o BD dita "manda um pix", então o texto colado é "manda um PIX".
- Dado um original vazio, quando o BD tenta salvar, então o app recusa com aviso.

### F-09 Remoção de palavras de preenchimento e formatação

**Épico:** E-02 · **Prioridade:** Importante · **Realiza:** UJ-2 · **Origem:** ER-14
**Descrição:** limpeza sem IA, ligada ou desligada por modo: tira as palavras de preenchimento de uma lista editável e formata o texto (primeira letra maiúscula, pontuação final, espaços duplos removidos).

**Requisitos funcionais**
- **RF-09.1** O BD pode ligar ou desligar, por modo, a remoção de palavras de preenchimento e a formatação, separadamente.
  - Consequência testável: com a remoção desligada, "é… eu acho que sim" é colado igual à transcrição.
- **RF-09.2** O BD pode editar a lista de palavras de preenchimento; a lista vem preenchida com as mais comuns em português e em inglês.
  - Consequência testável: com "tipo" na lista e a remoção ligada, "é tipo assim" vira "É assim." (com formatação ligada).
- **RF-09.3** A formatação deixa a primeira letra maiúscula, põe ponto final se a frase não terminar em pontuação e remove espaços duplos.
  - Consequência testável: a transcrição "olá  tudo bem" vira "Olá tudo bem."

**Critérios de aceite (alto nível)**
- Dado o modo "Chat" com remoção e formatação ligadas, quando o BD dita "hã… chego em dez minutos", então o texto colado é "Chego em dez minutos."
- Dado o modo "Ditado literal" com tudo desligado, quando o BD dita a mesma frase, então o texto colado é a transcrição sem alteração.

### F-10 Modos

**Épico:** E-03 · **Prioridade:** Essencial · **Realiza:** UJ-1, UJ-2 · **Origem:** ER-17
**Descrição:** o BD cria modos para cada tipo de texto. O Voice Desk vem com três modos iniciais editáveis: "Ditado" (sem refinamento, modo padrão), "E-mail" (refinamento formal) e "Chat" (limpeza, sem refinamento, com Enter ligado). Os prompts iniciais são escritos pelo time, sem copiar os do VoiceInk.

**Requisitos funcionais**
- **RF-10.1** O BD pode criar, editar, duplicar e excluir modos com: nome, emoji, idioma, provedor e modelo de transcrição, limpeza (F-09), refinamento ligado ou desligado, provedor e modelo de IA, prompt, uso de contexto (F-13) e envio de Enter (F-03).
  - Consequência testável: um modo novo salvo aparece no menu do ícone e pode ser escolhido como modo padrão.
- **RF-10.2** Sempre existe exatamente um modo padrão; o modo padrão não pode ser excluído enquanto for o padrão.
  - Consequência testável: o botão de excluir do modo padrão fica desativado, com a explicação "escolha outro modo padrão antes".
- **RF-10.3** O gravador mostra o nome e o emoji do modo que será aplicado ao ditado.
  - Consequência testável: com o Outlook associado ao modo "E-mail", o gravador mostra "E-mail" ao iniciar o ditado no Outlook.
- **RF-10.4** O nome do modo é único, com até 40 caracteres; o prompt aceita até 4.000 caracteres.
  - Consequência testável: salvar dois modos com o mesmo nome é recusado; um prompt com 4.001 caracteres é recusado com aviso.

**Critérios de aceite (alto nível)**
- Dada a primeira abertura, quando o BD abre a lista de modos, então vê "Ditado" (padrão), "E-mail" e "Chat", todos editáveis.
- Dado o modo "Chat" com Enter ligado, quando o BD dita no Slack, então o texto é colado e enviado.

### F-11 Refinamento por IA (Groq e Ollama)

**Épico:** E-03 · **Prioridade:** Essencial · **Realiza:** UJ-1 · **Origem:** ER-15
**Descrição:** se o modo tem refinamento ligado, o texto limpo (e o contexto, se o modo permitir) vai ao provedor de IA do modo com o prompt do modo, e a resposta vira o texto final. Na v1, os provedores de IA são o Groq (nuvem, com a chave do BD) e o Ollama (local, sem chave). Se o refinamento falhar, o ditado não se perde.

**Requisitos funcionais**
- **RF-11.1** O BD pode escolher, por modo, o provedor de IA (Groq ou Ollama) e o modelo, a partir da lista de modelos disponíveis no provedor.
  - Consequência testável: a lista de modelos do Groq aparece depois de cadastrar a chave; a do Ollama aparece quando o Ollama está rodando na máquina.
- **RF-11.2** O texto final é só a resposta da IA, sem comentários extras sobre o pedido nem citação do contexto quando ele não for relevante.
  - Consequência testável: em 10 ditados de teste com o modo "E-mail", nenhum texto colado começa com frases como "Aqui está o seu e-mail".
- **RF-11.3** Quando o refinamento falha (erro do provedor, chave inválida, Ollama fechado ou mais de 15 s sem resposta), o Voice Desk cola o texto limpo sem refinamento e avisa "colado sem refinamento: <motivo>".
  - Consequência testável: com o Ollama fechado, um ditado num modo com Ollama cola o texto limpo e mostra o aviso; o histórico registra o refinamento como "falhou".
- **RF-11.4** O histórico guarda, para cada ditado refinado, o texto antes e depois do refinamento, o provedor, o modelo e o tempo gasto.
  - Consequência testável: o detalhe do item no histórico (F-16) mostra os dois textos lado a lado.

**Critérios de aceite (alto nível)**
- Dado o modo "E-mail" com Groq, quando o BD dita "fala pro João que a reunião mudou pra quinta", então o texto colado é um parágrafo de e-mail formal com essa informação.
- Dado o Ollama fechado, quando o BD dita num modo com Ollama, então o texto limpo é colado em até 20 s e aparece o aviso.

**Fora do escopo desta funcionalidade:** outros provedores de IA (OpenAI, Anthropic, Gemini, OpenRouter, LM Studio e outros); comando de terminal local ("Local CLI"); modelo próprio embutido no app.

### F-12 Troca automática de modo pelo programa em foco

**Épico:** E-03 · **Prioridade:** Essencial · **Realiza:** UJ-1, UJ-2 · **Origem:** ER-18 (sem a troca por site)
**Descrição:** o BD associa programas a modos. No início do ditado, o Voice Desk vê o programa em foco e aplica o modo associado; se não houver associação, aplica o modo padrão. Como a configuração é sincronizada, um modo pode ter programas do Mac e do Windows ao mesmo tempo (ex.: Mail e Outlook no modo "E-mail").

**Requisitos funcionais**
- **RF-12.1** O BD pode associar um ou mais programas a um modo, escolhendo da lista de programas abertos ou instalados naquela máquina.
  - Consequência testável: o Outlook associado no Windows aparece na lista de programas do modo também no Mac, identificado como programa do Windows.
- **RF-12.2** Cada programa pertence a no máximo um modo.
  - Consequência testável: ao associar ao modo "Chat" um programa que já estava no "E-mail", o app pergunta se deve movê-lo; confirmado, ele sai do "E-mail".
- **RF-12.3** A ordem de escolha do modo, do mais forte para o mais fraco, é: palavra-gatilho (F-15), atalho do modo (F-14), programa em foco e modo padrão.
  - Consequência testável: com o Slack associado ao "Chat", um ditado iniciado pelo atalho do modo "E-mail" usa o "E-mail".

**Critérios de aceite (alto nível)**
- Dado o Outlook associado ao "E-mail", quando o BD inicia um ditado com o Outlook em foco, então o gravador mostra "E-mail" e o refinamento formal é aplicado.
- Dado um programa sem associação, quando o BD dita, então o modo padrão é aplicado.

**Fora do escopo desta funcionalidade:** troca de modo pelo site aberto no navegador (fase 2).

### F-13 Contexto do texto selecionado e da área de transferência

**Épico:** E-03 · **Prioridade:** Importante · **Realiza:** UJ-1 · **Origem:** ER-16 (sem o texto da janela e sem o OCR da tela)
**Descrição:** se o modo permitir, o refinamento recebe como contexto o texto selecionado no programa em foco e o conteúdo de texto da área de transferência no início do ditado. Exemplo: o BD seleciona o e-mail recebido e dita "responde dizendo que aceito".

**Requisitos funcionais**
- **RF-13.1** O BD pode ligar, por modo, o uso do texto selecionado e o uso da área de transferência como contexto, separadamente; o padrão é desligado.
  - Consequência testável: com os dois desligados, o histórico mostra que nenhum contexto foi enviado.
- **RF-13.2** O contexto enviado é limitado a 8.000 caracteres por fonte; o excesso é cortado. [PREMISSA]
  - Consequência testável: uma seleção de 10.000 caracteres chega ao refinamento com 8.000, e o histórico indica "contexto cortado".
- **RF-13.3** Sem permissão de acessibilidade, o texto selecionado é ignorado e o ditado segue sem ele.
  - Consequência testável: com a permissão negada, o ditado é concluído e o histórico registra "seleção indisponível".

**Critérios de aceite (alto nível)**
- Dado um e-mail recebido selecionado e o modo "E-mail" com seleção ligada, quando o BD dita "responde que aceito a proposta", então o texto colado é uma resposta coerente com o e-mail selecionado.
- Dado o modo com contexto desligado, quando o BD dita com texto selecionado, então a seleção não é enviada ao provedor.

**Notas / perguntas:** atenção à colagem: se houver texto selecionado, a colagem o substitui (comportamento padrão de colar). O detalhe de interação fica com a Nina.

### F-14 Atalho por modo

**Épico:** E-03 · **Prioridade:** Desejável · **Realiza:** UJ-1 · **Origem:** ER-01
**Descrição:** além do atalho global, o BD pode dar a um modo um atalho próprio, que inicia o ditado já naquele modo, sem depender do programa em foco.

**Requisitos funcionais**
- **RF-14.1** O BD pode definir um atalho opcional por modo, diferente do atalho global e dos atalhos de outros modos.
  - Consequência testável: um atalho repetido é recusado com aviso indicando qual modo já o usa.
- **RF-14.2** Iniciar pelo atalho do modo aplica aquele modo; acionar de novo o mesmo atalho para o ditado.
  - Consequência testável: com o Slack em foco (associado ao "Chat"), o atalho do "E-mail" grava no modo "E-mail".

**Critérios de aceite (alto nível)**
- Dado o atalho do "E-mail" definido, quando o BD o aciona em qualquer programa, então o gravador mostra "E-mail".
- Dado um modo sem atalho próprio, quando o BD abre a configuração do modo, então o campo de atalho está vazio e o modo só é usado pelas outras regras.

**Notas / perguntas:** os atalhos são por máquina e não sincronizam, porque as teclas diferem entre Mac e Windows.

### F-15 Palavra-gatilho falada

**Épico:** E-03 · **Prioridade:** Desejável · **Realiza:** UJ-1 · **Origem:** ER-18
**Descrição:** o BD dá a um modo uma ou mais palavras-gatilho. Se o ditado começar com uma delas, o modo é aplicado e a palavra é retirada do texto.

**Requisitos funcionais**
- **RF-15.1** O BD pode cadastrar palavras-gatilho por modo; cada palavra pertence a um único modo.
  - Consequência testável: cadastrar no "Chat" uma palavra que já é do "E-mail" é recusado com aviso.
- **RF-15.2** Se as primeiras 3 palavras do texto transcrito contêm uma palavra-gatilho, o modo dela é aplicado e a palavra é removida do texto final.
  - Consequência testável: com "e-mail" como gatilho do "E-mail", o ditado "e-mail, avisa o time que a entrega atrasou" é refinado no modo "E-mail" e o texto final não começa com "e-mail".

**Critérios de aceite (alto nível)**
- Dado o gatilho "tradução" no modo "Traduzir", quando o BD dita "tradução: bom dia a todos", então o texto colado é a tradução de "bom dia a todos".
- Dado um ditado sem gatilho, quando termina, então o modo é escolhido pelas demais regras (RF-12.3).

**Notas / perguntas:** como a palavra-gatilho só é conhecida depois da transcrição, o idioma e o provedor de transcrição usados são os do modo escolhido pelas outras regras; só a limpeza e o refinamento seguem o modo do gatilho.

### F-16 Histórico com busca e detalhe

**Épico:** E-04 · **Prioridade:** Importante · **Realiza:** UJ-4 · **Origem:** ER-23
**Descrição:** janela com a lista dos ditados concluídos daquela máquina, do mais recente para o mais antigo, com busca por texto e detalhe de cada item. Ditados cancelados não entram.

**Requisitos funcionais**
- **RF-16.1** O BD pode ver a lista do histórico com data e hora, modo, início do texto final e estado (colado, não colado, falhou).
  - Consequência testável: um ditado feito agora aparece no topo da lista em até 2 s após o fim do processamento.
- **RF-16.2** O BD pode buscar no texto transcrito e no texto final.
  - Consequência testável: com 5.000 itens, a busca por uma palavra devolve os itens que a contêm em até 1 s.
- **RF-16.3** O BD pode abrir o detalhe de um item: texto transcrito, texto final, modo, provedores e modelos, programa em foco, duração do áudio, tempo de transcrição, tempo de refinamento, resultado da colagem e contexto enviado (se houver).
  - Consequência testável: todos esses campos aparecem no detalhe de um ditado refinado.
- **RF-16.4** O BD pode copiar o texto final ou o transcrito, ouvir o áudio (enquanto não tiver sido apagado pela retenção) e excluir o item.
  - Consequência testável: depois de excluir, o item e o áudio somem da máquina e não voltam após reiniciar o app.

**Critérios de aceite (alto nível)**
- Dado um ditado que não colou, quando o BD abre o histórico e busca uma palavra dele, então encontra o item e copia o texto final.
- Dado um item com áudio já apagado pela retenção, quando o BD abre o detalhe, então o texto aparece e o áudio mostra "apagado pela retenção".

**Fora do escopo desta funcionalidade:** histórico da outra máquina (o histórico não sincroniza [PREMISSA]); painel de estatísticas.

### F-17 Retenção automática

**Épico:** E-04 · **Prioridade:** Importante · **Realiza:** UJ-4 · **Origem:** ER-24
**Descrição:** o BD define por quanto tempo guardar os áudios e, separadamente, os itens do histórico. O que passa do prazo é apagado sozinho.

**Requisitos funcionais**
- **RF-17.1** O BD pode escolher o prazo de retenção do áudio e o do histórico entre: 1 dia, 7 dias, 30 dias, 90 dias ou "não apagar". Os padrões são: áudio 7 dias e histórico 90 dias. [PREMISSA]
  - Consequência testável: a tela de configurações mostra os padrões numa instalação nova.
- **RF-17.2** A limpeza roda ao abrir o app e depois a cada 24 h, apagando o que passou do prazo.
  - Consequência testável: com o áudio em 1 dia, um áudio de 2 dias atrás some na próxima limpeza, e o texto do item continua no histórico.
- **RF-17.3** O BD pode apagar todo o histórico e todos os áudios de uma vez, com confirmação.
  - Consequência testável: depois de confirmar, a lista fica vazia e a pasta de áudios do app fica sem arquivos.

**Critérios de aceite (alto nível)**
- Dado o histórico em 7 dias, quando passa a limpeza diária, então nenhum item com mais de 7 dias permanece.
- Dado o prazo do áudio menor que o do histórico, quando o áudio vence, então o item continua com o texto e sem áudio.

### F-18 Transcrever de novo

**Épico:** E-04 · **Prioridade:** Desejável · **Realiza:** UJ-4 · **Origem:** FL-05 do relatório
**Descrição:** a partir de um item do histórico que ainda tem áudio, o BD manda processar de novo, no mesmo modo ou em outro. Útil quando a transcrição falhou por falta de rede.

**Requisitos funcionais**
- **RF-18.1** O BD pode transcrever de novo um item com áudio, escolhendo o modo.
  - Consequência testável: o resultado é gravado no mesmo item, que passa do estado "falhou" para "não colado", com o texto novo.
- **RF-18.2** O resultado de "transcrever de novo" vai para a área de transferência, não é colado em nenhum programa.
  - Consequência testável: depois do reprocessamento, Ctrl+V/Cmd+V cola o texto novo.

**Critérios de aceite (alto nível)**
- Dado um item "falhou" por falta de internet, quando o BD reconecta e manda transcrever de novo, então o item recebe o texto e o estado muda.
- Dado um item sem áudio, quando o BD abre o detalhe, então a ação "transcrever de novo" fica indisponível, com a explicação.

### F-19 Exportar o histórico em CSV

**Épico:** E-04 · **Prioridade:** Desejável · **Realiza:** UJ-4 · **Origem:** ER-25
**Descrição:** o BD exporta o histórico (só texto, sem áudio) para uma planilha.

**Requisitos funcionais**
- **RF-19.1** O BD pode exportar o histórico inteiro ou um intervalo de datas para um arquivo CSV em UTF-8.
  - Consequência testável: o arquivo abre no Excel e no Numbers com acentos corretos e uma linha por item.
- **RF-19.2** O CSV traz: data e hora, modo, programa em foco, texto transcrito, texto final, provedores, duração do áudio e estado.
  - Consequência testável: as colunas aparecem no cabeçalho do arquivo, nessa ordem.

**Critérios de aceite (alto nível)**
- Dado um histórico com 1.000 itens, quando o BD exporta tudo, então o arquivo é gerado em até 5 s.
- Dado um histórico vazio, quando o BD tenta exportar, então o app avisa "nada para exportar" e não gera arquivo.

### F-20 Sincronização da configuração entre máquinas

**Épico:** E-05 · **Prioridade:** Essencial [PREMISSA] · **Realiza:** UJ-3, UJ-6 · **Origem:** resumo ("Mudar"); substitui a sincronização do dicionário pelo iCloud (ER-22)
**Descrição:** o BD liga a sincronização em cada máquina. A partir daí, a configuração sincronizável (glossário) é a mesma no Mac e no Windows. Chaves de API, histórico, áudio, atalhos, microfone e modelos baixados ficam em cada máquina.

**Requisitos funcionais**
- **RF-20.1** O BD pode ligar a sincronização numa máquina e conectar a segunda máquina à mesma configuração.
  - Consequência testável: ao conectar o Windows a uma configuração que já existe no Mac, os modos, o vocabulário e as substituições do Mac aparecem no Windows em até 1 minuto.
- **RF-20.2** Uma alteração na configuração sincronizável feita numa máquina aparece na outra em até 5 minutos, com as duas conectadas à internet.
  - Consequência testável: uma palavra adicionada ao vocabulário no Mac aparece no Windows em até 5 minutos, sem ação do BD.
- **RF-20.3** Alterações feitas sem internet são enviadas quando a máquina reconecta.
  - Consequência testável: uma substituição criada com o Windows sem rede chega ao Mac até 5 minutos depois de reconectar.
- **RF-20.4** Se o mesmo item (um modo, uma palavra, uma substituição) for alterado nas duas máquinas antes de sincronizar, vale a alteração mais recente. [PREMISSA]
  - Consequência testável: com o prompt do "E-mail" alterado no Mac às 10h00 e no Windows às 10h01, sem rede, as duas máquinas ficam com o texto das 10h01 depois de reconectar.
- **RF-20.5** O BD vê o estado da sincronização (em dia, sincronizando, sem conexão, erro) e a data da última sincronização.
  - Consequência testável: com a rede desligada, o estado muda para "sem conexão" em até 1 minuto.
- **RF-20.6** O BD pode desligar a sincronização numa máquina; a configuração local fica como estava.
  - Consequência testável: depois de desligar, alterações no Mac não chegam mais ao Windows, e nada é apagado no Windows.

**Critérios de aceite (alto nível)**
- Dados o Mac e o Windows conectados, quando o BD cria o modo "Relatório" no Windows, então o modo aparece no Mac em até 5 minutos, com o mesmo prompt.
- Dado um modo que usa o Groq, quando ele chega numa máquina sem a chave do Groq, então o modo aparece com o aviso "cadastre a chave do Groq nesta máquina".
- Dados os dados sincronizados, quando alguém inspeciona o que trafega e o que fica guardado fora da máquina, então não há chaves de API, áudios nem textos ditados.

**Notas / perguntas:** o meio da sincronização é decisão do Otávio, dentro das restrições da seção 10 (sob controle do BD, sem serviço pago de terceiros sem aprovação, dados protegidos). Se o BD recusar a premissa, F-20 sai da v1 e F-21 cobre UJ-3 manualmente.

### F-21 Exportar e importar a configuração

**Épico:** E-05 · **Prioridade:** Importante · **Realiza:** UJ-3 · **Origem:** ER-25
**Descrição:** o BD salva a configuração sincronizável num arquivo e a carrega em outra máquina ou depois de reinstalar. Serve de cópia de segurança e de alternativa à sincronização.

**Requisitos funcionais**
- **RF-21.1** O BD pode exportar a configuração sincronizável para um arquivo.
  - Consequência testável: o arquivo contém modos, prompts, vocabulário, substituições e palavras de preenchimento, e não contém chaves de API.
- **RF-21.2** O BD pode importar um arquivo de configuração; o app mostra o que será substituído e pede confirmação.
  - Consequência testável: depois de confirmar, a configuração da máquina fica igual à do arquivo; se a sincronização estiver ligada, a outra máquina também recebe.
- **RF-21.3** Um arquivo inválido ou de outra origem é recusado sem alterar nada.
  - Consequência testável: importar um arquivo de texto qualquer mostra "arquivo de configuração inválido", e a configuração continua a mesma.

**Critérios de aceite (alto nível)**
- Dada uma configuração exportada no Mac, quando o BD a importa num Windows novo, então os modos e o dicionário ficam iguais aos do Mac.
- Dado um arquivo corrompido, quando o BD importa, então nada muda e aparece o aviso.

**Fora do escopo desta funcionalidade:** importar a configuração do VoiceInk (o formato dele não foi observado; avaliar na v2 se o BD pedir).

### F-22 Transcrição local (Whisper)

**Épico:** E-06 · **Prioridade:** Importante · **Realiza:** UJ-5 · **Origem:** ER-08
**Descrição:** o BD baixa um modelo Whisper para a máquina e o escolhe como provedor de transcrição de um modo. A transcrição acontece sem internet e sem enviar o áudio para fora. Na v1 há um único modelo oferecido: o equivalente ao large-v3-turbo, que o BD já usa. [PREMISSA]

**Requisitos funcionais**
- **RF-22.1** O BD pode baixar, ver o tamanho e o progresso, e apagar o modelo local.
  - Consequência testável: durante o download, a tela mostra a porcentagem; depois de apagar, o espaço é liberado e os modos que usavam o modelo passam a mostrar "modelo não baixado".
- **RF-22.2** Com o modelo baixado, um modo com transcrição local funciona com a rede desligada.
  - Consequência testável: com a rede desligada, um ditado de 30 s num modo local é transcrito e colado.
- **RF-22.3** Se o modo usa transcrição local e o modelo não está baixado, o Voice Desk avisa antes de gravar.
  - Consequência testável: o gravador mostra "baixe o modelo local" com atalho para a tela de modelos, e nada é gravado.
- **RF-22.4** O download interrompido pode ser retomado ou reiniciado sem deixar arquivo corrompido em uso.
  - Consequência testável: desligar a rede no meio do download e religar permite concluir; um modelo incompleto nunca aparece como "pronto".

**Critérios de aceite (alto nível)**
- Dado o modelo baixado e o Mac sem internet, quando o BD dita num modo local, então o texto é colado dentro do tempo da RNF-01.
- Dado um modo local com refinamento pelo Groq e sem internet, quando o BD dita, então a transcrição local funciona e o refinamento falha como em RF-11.3 (cola o texto limpo e avisa).

**Fora do escopo desta funcionalidade:** outros modelos locais (Parakeet, Nemotron, SenseVoice, Paraformer, Apple Speech) e importação de modelos pelo usuário.

### F-23 Escolha do microfone

**Épico:** E-07 · **Prioridade:** Importante · **Realiza:** UJ-1, UJ-2 · **Origem:** ER-04
**Descrição:** o BD escolhe o microfone usado nos ditados. O Voice Desk lembra a escolha e lida com a troca de dispositivo (fone que conecta ou desconecta).

**Requisitos funcionais**
- **RF-23.1** O BD pode escolher o microfone numa lista com os dispositivos de entrada conectados, ou "padrão do sistema".
  - Consequência testável: com um headset escolhido, o histórico registra o ditado com áudio do headset.
- **RF-23.2** Se o microfone escolhido não estiver conectado, o ditado usa o padrão do sistema e o gravador avisa qual microfone está em uso.
  - Consequência testável: com o headset desconectado, o ditado acontece pelo microfone padrão, com o aviso.
- **RF-23.3** Quando o microfone escolhido volta a ser conectado, ele volta a ser usado no próximo ditado.
  - Consequência testável: reconectar o headset faz o ditado seguinte usar o headset.

**Critérios de aceite (alto nível)**
- Dado o headset escolhido e conectado, quando o BD dita, então o nível de áudio do gravador reage à voz no headset.
- Dado nenhum microfone disponível, quando o BD aciona o atalho, então o gravador avisa "nenhum microfone encontrado" e não grava.

**Notas / perguntas:** a escolha do microfone é por máquina e não sincroniza.

### F-24 Sons de início e fim

**Épico:** E-07 · **Prioridade:** Desejável · **Realiza:** UJ-2 · **Origem:** ER-05
**Descrição:** um som curto ao começar a gravar e outro ao terminar o processamento, para o BD ditar sem olhar para a tela.

**Requisitos funcionais**
- **RF-24.1** O BD pode ligar ou desligar os sons e escolher entre pelo menos 3 sons próprios do Voice Desk, com prévia.
  - Consequência testável: com os sons desligados, nenhum som toca durante o ditado.
- **RF-24.2** O som de início toca antes da captura de áudio começar e não entra na gravação.
  - Consequência testável: ao ouvir o áudio no histórico, o som de início não aparece.

**Critérios de aceite (alto nível)**
- Dados os sons ligados, quando o BD dita, então ouve um som ao iniciar e outro quando o texto é colado.
- Dados os sons desligados, quando o BD dita, então nenhum som toca.

**Fora do escopo desta funcionalidade:** sons personalizados enviados pelo usuário; sons do VoiceInk (não podem ser reaproveitados).

### F-25 Pausar a mídia durante o ditado

**Épico:** E-07 · **Prioridade:** Desejável · **Realiza:** UJ-1 · **Origem:** ER-06
**Descrição:** se havia música ou vídeo tocando, a mídia pausa ao iniciar o ditado e volta ao terminar a gravação.

**Requisitos funcionais**
- **RF-25.1** O BD pode ligar ou desligar a pausa de mídia; o padrão é ligado.
  - Consequência testável: com a opção desligada, a música continua tocando durante o ditado.
- **RF-25.2** A mídia só é retomada se estava tocando antes do ditado.
  - Consequência testável: com a música já pausada pelo BD, ela continua pausada depois do ditado.

**Critérios de aceite (alto nível)**
- Dado o Spotify tocando, quando o BD inicia o ditado, então a música pausa em até 500 ms e volta quando ele para de gravar.
- Dado um programa de mídia que não aceita o controle do sistema, quando o BD dita, então o ditado funciona normalmente e a mídia não é afetada.

**Notas / perguntas:** no VoiceInk isso usa recurso privado da Apple (relatório §8); a viabilidade em cada sistema é avaliada pelo Otávio. Se não for viável numa plataforma, vira "silenciar o áudio do sistema", com aprovação do BD.

### F-26 Descartar ditado sem fala

**Épico:** E-07 · **Prioridade:** Desejável · **Realiza:** UJ-2 · **Origem:** ER-07
**Descrição:** se a gravação não tem fala (atalho acionado sem querer, microfone mudo), o Voice Desk não envia o áudio ao provedor e não cola nada.

**Requisitos funcionais**
- **RF-26.1** O BD pode ligar ou desligar o descarte de ditado sem fala; o padrão é ligado.
  - Consequência testável: com a opção desligada, uma gravação silenciosa é enviada ao provedor como qualquer outra.
- **RF-26.2** Com a opção ligada, uma gravação sem fala detectada termina com o aviso "nenhuma fala detectada", sem envio ao provedor, sem colagem e sem item no histórico.
  - Consequência testável: 5 s de silêncio não geram chamada ao provedor nem item no histórico.

**Critérios de aceite (alto nível)**
- Dado o descarte ligado, quando o BD grava 5 s em silêncio, então aparece "nenhuma fala detectada" e nada é colado.
- Dado o descarte ligado, quando o BD fala baixo mas audível, então o ditado é processado normalmente.

## 6. Requisitos não funcionais

| ID | Categoria | Requisito (com limite concreto) |
|---|---|---|
| RNF-01 | Desempenho | Tempo entre parar a gravação e o texto colado, em ditados de até 60 s, medido no percentil 90 de 20 ditados: (a) transcrição na nuvem sem refinamento: até 3 s; (b) nuvem com refinamento na nuvem: até 6 s; (c) transcrição local sem refinamento: até 10 s no Mac do BD e até 20 s no Windows do BD. Medido com conexão de pelo menos 10 Mbps. [PREMISSA] |
| RNF-02 | Desempenho | O gravador aparece em até 300 ms depois do atalho, e a captura começa em até 300 ms: a primeira palavra dita logo após o gravador aparecer não se perde em 19 de 20 testes. |
| RNF-03 | Consumo | Em repouso (sem ditado e sem modelo local carregado), o app usa em média menos de 1% de CPU em 5 minutos e até 250 MB de memória. [PREMISSA] |
| RNF-04 | Compatibilidade | macOS 14 ou superior e Windows 10 (22H2) e 11, 64 bits. As mesmas funcionalidades da v1 existem nas duas plataformas; as diferenças permitidas são as teclas (Cmd ou Ctrl) e o local do ícone (barra de menus ou bandeja). [PREMISSA] |
| RNF-05 | Compatibilidade da colagem | A colagem (F-03) e a leitura da seleção (F-13) funcionam em 100% dos testes com estes programas: no Windows, Outlook, Teams, Word, Chrome, Edge, Slack e Bloco de Notas; no Mac, Mail, Slack, Chrome, Safari, Word, Notas e VS Code. [PREMISSA: lista a confirmar pelo BD] |
| RNF-06 | Segurança | Chaves de API só no cofre do sistema; nunca em arquivo comum, log, exportação, sincronização ou tela (só os 4 últimos caracteres). Comunicação com provedores de nuvem só por conexão criptografada. |
| RNF-07 | Privacidade (LGPD) | Áudio e texto dos ditados podem ter dados de clientes. Ficam só na máquina e só saem para o provedor de transcrição e de IA do modo. A sincronização leva só a configuração sincronizável, protegida no trânsito e no armazenamento, e acessível só ao BD. Sem telemetria nem envio de uso para terceiros. O BD apaga tudo quando quiser (RF-17.3), e a retenção apaga o resto (F-17). |
| RNF-08 | Confiabilidade | Nenhum ditado concluído se perde: se a transcrição falhar ou o app fechar no meio do processamento, o áudio fica no histórico como "falhou" ao reabrir, em 100% dos testes de queda forçada. |
| RNF-09 | Observabilidade | Log local de erros e eventos (início e fim do ditado, provedor, tempos, resultado da colagem), sem texto ditado, áudio nem chaves; guardado por 7 dias, com até 20 MB; o BD abre a pasta do log pelo menu de configurações. |
| RNF-10 | Idioma | Interface em português do Brasil. Ditado em português do Brasil e inglês no mínimo. |
| RNF-11 | Acessibilidade | Todas as ações das telas de configuração e do histórico podem ser feitas só com o teclado; contraste WCAG 2.1 AA nessas telas. |
| RNF-12 | Instalação | Um instalador por plataforma; a instalação leva até 2 minutos. No Windows, instala sem privilégio de administrador. [PREMISSA: depende de o Windows do BD ser pessoal ou corporativo] |
| RNF-13 | Propriedade intelectual | Nenhum código, texto de interface, prompt, ícone, som ou marca do VoiceInk é usado. Os textos, prompts e sons iniciais são criados pelo time. |

## 7. Escopo da v1

### 7.1 Dentro

As 26 funcionalidades da seção 4 (F-01 a F-26), nas duas plataformas, com os provedores:
- **Transcrição:** Groq (nuvem) e Whisper local (um modelo).
- **IA:** Groq (nuvem) e Ollama (local).

Isso cobre tudo o que o resumo aprovado lista como "copiar o comportamento (v1)" e "mudar". Mapa:

| Resumo (v1) | Onde está |
|---|---|
| Gravação: atalho, Esc, gravador, microfone, sons, pausar mídia (ER-01 a ER-07) | F-01, F-14, F-23, F-24, F-25, F-26 |
| Transcrição na nuvem, idioma e dica de vocabulário (ER-09, ER-11) | F-02, F-07 |
| Vocabulário, substituição, palavras de preenchimento, formatação (ER-14, ER-20, ER-21) | F-07, F-08, F-09 |
| Colar e enviar Enter (ER-13) | F-03 |
| Modos, refinamento, contexto, troca por programa (ER-15 a ER-18, sem OCR) | F-10 a F-15 |
| Histórico, áudio, busca, retenção, CSV (ER-23 a ER-25) | F-16 a F-19 |
| Cofre de chaves, iniciar com o sistema, primeiro uso (ER-27 a ER-29) | F-04, F-05, F-06 |
| Mac e Windows com a mesma experiência | RNF-04, RNF-05 |
| Configuração sincronizada | F-20, F-21 |
| Transcrição local na v1 | F-22 |
| Conjunto enxuto de provedores | seção 7.1 |
| Sem licença, teste ou ativação | seção 7.2 |

### 7.2 Fora da v1 (com motivo)

| Item | Motivo | Quando |
|---|---|---|
| Contexto por OCR da tela e texto da janela atual (parte do ER-16) | Recursos de captura de tela muito diferentes entre Mac e Windows; risco de privacidade | v2 |
| Troca de modo pelo site aberto no navegador (parte do ER-18) | Depende de integração por navegador e por sistema | v2 |
| Transcrição em tempo real, com texto ao vivo (ER-10) | Não é essencial ao fluxo central | v2 |
| Transcrição de arquivo de áudio (ER-12) | Não é essencial ao fluxo central | v2 |
| Painel de estatísticas (ER-26) | O histórico (F-16) já guarda os tempos que as métricas da seção 9 usam | v2 |
| Outros provedores de transcrição e de IA (OpenAI, Deepgram, Anthropic, Gemini, OpenRouter, LM Studio etc.) | Reduzir a configuração inicial; entram como funcionalidades novas, a pedido do BD | v2, sob demanda |
| Outros modelos locais e importação de modelos (parte do ER-08) | Um modelo cobre o uso sem internet | v2, sob demanda |
| Modo "segurar para falar" | Não foi observado no relatório; sem pedido do BD | Avaliar |
| Sincronizar histórico e áudio entre máquinas | Privacidade e volume; a premissa é que só a configuração viaja | Depende da pergunta 2 |
| Atualização automática do app (parte do ER-31) | Uso por uma pessoa; a nova versão é instalada manualmente [PREMISSA] | v2 |
| Assistente/chat, "perguntar" e voz sintetizada (ER-19, ER-32) | Não fazem parte do ditado; TTS nem está confirmado no VoiceInk | Não previsto |
| Gravador no entalhe ("notch"), ações do app Atalhos da Apple, comando local ("Local CLI"), modelo de refino embutido (ER-03 parcial, ER-15 parcial, ER-27 parcial) | Exclusivos do Mac ou de nicho | Não previsto |
| Licença, teste grátis, afiliados, avisos remotos, convite para estrela no GitHub (ER-30, ER-31) | Uso próprio | Não previsto |
| Importar a configuração do VoiceInk | Formato não observado | Avaliar se o BD pedir |

## 8. Não objetivos

- Não é um produto comercial nem multiusuário: sem contas, planos, pagamentos ou administração.
- Não é um clone do VoiceInk: não busca paridade de funcionalidades, só o fluxo que o BD usa no dia a dia.
- Não guarda nem processa os ditados em servidor próprio: áudio e texto ficam na máquina ou vão direto ao provedor escolhido pelo BD.
- Não é assistente de voz nem controla o computador por comandos de voz.
- Não tem versão para celular nem para navegador.

## 9. Métricas de sucesso

Todas medidas na própria máquina, pelo histórico e pelo log local (sem telemetria, RNF-07).

**Primárias:**
- **M-1 Substituição do VoiceInk** — dias seguidos em que o BD dita só pelo Voice Desk nas duas máquinas, sem abrir o VoiceInk. Medida: itens no histórico de cada máquina por dia, mais a confirmação do BD ao fim do período. Meta: 30 dias seguidos. [PREMISSA] Valida a tese inteira (F-01 a F-22).
- **M-2 Tempo até colar** — percentil 90 do tempo entre parar a gravação e o texto colado, por tipo de modo (RNF-01). Fonte: tempos registrados no histórico (RF-16.3). Meta: dentro dos limites da RNF-01 nas duas plataformas. Valida F-02, F-03, F-11, F-22.

**Secundárias:**
- **M-3 Taxa de colagem** — ditados com estado "colado" divididos pelos ditados concluídos, por máquina. Meta: pelo menos 98% em cada máquina. Valida F-03 e o maior risco do produto.
- **M-4 Paridade de configuração** — alterações na configuração sincronizável que aparecem na outra máquina em até 5 minutos. Meta: 100% em 20 alterações de teste e nenhuma divergência relatada pelo BD em 30 dias. Valida F-20.

**Contra-métricas (não otimizar):**
- **M-C1 Ditados refeitos** — ditados seguidos de outro no mesmo programa em foco em até 30 s, divididos pelo total. Meta: até 10%. Um texto ruim faz o BD ditar de novo; se essa taxa subir, é sinal de que se ganhou velocidade (M-2) à custa da qualidade do texto, por exemplo cortando refinamento ou vocabulário. Equilibra M-2.
- **M-C2 Ditados colados sem refinamento por falha** — refinamentos com estado "falhou", divididos pelos ditados em modos com refinamento. Meta: até 2%. O fallback (RF-11.3) protege M-3, mas não pode esconder falhas frequentes de IA. Equilibra M-3.

## 10. Restrições e integrações

- **Licença e marca:** o VoiceInk é GPL-3.0. O Voice Desk não reaproveita código, textos, prompts, ícones, sons nem marca dele (RNF-13). Qualquer reuso de código depende de parecer do Otávio e de aprovação do BD antes.
- **Plataformas:** Mac e Windows desde a v1, com a mesma experiência (RNF-04). O maior risco técnico é colar e ler a seleção em cada sistema: deve ser validado no início do E-01, nas máquinas reais do BD.
- **Provedores externos:** Groq (transcrição e IA, chave do BD, custo dele), Ollama (IA local, instalado pelo BD) e Whisper local (modelo baixado pelo app). O custo de uso dos provedores é do BD.
- **Sincronização:** o meio é decisão do Otávio, com estas restrições de produto: (a) os dados ficam sob controle do BD; (b) nenhum serviço pago novo sem aprovação do BD; (c) só a configuração sincronizável trafega, protegida (RNF-07); (d) funciona com o Mac e o Windows em redes diferentes.
- **Permissões do sistema:** microfone e acessibilidade nas duas plataformas, mais as equivalentes no Mac. O primeiro uso (F-06) guia a concessão.
- **Distribuição:** só para as máquinas do BD; sem loja de apps na v1. [PREMISSA]

## 11. Premissas

- [PREMISSA] O Voice Desk é para uso próprio do BD, sem venda, licença, teste grátis ou ativação (seções 2.2, 7.2, 8).
- [PREMISSA] A sincronização da configuração entre Mac e Windows entra na v1 como Essencial; se o BD recusar, F-20 sai e F-21 cobre a necessidade manualmente (F-20).
- [PREMISSA] Na sincronização só viaja a configuração sincronizável: nada de chaves, histórico ou áudio (glossário, F-20, RNF-07).
- [PREMISSA] Em conflito de sincronização, vale a alteração mais recente do mesmo item (RF-20.4).
- [PREMISSA] Provedores da v1: transcrição pelo Groq e pelo Whisper local; IA pelo Groq e pelo Ollama, inferidos da instalação do BD (seção 7.1).
- [PREMISSA] A transcrição local entra na v1 com um modelo, o equivalente ao Whisper large-v3-turbo (F-22).
- [PREMISSA] Duração máxima do ditado: 10 minutos (RF-01.5).
- [PREMISSA] Vocabulário com até 500 palavras (RF-07.4).
- [PREMISSA] Contexto limitado a 8.000 caracteres por fonte (RF-13.2).
- [PREMISSA] Retenção padrão: áudio 7 dias e histórico 90 dias (RF-17.1).
- [PREMISSA] Metas de tempo até colar: 3 s, 6 s, 10 s no Mac e 20 s no Windows, no percentil 90 (RNF-01).
- [PREMISSA] Consumo em repouso: menos de 1% de CPU e até 250 MB de memória (RNF-03).
- [PREMISSA] Plataformas mínimas: macOS 14 e Windows 10 22H2 e 11, 64 bits (RNF-04).
- [PREMISSA] Lista de programas de referência para a colagem (RNF-05).
- [PREMISSA] Instalação no Windows sem privilégio de administrador (RNF-12).
- [PREMISSA] Sem atualização automática na v1 e sem loja de apps (seções 7.2 e 10).
- [PREMISSA] Sucesso = 30 dias seguidos usando só o Voice Desk nas duas máquinas (M-1).

## 12. Perguntas em aberto

1. Quais provedores de transcrição e de IA o BD usa de fato no dia a dia? Se não forem o Groq, o Whisper local e o Ollama, a seção 7.1 muda. — BD — não bloqueia (o PRD assume os da instalação dele).
2. O histórico (textos e áudios) deve aparecer na outra máquina? — BD — não bloqueia (o PRD assume que não).
3. O Windows do BD é pessoal ou corporativo, com restrição de instalação ou antivírus? — BD — não bloqueia o design, mas afeta a RNF-12 e o risco da colagem; precisa de resposta antes do E-01 ir para teste.
4. Quais programas o BD mais usa para ditar em cada máquina? — BD — não bloqueia (a RNF-05 traz uma lista inicial).

## 13. Histórico e reconciliação

| Versão | Data | Mudança | Origem |
|---|---|---|---|
| 1 | 2026-10-23 | Versão inicial | Resumo aprovado (ENG-51) + relatório de engenharia reversa do VoiceInk 2.13 |

Decisões tomadas neste PRD sobre o que o resumo deixou aberto ou o que não foi observado no VoiceInk:
- **Contexto:** o resumo fala em "seleção e área de transferência". O PRD exclui também o texto da janela atual, que o relatório lista junto do OCR, pelo mesmo motivo do OCR.
- **Atalho por modo e palavra-gatilho** (ER-01, ER-18) entraram como Desejáveis (F-14, F-15): estão dentro das faixas que o resumo manda copiar, mas não são citados nas jornadas.
- **Transcrever de novo** (F-18): vem do fluxo FL-05 do relatório e entrou como Desejável, por ser a forma de recuperar ditados que falharam (UJ-4).
- **Comportamentos não observados no VoiceInk** e definidos aqui: devolver o conteúdo da área de transferência depois de colar (RF-03.2); Enter por modo, não global (RF-03.3); ignorar o atalho durante o processamento (RF-01.6); colar o texto limpo quando o refinamento falha (RF-11.3); ordem de prioridade para escolher o modo (RF-12.3).
- **Sincronização:** substitui a sincronização do dicionário pelo iCloud (ER-22), que só valia entre Macs, e passa a cobrir toda a configuração sincronizável.
