#!/usr/bin/env bash
#
# Recupere les modeles de reranking cross-encoder (ONNX, quantifies INT8).
#
# Les modeles ne sont pas versionnes : ~280 Mo chacun. En leur absence, le reranking se desactive
# proprement et la recherche continue de fonctionner — ce script est donc optionnel, mais la
# qualite du classement s'en ressent.
#
# Ecart avec le script de l'application Spring : `fastembed` charge le tokenizer via la
# bibliotheque HuggingFace `tokenizers`, qui exige QUATRE fichiers et non un seul. Deux d'entre eux
# sont lus avec un `.expect()` — `tokenizer_config.json` doit porter `model_max_length` et
# `pad_token`, sans quoi le chargement panique au lieu de degrader.
#
#   ./scripts/download-rerankers.sh            # modele par defaut (bge-base)
#   ./scripts/download-rerankers.sh --all      # les trois variantes
#
set -euo pipefail

DEST="$(cd "$(dirname "$0")" && pwd)/rerankers"

# nom|url du modele|prefixe du depot HuggingFace pour les fichiers de tokenizer
#
# bge-base est le defaut, retenu cote Java apres mesure : 2,9x plus rapide que bge-reranker-v2-m3
# pour la meilleure fidelite de classement. mminilm est le repli quand le CPU ne suit pas
# (118 Mo, 7,3x plus rapide, classement plus grossier).
MODELS=(
  "bge-base|https://huggingface.co/onnx-community/bge-reranker-base-ONNX/resolve/main/onnx/model_int8.onnx|https://huggingface.co/onnx-community/bge-reranker-base-ONNX/resolve/main"
  "jina-v2|https://huggingface.co/jinaai/jina-reranker-v2-base-multilingual/resolve/main/onnx/model_int8.onnx|https://huggingface.co/jinaai/jina-reranker-v2-base-multilingual/resolve/main"
  "mminilm|https://huggingface.co/cross-encoder/mmarco-mMiniLMv2-L12-H384-v1/resolve/main/onnx/model_qint8_avx512_vnni.onnx|https://huggingface.co/cross-encoder/mmarco-mMiniLMv2-L12-H384-v1/resolve/main"
)

# Les quatre fichiers attendus par `load_tokenizer` de fastembed.
TOKENIZER_FILES=(tokenizer.json config.json special_tokens_map.json tokenizer_config.json)

fetch() {
  local url="$1" out="$2"
  # `--fail` : sans lui, curl ecrit une page d'erreur HTML dans le fichier et le chargement
  # echouerait plus tard sur un JSON illisible, loin de la cause.
  curl -sSL --fail --retry 3 -o "$out" "$url"
}

download_model() {
  local name="$1" model_url="$2" repo="$3"
  local dir="$DEST/$name"
  mkdir -p "$dir"

  echo "== $name"
  if [ -f "$dir/model.onnx" ]; then
    echo "  modele    : deja present, ignore"
  else
    echo "  modele    : telechargement (~280 Mo)..."
    fetch "$model_url" "$dir/model.onnx"
    echo "  modele    : $(wc -c < "$dir/model.onnx") octets"
  fi

  for file in "${TOKENIZER_FILES[@]}"; do
    if [ -f "$dir/$file" ]; then
      echo "  $file : deja present"
    else
      printf '  %-24s : ' "$file"
      fetch "$repo/$file" "$dir/$file"
      echo "$(wc -c < "$dir/$file") octets"
    fi
  done
}

WANTED="bge-base"
[ "${1:-}" = "--all" ] && WANTED="all"

for entry in "${MODELS[@]}"; do
  IFS='|' read -r name model_url repo <<< "$entry"
  if [ "$WANTED" = "all" ] || [ "$WANTED" = "$name" ]; then
    download_model "$name" "$model_url" "$repo"
  fi
done

echo
echo "Termine. Le serveur charge par defaut $DEST/bge-base ;"
echo "surcharger avec RERANKER_MODEL_DIR pour en choisir un autre."
