#!/usr/bin/env bash
# Reassina o Voice Desk.app dentro do .dmg com um requisito designado estável (só o identificador
# do bundle). Sem isso a assinatura ad-hoc do build vale o hash do binário, que muda a cada build;
# o macOS (TCC) amarra Microfone e Acessibilidade a essa identidade e o Ajustes passa a mostrar
# "ativo" para um binário que não é o que está rodando. Uso: assinar-dmg-mac.sh <arquivo.dmg>
set -euo pipefail

dmg="$1"
id_bundle="br.com.going2.voicedesk"
trabalho=$(mktemp -d)
rw="$trabalho/rw.dmg"
ponto="$trabalho/vol"
mkdir -p "$ponto"

hdiutil convert "$dmg" -format UDRW -o "$rw" -quiet
hdiutil attach "$rw" -mountpoint "$ponto" -nobrowse -quiet
trap 'hdiutil detach "$ponto" -quiet -force 2>/dev/null || true; rm -rf "$trabalho"' EXIT

app="$ponto/Voice Desk.app"
codesign --force --deep \
  -s - -i "$id_bundle" -r="designated => identifier \"$id_bundle\"" "$app"
codesign --verify --deep --strict "$app"
requisito=$(codesign -d -r- "$app" 2>&1 | grep '^designated' || true)
echo "$requisito"
[ "$requisito" = "designated => identifier \"$id_bundle\"" ] || { echo "requisito designado inesperado"; exit 1; }

hdiutil detach "$ponto" -quiet
rm -f "$dmg"
hdiutil convert "$rw" -format UDZO -o "$dmg" -quiet
