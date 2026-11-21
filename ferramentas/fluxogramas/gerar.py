"""Gera os PNGs dos fluxogramas a partir dos arquivos .dot da pasta fonte/.

Lê documentos/relatorio/fluxogramas/fonte/*.dot e grava os PNGs em documentos/relatorio/fluxogramas/.

Uso (a partir da raiz A3):
    npm install --prefix ferramentas/fluxogramas   # uma vez só (Graphviz em WebAssembly)
    python3 ferramentas/fluxogramas/gerar.py

Requer Node.js e Google Chrome.

Nos .dot, cada nó usa class=<tipo> para receber o símbolo padrão de fluxograma:
    term = início/fim   io = entrada/saída   proc = processamento
    dec  = decisão      sub = sub-algoritmo  con = conector
"""
import pathlib
import re
import subprocess
import tempfile

PASTA = pathlib.Path(__file__).resolve().parent          # ferramentas/fluxogramas (scripts)
SAIDA = PASTA.parent.parent / "documentos" / "relatorio" / "fluxogramas"
FONTE = SAIDA / "fonte"

CABECALHO = """digraph G {
  graph [rankdir=TB nodesep=0.3 ranksep=0.24 pad=0.15 bgcolor=white fontname="DejaVu Sans"]
  node  [fontname="DejaVu Sans" fontsize=11 penwidth=1.5 margin="0.12,0.05"]
  edge  [fontname="DejaVu Sans" fontsize=10 arrowsize=0.7 color="#222222" fontcolor="#7a1240"]
"""

CLASSES = {
    "term": 'shape=box style="rounded,filled" fillcolor="#d9f2d9" color="#2e8b3a"',
    "io":   'shape=polygon sides=4 skew=0.25 style=filled fillcolor="#dce9fb" color="#2f5fb3"',
    "proc": 'shape=box style=filled fillcolor="#fdebc8" color="#e08a1e"',
    "dec":  'shape=diamond style=filled fillcolor="#fbdbe6" color="#c2185b" margin="0.03,0.02"',
    "sub":  'shape=box style=filled peripheries=2 fillcolor="#ece3f7" color="#6a3fb5"',
    "con":  'shape=circle style=filled fillcolor="#cfe0fa" color="#2f5fb3" width=0.32 fixedsize=true',
}

HTML = """<!doctype html><html><head><meta charset="utf-8">
<style>html,body{{margin:0;background:#fff}}img{{display:block}}</style></head>
<body><img src="{svg}" width="{w}" height="{h}"></body></html>"""


def montar_dot(corpo: str) -> str:
    corpo = re.sub(r"class=(\w+)", lambda m: CLASSES[m.group(1)], corpo)
    return CABECALHO + corpo + "\n}\n"


for arq in sorted(FONTE.glob("*.dot")):
    png = SAIDA / (arq.stem + ".png")
    with tempfile.TemporaryDirectory() as tmp:
        tmp = pathlib.Path(tmp)
        dot, svg, html, bruto = tmp / "f.dot", tmp / "f.svg", tmp / "f.html", tmp / "bruto.png"
        dot.write_text(montar_dot(arq.read_text(encoding="utf-8")), encoding="utf-8")
        subprocess.run(["node", str(PASTA / "render.mjs"), str(dot), str(svg)], check=True)
        texto = svg.read_text(encoding="utf-8")
        w_pt, h_pt = (float(v) for v in re.search(r'width="([\d.]+)pt"\s+height="([\d.]+)pt"', texto).groups())
        w, h = round(w_pt * 4 / 3), round(h_pt * 4 / 3)
        html.write_text(HTML.format(svg=svg.as_uri(), w=w, h=h), encoding="utf-8")
        subprocess.run([
            "google-chrome", "--headless=new", "--disable-gpu", "--hide-scrollbars",
            "--allow-file-access-from-files", "--force-device-scale-factor=3",
            f"--window-size={w},{h}", f"--screenshot={bruto}", html.as_uri(),
        ], check=True, capture_output=True)
        subprocess.run(["convert", str(bruto), "-trim", "-bordercolor", "white",
                        "-border", "24", str(png)], check=True)
    print(f"gerado: {png.name}  ({w / 96:.1f} x {h / 96:.1f} pol. em tamanho real)")
