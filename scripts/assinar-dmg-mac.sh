#!/usr/bin/env bash
# Reassina o Voice Desk.app dentro do .dmg com um requisito designado estável (só o identificador
# do bundle). Sem isso a assinatura ad-hoc do build vale o hash do binário, que muda a cada build;
# o macOS (TCC) amarra Microfone e Acessibilidade a essa identidade e o Ajustes passa a mostrar
# "ativo" para um binário que não é o que está rodando. Uso: assinar-dmg-mac.sh <arquivo.dmg>
# A assinatura é feita numa cópia fora do volume (assinar dentro do volume montado falha no CI).
set -euo pipefail

dmg="$1"
id_bundle="br.com.going2.voicedesk"
requisito="designated => identifier \"$id_bundle\""
trabalho=$(mktemp -d)
rw="$trabalho/rw.dmg"
ponto="$trabalho/vol"
mkdir -p "$ponto" "$trabalho/copia"
montado=0
limpar() {
  [ "$montado" = 1 ] && hdiutil detach "$ponto" -quiet -force 2>/dev/null || true
  rm -rf "$trabalho"
}
trap limpar EXIT

hdiutil convert "$dmg" -format UDRW -o "$rw" -quiet
hdiutil attach "$rw" -mountpoint "$ponto" -nobrowse -quiet
montado=1

ditto "$ponto/Voice Desk.app" "$trabalho/copia/Voice Desk.app"
app="$trabalho/copia/Voice Desk.app"
xattr -cr "$app"
codesign --remove-signature "$app/Contents/MacOS/voice-desk" 2>/dev/null || true
codesign --force -s - -i "$id_bundle" -r="$requisito" "$app/Contents/MacOS/voice-desk"
codesign --force -s - -i "$id_bundle" -r="$requisito" "$app"
codesign --verify --strict "$app"
obtido=$(codesign -d -r- "$app" 2>&1 | grep '^designated' || true)
echo "$obtido"
[ "$obtido" = "$requisito" ] || { echo "requisito designado inesperado"; exit 1; }

rm -rf "$ponto/Voice Desk.app"
ditto "$app" "$ponto/Voice Desk.app"
hdiutil detach "$ponto" -quiet
montado=0
rm -f "$dmg"
hdiutil convert "$rw" -format UDZO -o "$dmg" -quiet
