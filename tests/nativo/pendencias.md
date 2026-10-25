# Pendências nativas

Itens que só se conferem em máquina real; o BD roda um roteiro por épico.

| Card | Épico | Item a conferir | Passos | Esperado | SO |
|---|---|---|---|---|---|
| ENG-66 | E1 | Gravador aparece sem tirar o foco | Abrir o Bloco de Notas/TextEdit, acionar o atalho global | Gravador em até 300 ms; o cursor continua no campo | macOS, Windows |
| ENG-66 | E1 | Parar pelo mesmo atalho | Falar e acionar o atalho de novo | Gravador passa a "processando" e some sozinho | macOS, Windows |
| ENG-66 | E1 | Esc duplo cancela, Esc simples passa | Gravando, apertar Esc duas vezes em 1 s; depois uma vez só | Duplo: gravador some, nada colado; simples: Esc chega ao programa em foco | macOS (exige Acessibilidade), Windows |
| ENG-66 | E1 | Atalho durante o processamento | Acionar o atalho enquanto processa | Não abre nova gravação; gravador indica "aguarde" | macOS, Windows |
| ENG-66 | E1 | Trocar o atalho sem reiniciar | Configurações › Atalhos: salvar outro; usar em outro programa | Novo atalho funciona de imediato; só letra ou atalho do sistema é recusado | macOS, Windows |
| ENG-66 | E1 | Microfone sem permissão | Negar o microfone e acionar o atalho | Gravador mostra "sem acesso ao microfone" com o caminho; nada gravado | macOS, Windows |
| ENG-66 | E1 | Limite de 10 min | Gravar sem parar | Aviso aos 9:30; aos 10:00 segue para o processamento | macOS, Windows |
| ENG-69 | E1 | Chave no cofre do sistema | Salvar uma chave do Groq real em Configurações › Provedores e chaves; abrir Keychain / Gerenciador de Credenciais | Entrada do Voice Desk com a chave; nada em arquivo comum; "válida" em até 10 s | macOS, Windows |
| ENG-69 | E1 | Ditado sem chave | Apagar a chave e ditar num modo com Groq | Aviso "Cadastre a chave do Groq" | macOS, Windows |
| ENG-70 | E1 | Ícone na barra de menus/bandeja sem janela | Abrir o app instalado | Ícone do Voice Desk aparece; nenhuma janela principal abre | macOS, Windows |
| ENG-70 | E1 | Menu do ícone | Clicar no ícone e usar cada item | Ditado, modo padrão (marca o modo), histórico e configurações executam a ação; fechar a janela não encerra o app | macOS, Windows |
| ENG-70 | E1 | Sair com confirmação | Menu > Sair; cancelar; repetir e confirmar | Cancelar mantém o ícone; confirmar remove o ícone e o atalho global deixa de funcionar | macOS, Windows |
| ENG-70 | E1 | Iniciar com o sistema | Com a opção ligada, reiniciar a máquina e entrar; depois desligar em Geral e repetir | Ligada: ícone em até 5 s, sem janela, atalho global funciona; desligada: não abre sozinho | macOS, Windows |
| ENG-70 | E1 | Estados do ícone | Ditar com o F-01 integrado | Ícone muda ao gravar/processar/erro e volta ao normal (depende de `bandeja::definir_estado` no F-01) | macOS, Windows |
| ENG-67 | E1 | Transcrição real no Groq | Com chave real do Groq no modo ativo, ditar ~20 s em português com um editor em foco | O texto transcrito é colado no editor; o item no histórico mostra provedor e modelo do modo | macOS, Windows |
| ENG-67 | E1 | Idioma inglês sem tradução | No modo com idioma "inglês", ditar uma frase em inglês | Texto em inglês, sem tradução | macOS, Windows |
| ENG-67 | E1 | Falha sem rede | Desligar a rede e ditar | Aviso "sem conexão com o provedor"; nada colado; item "falhou" no histórico com áudio | macOS, Windows |
| ENG-67 | E1 | Sem chave | Apagar a chave do Groq e iniciar o ditado | Gravador avisa "Cadastre a chave do Groq" com botão "Abrir chaves" que abre a tela de chaves | macOS, Windows |
| ENG-68 | E1 | Colar no Outlook (Windows) | Cursor no corpo de um e-mail novo; ditar uma frase com acentos, emoji e quebra de linha | Texto aparece na posição do cursor | Windows |
| ENG-68 | E1 | Colar no Mail e no Slack (Mac) | Mesmo cenário no corpo de um e-mail e numa conversa | Texto aparece na posição do cursor, com acentos, emoji e quebra de linha | macOS |
| ENG-68 | E1 | Área de transferência volta ao anterior | Copiar "abc"; ditar e esperar a colagem; colar com Cmd/Ctrl+V | Cola "abc" (e, com imagem copiada antes, a imagem) | macOS, Windows |
| ENG-68 | E1 | Sem campo de texto em foco | Foco na área de trabalho/janela sem campo; ditar | Texto fica na área de transferência e aparece "Texto copiado; cole com Cmd+V/Ctrl+V" (a detecção de "sem campo" não existe: pode colar em nada) | macOS, Windows |
| ENG-68 | E1 | Acessibilidade negada (Mac) | Remover a permissão de Acessibilidade; ditar | Aviso "Texto copiado; cole com Cmd+V"; colar manualmente funciona | macOS |
| ENG-68 | E1 | Enter depois de colar | Num modo com Enter ligado, ditar no chat; depois com Enter desligado (depende da área de modos) | Ligado: mensagem enviada sozinha; desligado: fica na caixa | macOS, Windows |
| ENG-71 | E1 | Detecção real de permissões no primeiro uso | Abrir o app pela 1ª vez sem conceder nada; ver o passo Permissões; conceder microfone (e Acessibilidade no Mac) | Estado muda de Pendente para Concedida; negada mostra o que deixa de funcionar e como conceder; "Abrir configurações do sistema" abre a tela certa | macOS, Windows |
| ENG-71 | E1 | Ditado de teste no primeiro uso | No último passo, clicar na caixa de teste, acionar o atalho e falar | O texto ditado aparece na caixa; com falha, aparece o motivo | macOS, Windows |
| ENG-71 | E1 | Primeiro uso não reaparece | Concluir o primeiro uso, fechar e reabrir o app | O primeiro uso não abre de novo; continua em Configurações › Primeiro uso | macOS, Windows |
| ENG-98 | E1 | Diálogo de microfone no Mac | Instalar o `.dmg` da pré-release, abrir o app (contornar Gatekeeper, ENG-97), Primeiro uso › Permissões › Conceder microfone | Diálogo do macOS pede o microfone com o texto em português; o app passa a constar em Privacidade e Segurança › Microfone. Sem assinatura o entitlement `audio-input` ainda não vale | macOS |
| ENG-95 | E1 | Ditado cola o texto no campo em foco (Windows) | Instalar o `.exe` do `dev-5a965d9`, cadastrar a chave Groq, abrir o Bloco de Notas, acionar o atalho, falar uma frase e acionar de novo | Texto transcrito colado no campo; log com `transcricao_concluida` e `colagem_resultado` | Windows, macOS |
| ENG-95 | E1 | Ditado em silêncio | Acionar o atalho e parar sem falar | Gravador mostra "Não ouvi fala" (não "Pronto"); nada colado | Windows, macOS |
| ENG-95 | E1 | Captura sem "buffer underrun" (Windows) | Ditar 3 vezes seguidas e ver o log | Sem `erro_no_fluxo_de_audio` repetido; se aparecer, abrir bug da captura | Windows |
| ENG-96 | E1 | Início mostra o atalho atual | Abrir o app instalado; ler "Como ditar"; trocar o atalho em Atalhos e voltar | Início exibe o atalho vigente (kbd), e Testar ditado inicia a gravação | macOS, Windows |
