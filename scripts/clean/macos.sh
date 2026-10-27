#!/bin/bash
set -u

DRY=0
[ "${1:-}" = "--dry-run" ] && DRY=1

ID="dev.rgdevment.tisty"
LIB="$HOME/Library"
DATA="$LIB/Application Support/tisty"

PROGRAM_FILES=(
  "/Applications/Tisty.app"
  "$HOME/Applications/Tisty.app"
  "$LIB/Application Support/$ID"
  "$LIB/Caches/tisty"
  "$LIB/Caches/$ID"
  "$LIB/WebKit/$ID"
  "$LIB/HTTPStorages/$ID"
  "$LIB/Saved Application State/$ID.savedState"
  "$LIB/Preferences/$ID.plist"
  "$LIB/LaunchAgents/$ID.plist"
)

LINKS=(
  "/opt/homebrew/bin/tisty"
  "/usr/local/bin/tisty"
  "$HOME/.local/bin/tisty"
)

first=$(defaults read -g AppleLanguages 2>/dev/null | tr -d ' "(),' | grep . | head -1)
case "$first" in es*) ES=1 ;; *) ES=0 ;; esac

say() { if [ "$ES" = 1 ]; then printf '%s\n' "$2"; else printf '%s\n' "$1"; fi; }

ask() {
  local answer
  printf '%s ' "$(say "$1" "$2")" >&2
  read -r answer || answer=""
  printf '%s' "$answer"
}

yes_to() {
  case "$1" in [yYsS]*) return 0 ;; *) return 1 ;; esac
}

ours() {
  [ -L "$1" ] && case "$(readlink "$1")" in *Tisty*.app/*) return 0 ;; esac
  return 1
}

take() {
  [ -e "$1" ] || [ -L "$1" ] || return 0
  if [ "$DRY" = 1 ]; then
    say "  would remove: $1" "  se borraría: $1"
  elif rm -rf "$1"; then
    say "  removed: $1" "  borrado: $1"
  else
    say "  could not remove: $1" "  no se pudo borrar: $1"
  fi
}

close_tisty() {
  if pgrep -x tisty-gui >/dev/null; then
    osascript -e "tell application id \"$ID\" to quit" >/dev/null 2>&1
    for _ in 1 2 3 4 5 6 7 8 9 10; do
      pgrep -x tisty-gui >/dev/null || break
      sleep 1
    done
  fi
  pkill -x tisty-gui 2>/dev/null
  pkill -x tisty 2>/dev/null
}

back_up() {
  [ -d "$DATA" ] || return 0
  local to
  to="$HOME/Desktop/tisty-backup-$(date +%Y%m%d-%H%M%S).tar.gz"
  if [ "$DRY" = 1 ]; then
    say "  would back up $DATA to $to" "  se respaldaría $DATA en $to"
    return 0
  fi
  if tar -czf "$to" -C "$LIB/Application Support" tisty; then
    say "Your tasks, documents and settings are kept in $to" "Tus tareas, documentos y ajustes quedan en $to"
    return 0
  fi
  rm -f "$to"
  say "The backup could not be made, so nothing was removed." "No se pudo hacer el respaldo, así que no se borró nada."
  return 1
}

leave_homebrew() {
  command -v brew >/dev/null || return 0
  local cask
  for cask in tisty tisty-beta; do
    brew list --cask "$cask" >/dev/null 2>&1 || continue
    if [ "$DRY" = 1 ]; then
      say "  would run: brew uninstall --cask $cask" "  se ejecutaría: brew uninstall --cask $cask"
    else
      brew uninstall --cask --force "$cask"
    fi
  done
  local download
  for download in "$(brew --cache)"/downloads/*tisty-installer* "$(brew --cache)"/Cask/tisty*; do
    take "$download"
  done
}

forget_keys() {
  if [ "$DRY" = 1 ]; then
    say "  would remove Tisty's entries from the Keychain" "  se borrarían las entradas de Tisty del Llavero"
    return 0
  fi
  while security delete-generic-password -s tisty >/dev/null 2>&1; do
    say "  removed a Keychain entry" "  borrada una entrada del Llavero"
  done
}

say "Tisty: remove every trace from this Mac" "Tisty: borrar todo rastro de este Mac"
[ "$DRY" = 1 ] && say "(dry run: nothing is touched)" "(ensayo: no se toca nada)"
echo

keys=0
if [ "$DRY" = 1 ]; then
  keys=1
else
  say "The Keychain holds your cloud accesses and this Mac's copy of your recovery phrase. Save the phrase first in Settings › Security if you might need it." \
      "El Llavero guarda tus accesos a la nube y la copia de tu frase de recuperación en este Mac. Guarda la frase primero en Configuración › Seguridad si podrías necesitarla."
  yes_to "$(ask "Remove them too? (y/N)" "¿Borrarlos también? (s/N)")" && keys=1
  echo
  sure=$(ask "Type DELETE to remove Tisty and everything it left:" "Escribe BORRAR para quitar Tisty y todo lo que dejó:")
  case "$sure" in DELETE|delete|BORRAR|borrar) ;; *) say "Cancelled. Nothing was removed." "Cancelado. No se borró nada."; exit 1 ;; esac
fi

[ "$DRY" = 1 ] || close_tisty
back_up || exit 1
leave_homebrew
[ "$DRY" = 1 ] || launchctl bootout "gui/$(id -u)/$ID" 2>/dev/null
[ "$DRY" = 1 ] || defaults delete "$ID" 2>/dev/null
for one in "${PROGRAM_FILES[@]}"; do take "$one"; done
for one in "${LINKS[@]}"; do ours "$one" && take "$one"; done
take "$DATA"
[ "$keys" = 1 ] && forget_keys

echo
if [ "$DRY" = 1 ]; then
  say "Dry run over. Run it again without --dry-run to remove." "Fin del ensayo. Ejecútalo otra vez sin --dry-run para borrar."
else
  say "Done. Tisty is gone from this Mac." "Listo. Tisty ya no está en este Mac."
fi
