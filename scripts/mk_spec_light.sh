#!/usr/bin/env bash
# mk_spec_light.sh
# macOS uyumlu (BSD sed/awk). specifications/ altındaki karmaşık .md dosyalarını
# sadeleştirip aynı klasör hiyerarşisini koruyarak spec_light/ altına yazar.

set -euo pipefail

SRC_DIR="${1:-Specifications}"
DST_DIR="${2:-spec_light}"
MAX_CODE_LINES="${MAX_CODE_LINES:-15}"          # Bu kadar satırı aşan code fence'ler özetlenir
PRESERVE_SMALL_CODE="${PRESERVE_SMALL_CODE:-1}" # 1 ise küçük kod blokları olduğu gibi korunur

if [[ ! -d "$SRC_DIR" ]]; then
  echo "Kaynak klasör bulunamadı: $SRC_DIR" >&2
  exit 1
fi

mkdir -p "$DST_DIR"

# Güvenli dosya gezintisi
find "$SRC_DIR" -type f -name "*.md" -print0 | while IFS= read -r -d '' SRC_FILE; do
  REL_PATH="${SRC_FILE#$SRC_DIR/}"
  DST_PATH="$DST_DIR/$REL_PATH"
  mkdir -p "$(dirname "$DST_PATH")"

  /usr/bin/awk -v MAX_CODE_LINES="$MAX_CODE_LINES" -v PRESERVE_SMALL_CODE="$PRESERVE_SMALL_CODE" '
    BEGIN {
      in_yaml = 0
      in_html_comment = 0
      in_code = 0
      code_count = 0
      code_lang = ""
      code_buf_len = 0
    }

    # küçük yardımcılar
    function trim(s) { sub(/^[[:space:]]+/, "", s); sub(/[[:space:]]+$/, "", s); return s }

    # buffered yazdırma
    function flush_code_block(is_small) {
      if (is_small) {
        lang = trim(code_lang)
        if (lang == "```" || lang == "") lang = ""
        # açılış fence
        if (lang == "") { print "```" } else { print "```" lang }
        # içerik
        for (i=1; i<=code_buf_len; i++) print code_buf[i]
        # kapanış fence
        print "```"
      } else {
        lang = trim(code_lang); if (lang == "```" || lang == "") lang="(unknown)"
        printf("[%d lines code: %s]\n", code_count, lang)
      }
      # buffer temizle
      code_buf_len = 0
    }

    # YAML front-matter: dosyanın en başında --- ... ---
    NR==1 && $0 ~ /^---[[:space:]]*$/ { in_yaml=1; next }
    in_yaml {
      if ($0 ~ /^---[[:space:]]*$/) { in_yaml=0 }
      next
    }

    # HTML yorumları: <!-- ... -->
    {
      line = $0
      if (in_html_comment) {
        if (line ~ /-->/) { sub(/.*-->/, "", line); in_html_comment=0 } else { next }
      }
      while (line ~ /<!--/) {
        if (line ~ /<!--.*-->/) {
          gsub(/<!--.*?-->/, "", line)
        } else {
          sub(/<!--.*/, "", line)
          in_html_comment=1
          break
        }
      }
      $0 = line
    }

    # code fence takibi: ```lang
    /^```/ {
      if (!in_code) {
        in_code=1; code_count=0; code_lang=$0; code_buf_len=0
        next
      } else {
        # kapanış
        if (PRESERVE_SMALL_CODE=="1" && code_count <= MAX_CODE_LINES) {
          flush_code_block(1)
        } else {
          flush_code_block(0)
        }
        in_code=0; code_lang=""; next
      }
    }

    # kod bloğu içinde: satırları bufferla ve say
    in_code {
      code_count++
      if (PRESERVE_SMALL_CODE=="1") { code_buf_len++; code_buf[code_buf_len]=$0 }
      next
    }

    # rozet/badge + tek satırlık görselleri at
    /^[[:space:]]*!\[[^]]*\]\([^)]*\)[[:space:]]*$/ { next }
    $0 ~ /(shields\.io|badge\.)/ && $0 ~ /!\[[^]]*\]\([^)]*\)/ { next }

    # Görselleri alt yazıya indir: ![alt](url) -> [Image: alt]
    { gsub(/!\[([^\]]*)\]\([^)]+\)/, "[Image: \\1]") }

    # Linkleri sadeleştir: [metin](url) -> metin
    { gsub(/\[([^\]]+)\]\(([^)]+)\)/, "\\1") }

    # Başlık normalize: en fazla ###
    /^[[:space:]]*#/ {
      sub(/[[:space:]]*#+[[:space:]]*$/, "", $0)
      sub(/^[[:space:]]*######[[:space:]]*/, "### ", $0)
      sub(/^[[:space:]]*#####[[:space:]]*/,  "### ", $0)
      sub(/^[[:space:]]*####[[:space:]]*/,   "### ", $0)
    }

    # Basit tablo satırlarını listeye çevir (sadece | ile başlayan/biten satırlar)
    !in_code {
      if ($0 ~ /^\|.*\|$/ && $0 !~ /`/) {
        # separator satırı atla
        if ($0 ~ /^[[:space:]]*\|?[[:space:]-:]+(\|[[:space:]-:]+)+\|?[[:space:]]*$/) next
        line=$0
        sub(/^[[:space:]]*\|[[:space:]]*/, "", line)
        sub(/[[:space:]]*\|[[:space:]]*$/, "", line)
        n=split(line, cols, /\|/)
        out="• "
        for (i=1;i<=n;i++) {
          c=trim(cols[i])
          if (c != "") out = out ((i==1)?"":" — ") c
        }
        print out
        next
      }
    }

    # Unicode ve özel karakterleri koru, fazla boşlukları sadeleştir
    { gsub(/[ \t]+/, " ") }

    # Boş/whitespace-only satırları kısalt
    /^[[:space:]]*$/ { print ""; next }

    # yaz
    { print $0 }
  ' "$SRC_FILE" \
  | /usr/bin/sed -E '
      # art arda 3+ boş satırı 2 boş satıra indir
      :a
      /^[[:space:]]*$/{
        N
        /^\n[[:space:]]*$/{
          N
          /^\n\n[[:space:]]*$/{
            s/\n\n\n+/\n\n/
            ba
          }
          P
          D
        }
      }
      # baştaki boş satırı temizle
      1 {
        /^[[:space:]]*$/d
      }
    ' > "$DST_PATH"

  echo "✔  ${SRC_FILE} -> ${DST_PATH}"
done

echo ""
echo "Bitti. Çıktılar: $DST_DIR"
echo "İpuçları:"
echo "  - MAX_CODE_LINES=8 ./scripts/mk_spec_light.sh       # 8+ satır kod bloklarını özetle"
echo "  - PRESERVE_SMALL_CODE=0 ./scripts/mk_spec_light.sh  # tüm kod bloklarını özetle"

