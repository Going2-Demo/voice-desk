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
