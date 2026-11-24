"""Monta o relatório final (seções 1 a 20) no modelo do professor.

- Parte do docx da Entrega 2 (o original não é alterado), que já tem as seções 1 a 7 e os títulos
  de 8 a 20 com o texto-modelo.
- Atualiza a tabela 7.2 e as regras da seção 7 que mudaram na implementação.
- Troca o texto-modelo de cada seção 8 a 20 pelo conteúdo de documentos/relatorio/secoes/NN-*.md.
  Imagens em Markdown (![legenda](caminho.png)) viram figuras numeradas; as muito largas são
  giradas para caber melhor na página.

Uso: python3 ferramentas/montar_relatorio.py   (precisa de Pillow: pip install pillow)
"""
import html
import pathlib
import re
import shutil
import tempfile
import zipfile

from PIL import Image

RAIZ = pathlib.Path(__file__).resolve().parent.parent
RELATORIO = RAIZ / "documentos" / "relatorio"
ORIGEM = RAIZ / "documentos" / "entregues" / "Equipe 11 - Entrega 2 - Modelagem da Solução.docx"
DESTINO = RELATORIO / "saida" / "Equipe 11 - Relatório Final (Entregas 3, 4 e 5).docx"
SECOES = RELATORIO / "secoes"
TITULOS = {
    8: "8. Algoritmo proposto", 9: "9. Estruturas de dados utilizadas", 10: "10. Organização do programa e POO",
    11: "11. Tecnologias utilizadas", 12: "12. Implementação", 13: "13. Persistência e armazenamento de dados",
    14: "14. Tratamento de erros e exceções", 15: "15. Testes realizados", 16: "16. Resultados e análise",
    17: "17. Limitações", 18: "18. Possíveis melhorias", 19: "19. Conclusão", 20: "20. Referências",
}

# Textos das seções 1 a 7 que mudaram com a implementação: (texto antigo, texto novo)
AJUSTES_TEXTO = [
    ("Apenas o SuperAdmin pode gerenciar (ativar, bloquear ou remover) organizadores.",
     "Organizadores podem criar a própria conta (autocadastro); apenas o SuperAdmin pode ativar, "
     "bloquear, editar ou remover organizadores."),
]

# Tabela 7.2: (texto antigo da célula, texto novo)
TABELA_72 = [
    ("id, nome, documento, e-mail, senha, perfil",
     "id, nome, documento, e-mail, senha (armazenada como hash), perfil"),
    ("id, razão social, CNPJ, contato, status (ativo/bloqueado)",
     "id, razão social, CNPJ, contato, status (ativo/bloqueado), id do usuário"),
    ("inteiro, texto, texto, texto, texto</w:t>",
     "inteiro, texto, texto, texto, texto, inteiro</w:t>"),
    ("id, nome, local, data, horário, capacidade, status, id do organizador",
     "id, nome, descrição, local, data, horário, capacidade, total vendido, permite cancelamento, status, id do organizador"),
    ("inteiro, texto, texto, data, hora, inteiro, texto, inteiro",
     "inteiro, texto, texto, texto, data, hora, inteiro, inteiro, lógico, texto, inteiro"),
    ("id, categoria, preço, quantidade disponível, id do evento",
     "id, categoria, preço, quantidade total, quantidade disponível, com assento marcado, id do evento"),
    ("inteiro, texto, real, inteiro, inteiro</w:t>",
     "inteiro, texto, real, inteiro, inteiro, lógico, inteiro</w:t>"),
    ("id, setor, fileira, número, situação (livre/ocupado), id do evento",
     "id, setor, fileira, número, situação (livre/ocupado), id da categoria, id do evento"),
    ("inteiro, texto, texto, inteiro, texto, inteiro</w:t>",
     "inteiro, texto, texto, inteiro, texto, inteiro, inteiro</w:t>"),
    ("id, data da compra, valor total, status, id do cliente, id do ingresso, id do assento",
     "id, data da compra, valor total, tipo (inteira/meia), forma de pagamento, status (confirmada/cancelada/utilizada), id do cliente, id do ingresso, id do assento (opcional)"),
    ("inteiro, data/hora, real, texto, inteiro, inteiro, inteiro",
     "inteiro, data/hora, real, inteiro, inteiro, texto, inteiro, inteiro, inteiro"),
]

FONTE = ('<w:rFonts w:ascii="Helvetica Neue" w:hAnsi="Helvetica Neue" '
         'w:eastAsia="Times New Roman" w:cs="Times New Roman"/>')
MONO = '<w:rFonts w:ascii="Courier New" w:hAnsi="Courier New" w:eastAsia="Courier New" w:cs="Courier New"/>'
LANG = '<w:lang w:val="pt-BR"/>'
LARGURA_UTIL = 9360  # dxa (página Carta 12240 − margens 1440 + 1440)


def esc(t: str) -> str:
    return html.escape(t, quote=False)


def runs(texto: str, tamanho=24, cor=None) -> str:
    """Converte **negrito** e `código` em runs do Word."""
    saida = []
    for parte in re.split(r"(\*\*[^*]+\*\*|`[^`]+`)", texto):
        if not parte:
            continue
        negrito = parte.startswith("**")
        codigo = parte.startswith("`")
        if negrito or codigo:
            parte = parte.strip("*`")
        rpr = (MONO if codigo else FONTE) + ("<w:b/><w:bCs/>" if negrito else "")
        if cor:
            rpr += f'<w:color w:val="{cor}"/>'
        tam = tamanho - 2 if codigo else tamanho
        rpr += f'<w:sz w:val="{tam}"/><w:szCs w:val="{tam}"/>{LANG}'
        saida.append(f'<w:r><w:rPr>{rpr}</w:rPr><w:t xml:space="preserve">{esc(parte)}</w:t></w:r>')
    return "".join(saida)


def subtitulo(texto: str) -> str:
    # mesmo formato dos subtítulos 7.x do documento
    rpr = (FONTE + '<w:b/><w:bCs/><w:color w:val="92D050"/><w:kern w:val="0"/>'
           '<w:sz w:val="24"/><w:szCs w:val="24"/>' + LANG)
    return (f'<w:p><w:pPr><w:keepNext/><w:spacing w:before="240" w:after="120" w:line="240" w:lineRule="auto"/>'
            f'<w:outlineLvl w:val="3"/><w:rPr>{rpr}</w:rPr></w:pPr>'
            f'<w:r><w:rPr>{rpr}</w:rPr><w:t xml:space="preserve">{esc(texto)}</w:t></w:r></w:p>')


def subtitulo_menor(texto: str) -> str:
    return (f'<w:p><w:pPr><w:keepNext/><w:spacing w:before="200" w:after="80" w:line="240" w:lineRule="auto"/></w:pPr>'
            f'{runs("**" + texto + "**", 22)}</w:p>')


def paragrafo(texto: str) -> str:
    return (f'<w:p><w:pPr><w:spacing w:after="120" w:line="276" w:lineRule="auto"/>'
            f'<w:ind w:firstLine="720"/><w:jc w:val="both"/></w:pPr>{runs(texto)}</w:p>')


def marcador(texto: str) -> str:
    return (f'<w:p><w:pPr><w:numPr><w:ilvl w:val="0"/><w:numId w:val="2"/></w:numPr>'
            f'<w:spacing w:after="60" w:line="276" w:lineRule="auto"/><w:ind w:left="720" w:hanging="360"/>'
            f'<w:jc w:val="both"/></w:pPr>{runs(texto)}</w:p>')


def codigo(linhas: list[str]) -> str:
    rpr = f'{MONO}<w:sz w:val="17"/><w:szCs w:val="17"/><w:lang w:val="pt-BR"/>'
    corpo = '<w:r><w:rPr>' + rpr + '</w:rPr><w:br/></w:r>'
    partes = [f'<w:r><w:rPr>{rpr}</w:rPr><w:t xml:space="preserve">{esc(l)}</w:t></w:r>' for l in linhas]
    borda = '<w:{0} w:val="single" w:sz="4" w:space="4" w:color="BFBFBF"/>'
    bordas = "".join(borda.format(lado) for lado in ("top", "left", "bottom", "right"))
    return (f'<w:p><w:pPr><w:pBdr>{bordas}</w:pBdr><w:shd w:val="clear" w:color="auto" w:fill="F5F5F5"/>'
            f'<w:spacing w:before="120" w:after="200" w:line="240" w:lineRule="auto"/>'
            f'<w:ind w:left="113" w:right="113"/></w:pPr>{corpo.join(partes)}</w:p>')


def tabela(linhas: list[list[str]]) -> str:
    ncol = len(linhas[0])
    pesos = [max(len(re.sub(r"[*`]", "", l[i])) for l in linhas) for i in range(ncol)]
    pesos = [max(p, 8) for p in pesos]
    larguras = [round(LARGURA_UTIL * p / sum(pesos)) for p in pesos]
    larguras[-1] = LARGURA_UTIL - sum(larguras[:-1])
    borda = '<w:{0} w:val="single" w:sz="4" w:space="0" w:color="A6A6A6"/>'
    bordas = "".join(borda.format(l) for l in ("top", "left", "bottom", "right", "insideH", "insideV"))
    xml = [f'<w:tbl><w:tblPr><w:tblW w:w="{LARGURA_UTIL}" w:type="dxa"/><w:jc w:val="center"/>'
           f'<w:tblBorders>{bordas}</w:tblBorders><w:tblLayout w:type="fixed"/>'
           f'<w:tblCellMar><w:top w:w="40" w:type="dxa"/><w:left w:w="80" w:type="dxa"/>'
           f'<w:bottom w:w="40" w:type="dxa"/><w:right w:w="80" w:type="dxa"/></w:tblCellMar></w:tblPr>'
           f'<w:tblGrid>{"".join(f"<w:gridCol w:w={chr(34)}{w}{chr(34)}/>" for w in larguras)}</w:tblGrid>']
    for i, linha in enumerate(linhas):
        cab = i == 0
        xml.append("<w:tr>" + ("<w:trPr><w:tblHeader/></w:trPr>" if cab else ""))
        for celula, w in zip(linha, larguras):
            shd = '<w:shd w:val="clear" w:color="auto" w:fill="E2F0D9"/>' if cab else ""
            texto = f"**{celula}**" if cab and celula else celula
            xml.append(f'<w:tc><w:tcPr><w:tcW w:w="{w}" w:type="dxa"/>{shd}<w:vAlign w:val="center"/></w:tcPr>'
                       f'<w:p><w:pPr><w:spacing w:after="0" w:line="240" w:lineRule="auto"/></w:pPr>'
                       f'{runs(texto, 20)}</w:p></w:tc>')
        xml.append("</w:tr>")
    xml.append("</w:tbl>")
    # parágrafo vazio depois da tabela para separar do próximo bloco
    return "".join(xml) + '<w:p><w:pPr><w:spacing w:after="0"/></w:pPr></w:p>'


MAX_W, MAX_H = 6.5 * 914400, 8.1 * 914400


def preparar_imagem(origem: pathlib.Path, destino: pathlib.Path) -> None:
    """Copia a imagem; se ela ficar maior girada (diagrama muito largo), grava girada 90°."""
    img = Image.open(origem)
    w, h = img.size
    normal = min(MAX_W / w, MAX_H / h)
    girada = min(MAX_W / h, MAX_H / w)
    if w > h and girada > normal * 1.15:
        img.rotate(90, expand=True).save(destino)
    else:
        shutil.copy(origem, destino)


def figura(rid: str, num: int, png: pathlib.Path, legenda: str) -> str:
    w_px, h_px = Image.open(png).size
    escala = min(MAX_W / w_px, MAX_H / h_px)
    cx, cy = round(w_px * escala), round(h_px * escala)
    imagem = (
        f'<w:r><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0">'
        f'<wp:extent cx="{cx}" cy="{cy}"/><wp:docPr id="{100 + num}" name="Figura {num}"/>'
        f'<wp:cNvGraphicFramePr><a:graphicFrameLocks xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" noChangeAspect="1"/></wp:cNvGraphicFramePr>'
        f'<a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">'
        f'<a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture">'
        f'<pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">'
        f'<pic:nvPicPr><pic:cNvPr id="{100 + num}" name="{png.name}"/><pic:cNvPicPr/></pic:nvPicPr>'
        f'<pic:blipFill><a:blip r:embed="{rid}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>'
        f'<pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="{cx}" cy="{cy}"/></a:xfrm>'
        f'<a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr></pic:pic>'
        f'</a:graphicData></a:graphic></wp:inline></w:drawing></w:r>')
    return (f'<w:p><w:pPr><w:keepNext/><w:spacing w:before="200" w:after="60"/><w:jc w:val="center"/></w:pPr>{imagem}</w:p>'
            f'<w:p><w:pPr><w:spacing w:after="240"/><w:jc w:val="center"/></w:pPr>'
            f'{runs(f"**Figura {num}** – {legenda}", 20)}</w:p>')


def converter_markdown(md: str, pasta_md: pathlib.Path, nova_figura) -> str:
    """Converte um arquivo de seção (subconjunto de Markdown) em XML do Word."""
    blocos = []
    linhas = md.splitlines()
    i = 0
    while i < len(linhas):
        linha = linhas[i]
        imagem = re.fullmatch(r"!\[(.+)\]\((.+\.png)\)", linha.strip())
        if linha.startswith("# "):            # o título da seção já existe no documento
            i += 1
        elif imagem:
            blocos.append(nova_figura((pasta_md / imagem.group(2)).resolve(), imagem.group(1)))
            i += 1
        elif linha.startswith("## "):
            blocos.append(subtitulo(linha[3:].strip()))
            i += 1
        elif linha.startswith("### "):
            blocos.append(subtitulo_menor(linha[4:].strip()))
            i += 1
        elif linha.startswith("```"):
            j = i + 1
            while not linhas[j].startswith("```"):
                j += 1
            blocos.append(codigo(linhas[i + 1:j]))
            i = j + 1
        elif linha.startswith("|"):
            tab = []
            while i < len(linhas) and linhas[i].startswith("|"):
                celulas = [c.strip() for c in linhas[i].strip().strip("|").split("|")]
                if not all(re.fullmatch(r":?-+:?", c) for c in celulas):
                    tab.append(celulas)
                i += 1
            blocos.append(tabela(tab))
        elif linha.startswith("- "):
            texto = [linha[2:].strip()]
            i += 1
            # linhas de continuação do mesmo item (indentadas)
            while i < len(linhas) and linhas[i].startswith("  ") and linhas[i].strip():
                texto.append(linhas[i].strip())
                i += 1
            blocos.append(marcador(" ".join(texto)))
        elif linha.strip() in ("", "---"):
            i += 1
        else:
            texto = [linha.strip()]
            i += 1
            while i < len(linhas) and linhas[i].strip() and not re.match(r"(#|```|\||- |---|!\[)", linhas[i]):
                texto.append(linhas[i].strip())
                i += 1
            blocos.append(paragrafo(" ".join(texto)))
    return "".join(blocos)


def main():
    with tempfile.TemporaryDirectory() as tmp:
        pasta = pathlib.Path(tmp)
        with zipfile.ZipFile(ORIGEM) as z:
            z.extractall(pasta)
        doc_path = pasta / "word" / "document.xml"
        doc = doc_path.read_text(encoding="utf-8")

        # ---- 1. seções 1 a 7: tabela 7.2 e regras que mudaram ----
        for antigo, novo in TABELA_72 + AJUSTES_TEXTO:
            assert doc.count(antigo) == 1, f"não encontrei (ou achei repetido): {antigo}"
            doc = doc.replace(antigo, novo)

        # ---- 2. figuras: arquivo, relacionamento e numeração única ----
        (pasta / "word" / "media").mkdir(exist_ok=True)
        rels_path = pasta / "word" / "_rels" / "document.xml.rels"
        rels = [rels_path.read_text(encoding="utf-8")]
        contador = [0]

        def nova_figura(png: pathlib.Path, legenda: str) -> str:
            contador[0] += 1
            num = contador[0]
            nome = f"figura{num:02d}.png"
            preparar_imagem(png, pasta / "word" / "media" / nome)
            rid = f"rIdFigura{num}"
            rels[0] = rels[0].replace("</Relationships>",
                                      f'<Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/{nome}"/></Relationships>')
            return figura(rid, num, pasta / "word" / "media" / nome, legenda)

        # ---- 3. seções 8 a 20: troca o texto-modelo pelo conteúdo ----
        texto_de = lambda m: re.sub(r"<[^>]+>", "", m.group(0))
        # primeiro marca o lugar de cada seção; depois converte em ordem (figuras numeradas de 1 em diante)
        for n in range(20, 7, -1):
            paragrafos = list(re.finditer(r"<w:p[ >].*?</w:p>", doc, flags=re.S))
            ini = next(m for m in paragrafos if texto_de(m) == TITULOS[n])
            if n < 20:
                fim = next(m for m in paragrafos if texto_de(m) == TITULOS[n + 1]).start()
            else:
                fim = doc.rindex("<w:sectPr")
            doc = doc[:ini.end()] + f"<!--secao{n}-->" + doc[fim:]
        for n in range(8, 21):
            arquivo = next(SECOES.glob(f"{n:02d}-*.md"))
            novo = converter_markdown(arquivo.read_text(encoding="utf-8"), arquivo.parent, nova_figura)
            doc = doc.replace(f"<!--secao{n}-->", novo)

        rels_path.write_text(rels[0], encoding="utf-8")
        ct_path = pasta / "[Content_Types].xml"
        ct = ct_path.read_text(encoding="utf-8")
        if 'Extension="png"' not in ct:
            ct = ct.replace("<Default Extension=\"xml\"", '<Default Extension="png" ContentType="image/png"/><Default Extension="xml"')
        ct_path.write_text(ct, encoding="utf-8")

        # garante os prefixos de namespace usados nas figuras
        raiz = re.match(r"<\?xml[^>]*>\s*<w:document[^>]*>", doc).group(0)
        for prefixo in ("wp", "r"):
            assert f"xmlns:{prefixo}=" in raiz, f"namespace {prefixo} ausente"
        doc_path.write_text(doc, encoding="utf-8")

        DESTINO.parent.mkdir(exist_ok=True)
        if DESTINO.exists():
            DESTINO.unlink()
        with zipfile.ZipFile(DESTINO, "w", zipfile.ZIP_DEFLATED) as z:
            for arq in sorted(pasta.rglob("*")):
                if arq.is_file():
                    z.write(arq, arq.relative_to(pasta).as_posix())
    print(f"gerado: {DESTINO.relative_to(RAIZ)}  ({contador[0]} figuras)")


if __name__ == "__main__":
    main()
