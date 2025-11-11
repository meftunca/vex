#!/usr/bin/env python3
"""
mk_spec_light.py
Markdown specification dosyalarını context koruyarak optimize eder.
"""

import os
import sys
import re
from pathlib import Path
from typing import List, Tuple

# Yapılandırma
PRESERVE_ALL_CODE = os.getenv("PRESERVE_ALL_CODE", "1") == "1"  # Tüm kodları koru
SRC_DIR = sys.argv[1] if len(sys.argv) > 1 else "Specifications"
DST_DIR = sys.argv[2] if len(sys.argv) > 2 else "spec_light"


def process_markdown_content(content: str) -> str:
    """Markdown içeriğini optimize et"""
    lines = content.split('\n')
    output = []
    in_code_block = False
    code_buffer = []
    code_lang = ""
    skip_yaml = False
    skip_toc = False
    
    for i, line in enumerate(lines):
        # YAML front matter'ı atla
        if i == 0 and line.strip() == "---":
            skip_yaml = True
            continue
        if skip_yaml:
            if line.strip() == "---":
                skip_yaml = False
            continue
        
        # Table of Contents başlangıcını tespit et ve atla
        if re.match(r'^##?\s+Table of Contents', line, re.IGNORECASE):
            skip_toc = True
            continue
        
        # TOC bitişi: bir sonraki ## veya ### başlık
        if skip_toc and re.match(r'^##\s+\w', line):
            skip_toc = False
            # Bu satırı işlemeye devam et (başlık olarak)
        
        if skip_toc:
            continue
        
        # HTML yorumlarını atla
        if '<!--' in line and '-->' in line:
            line = re.sub(r'<!--.*?-->', '', line)
        elif '<!--' in line:
            continue
        elif '-->' in line:
            continue
            
        # Code fence kontrolü - TÜM KODLARI KORU
        if line.strip().startswith('```'):
            if not in_code_block:
                # Kod bloğu başlangıcı
                in_code_block = True
                code_lang = line.strip()[3:].strip()
                code_buffer = [line]
            else:
                # Kod bloğu bitişi - tüm kodu yazdır
                code_buffer.append(line)
                output.extend(code_buffer)
                in_code_block = False
                code_buffer = []
                code_lang = ""
            continue
        
        if in_code_block:
            code_buffer.append(line)
            continue
        
        # Badge/shield görsellerini atla
        if re.match(r'^\s*!\[.*\]\(.*(?:shields\.io|badge\.).*\)\s*$', line):
            continue
        
        # Görselleri alt yazıya indir: ![alt](url) -> [Image: alt]
        line = re.sub(r'!\[([^\]]*)\]\([^)]+\)', r'[Image: \1]', line)
        
        # Linkleri sadeleştir: [text](url) -> text
        line = re.sub(r'\[([^\]]+)\]\([^)]+\)', r'\1', line)
        
        # Emojileri kaldır (GitHub Spaces için)
        line = re.sub(r'[\U0001F300-\U0001F9FF]', '', line)  # Emoji blokları
        line = re.sub(r'[\u2600-\u26FF]', '', line)  # Miscellaneous Symbols
        line = re.sub(r'[\u2700-\u27BF]', '', line)  # Dingbats
        
        # Bold/italic formatlamayı kaldır (agresif boyut düşürme)
        line = re.sub(r'\*\*\*([^*]+)\*\*\*', r'\1', line)  # ***bold+italic***
        line = re.sub(r'\*\*([^*]+)\*\*', r'\1', line)      # **bold**
        line = re.sub(r'\*([^*]+)\*', r'\1', line)          # *italic*
        line = re.sub(r'___([^_]+)___', r'\1', line)        # ___bold+italic___
        line = re.sub(r'__([^_]+)__', r'\1', line)          # __bold__
        line = re.sub(r'_([^_]+)_', r'\1', line)            # _italic_
        
        # Başlıkları normalize et (max ###)
        if line.lstrip().startswith('#'):
            # Trailing # karakterlerini temizle
            line = re.sub(r'\s*#+\s*$', '', line)
            # Derin başlıkları ### seviyesine indir
            line = re.sub(r'^(\s*)#{4,}(\s+)', r'\1###\2', line)
        
        # Tabloları listeye çevir
        if '|' in line and not in_code_block and '`' not in line:
            # Table separator satırlarını atla
            if re.match(r'^\s*\|?\s*[-:]+\s*(\|\s*[-:]+\s*)+\|?\s*$', line):
                continue
            # Tablo satırlarını listeye çevir
            if re.match(r'^\s*\|.*\|\s*$', line):
                cells = [cell.strip() for cell in line.strip().strip('|').split('|')]
                cells = [c for c in cells if c]  # Boş hücreleri atla
                if cells:
                    output.append('• ' + ' — '.join(cells))
                continue
        
        # Fazla boşlukları temizle
        line = re.sub(r'[ \t]+', ' ', line)
        
        output.append(line)
    
    # Art arda gelen boş satırları azalt (max 2 boş satır)
    result = []
    empty_count = 0
    for line in output:
        if line.strip() == '':
            empty_count += 1
            if empty_count <= 2:
                result.append(line)
        else:
            empty_count = 0
            result.append(line)
    
    # Başta ve sondaki boş satırları temizle
    while result and result[0].strip() == '':
        result.pop(0)
    while result and result[-1].strip() == '':
        result.pop()
    
    # Son bölümdeki "Maintained by" footer'ını temizle (sadece dosya sonunda)
    # Son 10 satıra bak, eğer "Maintained by" varsa oradan sonrasını kes
    final_result = result
    for i in range(max(0, len(result) - 15), len(result)):
        if i < len(result):
            line = result[i]
            if re.search(r'^\*\*Maintained by\*\*|^Maintained by:', line, re.IGNORECASE):
                # Bu satırdan sonrasını kes
                final_result = result[:i]
                # Önceki --- satırını da kaldır
                if final_result and final_result[-1].strip() == '---':
                    final_result.pop()
                break
    
    return '\n'.join(final_result)


def process_file(src_path: Path, dst_path: Path):
    """Tek bir markdown dosyasını işle"""
    try:
        # Dosyayı oku
        with open(src_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # İçeriği optimize et
        optimized = process_markdown_content(content)
        
        # Hedef dizini oluştur
        dst_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Optimized içeriği yaz
        with open(dst_path, 'w', encoding='utf-8') as f:
            f.write(optimized)
        
        print(f"✔  {src_path} -> {dst_path}")
        
    except Exception as e:
        print(f"✗  {src_path}: {e}", file=sys.stderr)


def main():
    """Ana işlev"""
    src_dir = Path(SRC_DIR)
    dst_dir = Path(DST_DIR)
    
    if not src_dir.exists():
        print(f"Kaynak klasör bulunamadı: {src_dir}", file=sys.stderr)
        sys.exit(1)
    
    # Hedef dizini oluştur
    dst_dir.mkdir(parents=True, exist_ok=True)
    
    # Tüm .md dosyalarını bul ve işle
    md_files = sorted(src_dir.rglob("*.md"))
    
    for src_path in md_files:
        rel_path = src_path.relative_to(src_dir)
        dst_path = dst_dir / rel_path
        process_file(src_path, dst_path)
    
    print()
    print(f"Bitti. Çıktılar: {dst_dir}")
    print("Optimizasyonlar:")
    print("  - Tüm kod blokları korundu (PRESERVE_ALL_CODE=1)")
    print("  - Emojiler kaldırıldı")
    print("  - Bold/italic formatları temizlendi")
    print("  - TOC ve footer'lar temizlendi")


if __name__ == "__main__":
    main()
