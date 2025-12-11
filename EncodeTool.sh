#!/bin/zsh

# ==============================================================================
# SCRIPT VIDÉO V4 (Fix 10-bit Pixel Format)
# ==============================================================================

# --- CONFIGURATION ---

SOURCE_DIR=""
EXPORT_DIR=""

# Chemin LUT (Vérifie bien ce chemin)
LUT_PATH='/path/to/your/LUT.cube' # ⚠️ EDIT THIS PATH

# ------------------------------------------------------------------------------

# Détection de ffmpeg
if [[ -x "/opt/homebrew/bin/ffmpeg" ]]; then
    FFMPEG_CMD="/opt/homebrew/bin/ffmpeg"
elif command -v ffmpeg &> /dev/null; then
    FFMPEG_CMD="ffmpeg"
else
    echo "❌ ERREUR : ffmpeg n'est pas installé."
    exit 1
fi

# Vérification de ffprobe pour la durée (optionnel mais recommandé pour la barre de progression)
if ! command -v ffprobe &> /dev/null; then
    echo "⚠️  ATTENTION : ffprobe introuvable. L'estimation du temps restant ne sera pas disponible."
fi

function clean_path() {
    local p="$1"
    p="${p%\"}" ; p="${p#\"}" ; p="${p%\'}" ; p="${p#\'}"
    echo "$p" | xargs
}

function get_source_folder() {
    if [[ -n "$SOURCE_DIR" ]]; then
        if [[ ! -d "$SOURCE_DIR" ]]; then
            echo "❌ ERREUR Source invalide" >&2; exit 1
        fi
        echo "$SOURCE_DIR"
    else
        echo "\n📂 GLISSEZ-DÉPOSEZ le dossier SOURCE ici :" >&2
        read RAW_FOLDER
        FOLDER=$(clean_path "$RAW_FOLDER")
        if [[ ! -d "$FOLDER" ]]; then echo "❌ Dossier invalide" >&2; exit 1; fi
        echo "$FOLDER"
    fi
}

function setup_export_folder() {
    local source="$1"
    if [[ -n "$EXPORT_DIR" ]]; then
        mkdir -p "$EXPORT_DIR"
        echo "$EXPORT_DIR"
    else
        echo "\n📂 GLISSEZ-DÉPOSEZ le dossier EXPORT (ou Entrée pour auto) :" >&2
        read RAW_FOLDER
        FOLDER=$(clean_path "$RAW_FOLDER")
        if [[ -z "$FOLDER" ]]; then
             local auto_export="${source}/export"
             mkdir -p "$auto_export"
             echo "$auto_export"
        else
             mkdir -p "$FOLDER"
             echo "$FOLDER"
        fi
    fi
}

function get_formatted_date() { stat -f "%Sm" -t "%Y-%m-%d_%H%M" "$1"; }
function apply_original_dates() {
    local src="$1"; local dest="$2"
    local ctime=$(stat -f "%SB" -t "%Y%m%d%H%M.%S" "$src")
    local mtime=$(stat -f "%Sm" -t "%Y%m%d%H%M.%S" "$src")
    touch -t "$ctime" "$dest"; touch -mt "$mtime" "$dest"
}

# --- FONCTIONS BARRE DE PROGRESSION ---

function get_duration() {
    ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 "$1" 2>/dev/null
}

function run_ffmpeg_progress() {
    local input_file="$1"
    local current_idx="$2"
    local total_files="$3"
    shift 3

    local filename=$(basename "$input_file")
    local duration=$(get_duration "$input_file")
    local duration_us=0
    local time_us=0
    local speed=0
    local fps=0
    local bitrate="0"

    if [[ -n "$duration" ]]; then
        duration_us=$(printf "%.0f" $((duration * 1000000))) 2>/dev/null
    fi

    # Couleurs
    local BLUE="\033[34m"
    local CYAN="\033[36m"
    local GREEN="\033[32m"
    local YELLOW="\033[33m"
    local RESET="\033[0m"

    # Trap pour restaurer le curseur en cas d'interruption (Ctrl+C)
    trap 'tput cnorm; exit 1' INT TERM

    separator() {
        printf "${BLUE}──────────────────────────────────────────────────────────${RESET}\n"
    }

    make_bar() {
        local pct=$1
        local width=25
        local filled=$(( pct * width / 100 ))
        local empty=$(( width - filled ))
        local i # Variable locale pour éviter le conflit avec la boucle principale

        printf "["
        # Utilisation de boucles zsh natives au lieu de 'seq' pour compatibilité
        printf "${GREEN}"
        for ((i=0; i<filled; i++)); do printf "█"; done
        printf "${RESET}"
        for ((i=0; i<empty; i++)); do printf "░"; done
        printf "]"
    }

    tput civis

    # État pour le premier affichage
    local first_run=1

    "$FFMPEG_CMD" "$@" -progress pipe:1 -nostats | while read -r line; do
        local key=${line%%=*}
        local value=${line#*=}

        case "$key" in
            out_time_us) time_us=$value ;;
            speed) speed=${value%x} ;;
            fps) fps=$value ;;
            bitrate) bitrate=$value ;;
            progress)
                if [[ "$value" == "continue" ]]; then

                    # Utilisation d'awk pour les calculs (Robuste)
                    local dur=${duration_us:-0}
                    local cur=${time_us:-0}
                    local spd=${speed:-0}

                    stats=$(awk -v dur="$dur" -v cur="$cur" -v spd="$spd" '
                    BEGIN {
                        dur += 0; cur += 0; spd += 0;
                        if (dur <= 0) dur = 1;

                        pct = (cur * 100) / dur;
                        if (pct > 100) pct = 100;
                        if (pct < 0) pct = 0;

                        eta_str = "--";
                        if (spd > 0.0001) {
                            rem_us = dur - cur;
                            if (rem_us < 0) rem_us = 0;
                            rem_s = rem_us / 1000000.0 / spd;

                            h = int(rem_s / 3600);
                            rem_s = rem_s % 3600;
                            m = int(rem_s / 60);
                            s = int(rem_s % 60);

                            eta_str = sprintf("%02d:%02d:%02d", h, m, s);
                        }

                        printf "%d|%s", pct, eta_str;
                    }')

                    local percent=0
                    local eta="--"
                    IFS='|' read -r percent eta <<< "$stats"

                    local global_percent=$(awk -v idx="$current_idx" -v t="$total_files" -v p="$percent" '
                        BEGIN { 
                            t += 0; if (t<=0) t=1;
                            printf "%d", ((idx-1)*100 + p)/t 
                        }
                    ')

                    # Affichage STABLE (Cursor Up)
                    # Le bloc fait exactement 10 lignes
                    
                    if [[ $first_run -eq 0 ]]; then
                        # Remonter de 10 lignes pour écraser le précédent
                        printf "\033[10A"
                    fi
                    first_run=0

                    separator
                    printf " Processing ${current_idx}/${total_files} : ${YELLOW}%s${RESET}\033[K\n" "$filename"
                    separator

                    printf " Progress : "
                    make_bar "$percent"
                    printf " ${YELLOW}%d%%%s\033[K\n" "$percent" "$RESET"

                    printf " ETA      : ${CYAN}%s${RESET}\033[K\n" "$eta"
                    printf " FPS      : ${CYAN}%s${RESET}\033[K\n" "$fps"
                    printf " Bitrate  : ${CYAN}%s${RESET}\033[K\n" "$bitrate"

                    separator
                    printf "\033[K\n" # Ligne vide

                    printf " TOTAL : "
                    make_bar "$global_percent"
                    printf " ${YELLOW}%d%%%s (%d/%d)\033[K\n" "$global_percent" "$RESET" "$current_idx" "$total_files"
                fi
                ;;
        esac
    done

    # Fin du fichier : On nettoie le bloc de 10 lignes et on affiche juste le succès
    # On remonte de 10 lignes (taille du bloc)
    printf "\033[10A"
    # On efface tout ce qui est en dessous
    printf "\033[J"
    
    # On affiche le résumé propre
    printf "✅ ${GREEN}%s${RESET} (Terminé)\n" "$filename"
    
    tput cnorm
}

# --- INITIALISATION ---

clear
echo "=========================================="
echo "   SCRIPT VIDÉO V4 (Fix 10-bit)"
echo "=========================================="

CURRENT_SOURCE=$(get_source_folder)
[[ -z "$CURRENT_SOURCE" ]] && exit 1

echo "✅ Source : $CURRENT_SOURCE"

setopt nullglob
files=("$CURRENT_SOURCE"/*.(mp4|MP4|mov|MOV))
num_files=${#files[@]}
unsetopt nullglob

if [[ $num_files -eq 0 ]]; then echo "⚠️  Aucune vidéo trouvée."; exit 1; fi
echo "🎞️  Vidéos trouvées : $num_files"

echo "\nAction ?"
echo "1) Rename (Renommer avec date)"
echo "2) Reencode (H265 10-bit)"
echo "3) Rename & Reencode (10-bit)"
echo "4) Apply LUT (S-Log2 -> Rec709 10-bit)"
echo "5) Quitter"

read "choice?Votre choix [1-5]: "

case $choice in
    1)
        echo "\n🚀 RENOMMAGE..."
        for file in $files; do
            dirname=$(dirname "$file")
            filename=$(basename "$file")
            date_prefix=$(get_formatted_date "$file")
            if [[ "$filename" == "$date_prefix"* ]]; then
                echo "⏭️  $filename"
            else
                mv "$file" "${dirname}/${date_prefix}_${filename}"
                echo "✅ $filename -> ${date_prefix}_${filename}"
            fi
        done
        ;;
    2)
        TARGET_EXPORT=$(setup_export_folder "$CURRENT_SOURCE")
        echo "\n🚀 ENCODAGE 10-BIT..."
        i=1
        for file in $files; do
            filename=$(basename "$file")
            output="${TARGET_EXPORT}/${filename%.*}.mov"
            run_ffmpeg_progress "$file" "$i" "$num_files" -y -i "$file" \
            -c:v hevc_videotoolbox -profile:v main10 -pix_fmt p010le -q:v 65 -tag:v hvc1 \
            -c:a aac -b:a 192k \
            "$output" -loglevel error
            apply_original_dates "$file" "$output"
            ((i++))
        done
        ;;
    3)
        TARGET_EXPORT=$(setup_export_folder "$CURRENT_SOURCE")
        echo "\n🚀 TEXTE & ENCODAGE 10-BIT..."
        i=1
        for file in $files; do
            filename=$(basename "$file")
            date_prefix=$(get_formatted_date "$file")
            output="${TARGET_EXPORT}/${date_prefix}_${filename%.*}.mov"
            run_ffmpeg_progress "$file" "$i" "$num_files" -y -i "$file" \
            -c:v hevc_videotoolbox -profile:v main10 -pix_fmt p010le -q:v 65 -tag:v hvc1 \
            -c:a aac -b:a 192k \
            "$output" -loglevel error
            apply_original_dates "$file" "$output"
            ((i++))
        done
        ;;
    4)
        if [ ! -f "$LUT_PATH" ]; then echo "❌ ERREUR LUT"; exit 1; fi
        TARGET_EXPORT=$(setup_export_folder "$CURRENT_SOURCE")
        echo "\n🚀 APPLICATION LUT (High Quality)..."
        i=1
        for file in $files; do
            filename=$(basename "$file")
            output="${TARGET_EXPORT}/${filename%.*}_lutted.mov"
            # Le format=p010le force le traitement en 10bits AVANT la LUT
            # Le -pix_fmt p010le force le format de sortie POUR l'encodeur
            run_ffmpeg_progress "$file" "$i" "$num_files" -y -i "$file" \
            -vf "format=p010le,lut3d='$LUT_PATH'" \
            -c:v hevc_videotoolbox -profile:v main10 -pix_fmt p010le -q:v 60 -tag:v hvc1 \
            -c:a aac -b:a 192k \
            "$output" -loglevel error
            apply_original_dates "$file" "$output"
            ((i++))
        done
        ;;
    5) exit 0 ;;
    *) echo "❌" ;;
esac

echo "\n✨ Terminé !"
if [[ -n "$TARGET_EXPORT" ]]; then open "$TARGET_EXPORT"; fi