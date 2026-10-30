#!/bin/zsh
# Le shell interactif peut transmettre le mode trace (xtrace) au script.
# Cela affiche les affectations locales sec=... et pct=... dans la progression.
unsetopt xtrace verbose
# Outil de renommage et conversion vidéo (macOS / zsh)
# Déposez les fichiers LUT .cube aux chemins configurés ci-dessous.

# --- CONFIGURATION -----------------------------------------------------------
SOURCE_DIR="" # Vide = demander le dossier source au lancement
EXPORT_DIR="" # Vide = proposer un dossier export (Entrée = source/export)

# LUT .cube, une par profil. Modifiez ces chemins pour correspondre à vos fichiers.
LUT_SLOG2="/path/to/your/S-Log2.cube"
LUT_SLOG3="/path/to/your/S-Log3.cube"
LUT_DLOGM="/path/to/your/D-Log-M.cube"

# --- Dépendances -------------------------------------------------------------
if [[ -x /opt/homebrew/bin/ffmpeg ]]; then
  FFMPEG_CMD="/opt/homebrew/bin/ffmpeg"
elif (( $+commands[ffmpeg] )); then
  FFMPEG_CMD="$(command -v ffmpeg)"
else
  print "❌ ffmpeg n'est pas installé."
  exit 1
fi
if (( ! $+commands[ffprobe] )); then
  print "⚠️ ffprobe est introuvable : durée et progression estimées indisponibles."
fi

clean_path() {
  local p="$1"
  p="${p#\"}"; p="${p%\"}"; p="${p#\'}"; p="${p%\'}"
  # Le Terminal peut coller les espaces échappés (\ ) et un espace insécable final.
  p="$(print -rn -- "$p" | perl -CSDA -pe 's/\\([[:space:]()])/\1/g; s/^[\s\x{a0}]+|[\s\x{a0}]+$//g')"
  print -r -- "$p"
}

get_source_folder() {
  local folder
  if [[ -n "$SOURCE_DIR" ]]; then
    folder="$SOURCE_DIR"
  else
    print "📂 Glissez-déposez le dossier source ici :" >&2
    read -r folder
    folder="$(clean_path "$folder")"
  fi
  if [[ ! -d "$folder" ]]; then
    print "❌ Dossier source invalide : $folder"
    return 1
  fi
  print -r -- "$folder"
}

setup_export_folder() {
  local source="$1" folder
  if [[ -n "$EXPORT_DIR" ]]; then
    folder="$EXPORT_DIR"
  else
    print "📂 Glissez-déposez le dossier export (ou Entrée pour créer source/export) :" >&2
    read -r folder
    folder="$(clean_path "$folder")"
    [[ -z "$folder" ]] && folder="$source/export"
  fi
  mkdir -p "$folder" || return 1
  print -r -- "$folder"
}

get_formatted_date() { stat -f "%Sm" -t "%Y-%m-%d_%H%M" "$1"; }
apply_original_dates() {
  local src="$1" dest="$2" ctime mtime
  [[ -e "$dest" ]] || return 0
  ctime=$(stat -f "%SB" -t "%Y%m%d%H%M.%S" "$src") || return
  mtime=$(stat -f "%Sm" -t "%Y%m%d%H%M.%S" "$src") || return
  touch -t "$ctime" "$dest"
  touch -mt "$mtime" "$dest"
}

progress_bar() {
  local percent="$1" width="$2" bar_index filled=$(( $1 * $2 / 100 ))
  printf '[\033[32m'
  for ((bar_index=0; bar_index<filled; bar_index++)); do printf '\u2588'; done
  printf '\033[0m'
  for ((bar_index=filled; bar_index<width; bar_index++)); do printf '\u2591'; done
  printf ']'
}

# Exécute ffmpeg avec le bloc de progression à dix lignes.
run_ffmpeg_progress() {
  setopt local_options local_traps
  local input="$1" idx="$2" total="$3"
  shift 3
  local filename duration logfile ffstatus
  local line key value stats percent=0 eta="--" global_percent=0
  local time_us=0 speed=0 fps=0 bitrate="0" first_run=1 terminal_mode=0
  local terminal_width=${COLUMNS:-80} bar_width=25 separator_line separator_index
  local blue=$'\033[34m' cyan=$'\033[36m' yellow=$'\033[33m' reset=$'\033[0m'
  filename="$(basename "$input")"
  duration=0
  if (( $+commands[ffprobe] )); then
    duration=$("$commands[ffprobe]" -v error -show_entries format=duration \
      -of default=noprint_wrappers=1:nokey=1 "$input" 2>/dev/null)
  fi
  [[ "$duration" == <->(|.<->) ]] || duration=0
  if [[ -t 1 ]]; then
    terminal_mode=1
    (( bar_width > terminal_width - 30 )) && bar_width=$(( terminal_width - 30 ))
    (( bar_width < 1 )) && bar_width=1
    for ((separator_index=0; separator_index<58 && separator_index<terminal_width-1; separator_index++)); do
      separator_line+="$(printf '\u2500')"
    done
  fi
  logfile="$(mktemp "${TMPDIR:-/tmp}/video-ffmpeg.XXXXXX")" || return 1
  trap '(( terminal_mode )) && tput cnorm 2>/dev/null; rm -f "$logfile"; return 130' INT TERM
  (( terminal_mode )) && tput civis 2>/dev/null
  "$FFMPEG_CMD" "$@" -progress pipe:1 -nostats 2>"$logfile" | while IFS= read -r line; do
    key=${line%%=*}
    value=${line#*=}
    case "$key" in
      out_time_us) time_us="$value" ;;
      speed) speed=${value%x} ;;
      fps) fps="$value" ;;
      bitrate) bitrate="$value" ;;
      progress)
        stats=$(awk -v duration="$duration" -v current="$time_us" -v speed="$speed" '
          BEGIN {
            duration *= 1000000; current += 0; speed += 0;
            percent = duration > 0 ? int(current * 100 / duration) : 0;
            if (percent > 100) percent = 100;
            if (percent < 0) percent = 0;
            eta = "--";
            if (duration > 0 && speed > 0.0001) {
              remaining = (duration - current) / 1000000 / speed;
              if (remaining < 0) remaining = 0;
              eta = sprintf("%02d:%02d:%02d", int(remaining / 3600), int(remaining / 60) % 60, int(remaining) % 60);
            }
            printf "%d|%s", percent, eta;
          }')
        IFS='|' read -r percent eta <<< "$stats"
        global_percent=$(( ((idx - 1) * 100 + percent) / total ))
        if (( terminal_mode )); then
          (( first_run == 0 )) && printf '\033[10A\r'
          first_run=0
          printf '%s%s%s\033[K\n' "$blue" "$separator_line" "$reset"
          printf '%s%.*s%s\033[K\n' "$yellow" "$((terminal_width - 1))" " Processing $idx/$total : $filename" "$reset"
          printf '%s%s%s\033[K\n' "$blue" "$separator_line" "$reset"
          printf ' Progress : '
          progress_bar "$percent" "$bar_width"
          printf ' %s%d%%%s\033[K\n' "$yellow" "$percent" "$reset"
          printf ' ETA      : %s%s%s\033[K\n' "$cyan" "$eta" "$reset"
          printf ' FPS      : %s%s%s\033[K\n' "$cyan" "$fps" "$reset"
          printf ' Bitrate  : %s%s%s\033[K\n' "$cyan" "$bitrate" "$reset"
          printf '%s%s%s\033[K\n' "$blue" "$separator_line" "$reset"
          printf '\033[K\n'
          printf ' TOTAL : '
          progress_bar "$global_percent" "$bar_width"
          printf ' %s%d%%%s (%d/%d)\033[K\n' "$yellow" "$global_percent" "$reset" "$idx" "$total"
        fi
        ;;
    esac
  done
  ffstatus=${pipestatus[1]}
  if (( terminal_mode )); then
    (( first_run == 0 )) && printf '\033[10A\r\033[J'
    tput cnorm 2>/dev/null
  fi
  if (( ffstatus != 0 )); then
    cat "$logfile" >&2
  else
    printf '\u2705 %s (Terminé)\n' "$filename"
  fi
  rm -f "$logfile"
  return $ffstatus
}

list_videos() {
  local dir="$1"
  setopt local_options null_glob
  REPLY=( "$dir"/*.(mp4|MP4|mov|MOV) )
}

rename_files() {
  local source="$1" file dirname filename prefix
  list_videos "$source"
  for file in "${REPLY[@]}"; do
    dirname="$(dirname "$file")"; filename="$(basename "$file")"
    prefix="$(get_formatted_date "$file")"
    if [[ "$filename" == "$prefix"_* ]]; then
      print "⏭️  $filename"
    else
      mv "$file" "$dirname/${prefix}_${filename}" || return 1
      print "✅ $filename → ${prefix}_${filename}"
    fi
  done
}

encode_files() {
  local source="$1" target="$2" mode="$3" file filename prefix output i=1
  list_videos "$source"
  local files=( "${REPLY[@]}" ) total=${#REPLY[@]}
  (( total )) || { print "⚠️ Aucune vidéo trouvée."; return 0; }
  for file in "${files[@]}"; do
    filename="$(basename "$file")"
    prefix=""
    [[ "$mode" == "rename" ]] && prefix="$(get_formatted_date "$file")_"
    output="$target/${prefix}${filename%.*}.mov"
    print "\n🚀 Encodage H.265 10-bit : $filename"
    run_ffmpeg_progress "$file" "$i" "$total" -y -i "$file" \
      -c:v hevc_videotoolbox -profile:v main10 -pix_fmt p010le -q:v 65 -tag:v hvc1 \
      -c:a aac -b:a 192k "$output" || { print "\n❌ Échec ffmpeg pour $filename"; return 1; }
    apply_original_dates "$file" "$output"
    (( i++ ))
  done
}

apply_lut() {
  local source="$1" target="$2" lut="$3" profile="$4" file filename output i=1
  [[ -f "$lut" ]] || { print "❌ LUT introuvable : $lut (modifiez son chemin en haut du script)"; return 1; }
  list_videos "$source"
  local files=( "${REPLY[@]}" ) total=${#REPLY[@]}
  (( total )) || { print "⚠️ Aucune vidéo trouvée."; return 0; }
  for file in "${files[@]}"; do
    filename="$(basename "$file")"
    output="$target/${filename%.*}_${profile}_lutted.mov"
    print "\n🎨 Profil $profile : $filename"
    run_ffmpeg_progress "$file" "$i" "$total" -y -i "$file" \
      -vf "format=p010le,lut3d='$lut'" \
      -c:v hevc_videotoolbox -profile:v main10 -pix_fmt p010le -q:v 60 -tag:v hvc1 \
      -c:a aac -b:a 192k "$output" || { print "\n❌ Échec ffmpeg pour $filename"; return 1; }
    apply_original_dates "$file" "$output"
    (( i++ ))
  done
}

select_profile() {
  print "\nProfil caméra pour l'application de LUT :"
  print "1) S-Log2"
  print "2) S-Log3"
  print "3) DJI Osmo Action 4 — D-Log M"
  read "profile_choice?Votre choix [1-3] : "
  case "$profile_choice" in
    1) PROFILE_NAME="S-Log2"; PROFILE_LUT="$LUT_SLOG2" ;;
    2) PROFILE_NAME="S-Log3"; PROFILE_LUT="$LUT_SLOG3" ;;
    3) PROFILE_NAME="D-Log-M"; PROFILE_LUT="$LUT_DLOGM" ;;
    *) print "❌ Choix invalide."; return 1 ;;
  esac
}

# --- Boucle du menu ----------------------------------------------------------
clear
print "=========================================="
print "       SCRIPT VIDÉO — MENU PROFILS"
print "=========================================="
while true; do
  CURRENT_SOURCE="$(get_source_folder)" && break
  print "Réessayez avec un dossier existant. (Ctrl+C pour quitter)"
done
print "✅ Source : $CURRENT_SOURCE"

while true; do
  print "\nAction ?"
  print "1) Rename (renommer avec la date)"
  print "2) Reencode (H.265 10-bit)"
  print "3) Rename & Reencode (H.265 10-bit)"
  print "4) Apply LUT (choisir S-Log2, S-Log3 ou DJI D-Log M)"
  print "5) Quitter"
  read "choice?Votre choix [1-5] : "
  TARGET_EXPORT=""
  case "$choice" in
    1)
      print "\n🚀 RENOMMAGE..."
      rename_files "$CURRENT_SOURCE"
      ;;
    2|3|4)
      if [[ "$choice" == 4 ]]; then
        select_profile || { read "?Entrée pour revenir au menu..."; continue; }
      fi
      TARGET_EXPORT="$(setup_export_folder "$CURRENT_SOURCE")" || { print "❌ Dossier export invalide."; read "?Entrée pour revenir au menu..."; continue; }
      case "$choice" in
        2) encode_files "$CURRENT_SOURCE" "$TARGET_EXPORT" "plain" ;;
        3) encode_files "$CURRENT_SOURCE" "$TARGET_EXPORT" "rename" ;;
        4) apply_lut "$CURRENT_SOURCE" "$TARGET_EXPORT" "$PROFILE_LUT" "$PROFILE_NAME" ;;
      esac
      [[ -n "$TARGET_EXPORT" ]] && open "$TARGET_EXPORT" 2>/dev/null
      ;;
    5|q|Q) print "👋 Au revoir."; exit 0 ;;
    *) print "❌ Choix invalide." ;;
  esac
  print "\n✨ Opération terminée."
  read "?Entrée pour revenir au menu..."
  clear
done
