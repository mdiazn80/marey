"""Genera la base de iconos e imágenes de Marey para un proyecto Tauri v2."""
import os
import subprocess
import sys

from fontTools.ttLib import TTFont
from fontTools.varLib import instancer
from fontTools.pens.svgPathPen import SVGPathPen
from fontTools.pens.transformPen import TransformPen
from fontTools.pens.boundsPen import BoundsPen
from PIL import Image

FONT, OUT = sys.argv[1], sys.argv[2]

GROUND = "#0E0D0B"
CHALK = "#EEEAE0"
PAPER = "#F2EFE7"
INK = "#15140F"
ACCENT = "#FF5A36"

# ---------- Marca A · Cronofotografía (coordenadas en caja 512) ----------
ECHOES = [(88, 0.12), (110, 0.24), (132, 0.38), (154, 0.56)]  # x de la pata izquierda
MAIN_X = 176
MARK_BOUNDS = (85.5, 122.0, 430.0, 390.0)  # x0, y0, x1, y1 incluyendo trazo y puntos
MARK_CX = (MARK_BOUNDS[0] + MARK_BOUNDS[2]) / 2
MARK_CY = (MARK_BOUNDS[1] + MARK_BOUNDS[3]) / 2


def m_points(x):
    return f"{x},376 {x},136 {x + 120},280 {x + 240},136 {x + 240},376"


def mark_body(fg, accent=ACCENT):
    echoes = "".join(
        f'<polyline points="{m_points(x)}" opacity="{o}"/>' for x, o in ECHOES
    )
    dots = "".join(
        f'<circle cx="{cx}" cy="{cy}" r="14"/>'
        for cx, cy in [(176, 376), (176, 136), (296, 280), (416, 136), (416, 376)]
    )
    return (
        f'<g fill="none" stroke="{fg}" stroke-linecap="round" stroke-linejoin="round" stroke-width="5">{echoes}</g>'
        f'<polyline points="{m_points(MAIN_X)}" fill="none" stroke="{fg}" stroke-width="20" '
        f'stroke-linecap="round" stroke-linejoin="round"/>'
        f'<g fill="{accent}">{dots}</g>'
    )


def placed_mark(cx, cy, width, fg, accent=ACCENT):
    """Marca centrada en (cx, cy) con un ancho visual dado."""
    s = width / (MARK_BOUNDS[2] - MARK_BOUNDS[0])
    return (
        f'<g transform="translate({cx:.2f} {cy:.2f}) scale({s:.4f}) '
        f'translate({-MARK_CX:.2f} {-MARK_CY:.2f})">{mark_body(fg, accent)}</g>'
    )


def svg(w, h, body):
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" '
        f'viewBox="0 0 {w} {h}">{body}</svg>\n'
    )


# ---------- Wordmark "marey" en Sora SemiBold, convertido a trazados ----------
font = instancer.instantiateVariableFont(TTFont(FONT), {"wght": 600})
UPM = font["head"].unitsPerEm
GLYPHS = font.getGlyphSet()
CMAP = font.getBestCmap()
X_HEIGHT = font["OS/2"].sxHeight


def word_path(text, tracking=-0.035):
    pen = SVGPathPen(GLYPHS)
    bounds = BoundsPen(GLYPHS)
    x = 0.0
    for i, ch in enumerate(text):
        g = CMAP[ord(ch)]
        GLYPHS[g].draw(TransformPen(pen, (1, 0, 0, -1, x, 0)))
        GLYPHS[g].draw(TransformPen(bounds, (1, 0, 0, -1, x, 0)))
        x += font["hmtx"][g][0]
        if i < len(text) - 1:
            x += tracking * UPM
    return pen.getCommands(), bounds.bounds  # bounds en unidades, y hacia abajo


WORD_D, WORD_B = word_path("marey")


def placed_word(x_left, xh_center_y, font_px, fill):
    """Wordmark con su borde izquierdo en x_left y la altura x centrada en xh_center_y."""
    s = font_px / UPM
    baseline = xh_center_y + (X_HEIGHT * s) / 2
    tx = x_left - WORD_B[0] * s
    return f'<path fill="{fill}" transform="translate({tx:.2f} {baseline:.2f}) scale({s:.5f})" d="{WORD_D}"/>'


def word_width(font_px):
    return (WORD_B[2] - WORD_B[0]) * font_px / UPM


def lockup(cx, cy, mark_w, fg):
    """Lockup horizontal: marca + wordmark, centrado en (cx, cy)."""
    font_px = mark_w * 0.70
    gap = mark_w * 0.16
    total = mark_w + gap + word_width(font_px)
    left = cx - total / 2
    return placed_mark(left + mark_w / 2, cy, mark_w, fg) + placed_word(
        left + mark_w + gap, cy, font_px, fg
    )


def lockup_size(mark_w):
    font_px = mark_w * 0.70
    mark_h = mark_w * (MARK_BOUNDS[3] - MARK_BOUNDS[1]) / (MARK_BOUNDS[2] - MARK_BOUNDS[0])
    return mark_w + mark_w * 0.16 + word_width(font_px), mark_h


# ---------- Utilidades de salida ----------
def write(rel, content):
    path = os.path.join(OUT, rel)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)
    return path


def png(svg_path, png_rel, w, h):
    out = os.path.join(OUT, png_rel)
    os.makedirs(os.path.dirname(out), exist_ok=True)
    subprocess.run(["rsvg-convert", "-w", str(w), "-h", str(h), svg_path, "-o", out], check=True)
    return out


def bmp(svg_path, bmp_rel, w, h):
    tmp = png(svg_path, bmp_rel.replace(".bmp", ".tmp.png"), w, h)
    out = os.path.join(OUT, bmp_rel)
    Image.open(tmp).convert("RGB").save(out, "BMP")
    os.remove(tmp)


# ---------- 1. Icono maestro para `tauri icon` ----------
R = 229  # radio ~22 % del lado
icon = svg(1024, 1024,
           f'<rect width="1024" height="1024" rx="{R}" fill="{GROUND}"/>' + placed_mark(512, 512, 640, CHALK))
p = write("app-icon.svg", icon)
png(p, "app-icon.png", 1024, 1024)

# Variante con el margen de la rejilla de macOS (824 px de forma en lienzo de 1024)
icon_mac = svg(1024, 1024,
               f'<rect x="100" y="100" width="824" height="824" rx="185" fill="{GROUND}"/>'
               + placed_mark(512, 512, 515, CHALK))
p = write("app-icon-macos.svg", icon_mac)
png(p, "app-icon-macos.png", 1024, 1024)

# ---------- 2. Icono de bandeja (monocromo) ----------
def tray(fg):
    # Solo la M principal, gruesa, sin ecos: a 16–32 px los ecos desaparecen.
    pts = "10,54 10,10 32,36.4 54,10 54,54"
    return svg(64, 64,
               f'<polyline points="{pts}" fill="none" stroke="{fg}" stroke-width="9" '
               f'stroke-linecap="round" stroke-linejoin="round" transform="translate(0 0)"/>')

for name, fg in [("tray-icon-template", "#000000"), ("tray-icon-light", "#FFFFFF")]:
    p = write(f"tray/{name}.svg", tray(fg))
    png(p, f"tray/{name}.png", 32, 32)
    png(p, f"tray/{name}@2x.png", 64, 64)

# ---------- 3. Marca y lockup (fondo transparente) ----------
mw = 512
mh = mw * (MARK_BOUNDS[3] - MARK_BOUNDS[1]) / (MARK_BOUNDS[2] - MARK_BOUNDS[0])
for suffix, fg in [("on-dark", CHALK), ("on-light", INK)]:
    p = write(f"brand/marey-mark-{suffix}.svg",
              svg(mw, round(mh), placed_mark(mw / 2, mh / 2, mw, fg)))
    png(p, f"brand/marey-mark-{suffix}.png", 1024, round(mh * 2))

    lw, lh = lockup_size(400)
    W, H = round(lw + 40), round(lh + 40)
    p = write(f"brand/marey-lockup-{suffix}.svg", svg(W, H, lockup(W / 2, H / 2, 400, fg)))
    png(p, f"brand/marey-lockup-{suffix}.png", W * 2, H * 2)

# ---------- 4. Imágenes de instalador Windows ----------
# NSIS header 150×57 (fondo blanco, arriba a la derecha del instalador)
p = write("installer/nsis-header.svg", svg(150, 57,
          '<rect width="150" height="57" fill="#FFFFFF"/>' + lockup(75, 28.5, 40, INK)))
bmp(p, "installer/nsis-header.bmp", 150, 57)

# NSIS sidebar 164×314 (páginas de bienvenida y fin)
p = write("installer/nsis-sidebar.svg", svg(164, 314,
          f'<rect width="164" height="314" fill="{GROUND}"/>'
          + placed_mark(82, 128, 112, CHALK)
          + placed_word(82 - word_width(34) / 2, 214, 34, CHALK)))
bmp(p, "installer/nsis-sidebar.bmp", 164, 314)

# WiX banner 493×58 (el texto del instalador va a la izquierda: marca a la derecha)
p = write("installer/wix-banner.svg", svg(493, 58,
          '<rect width="493" height="58" fill="#FFFFFF"/>' + placed_mark(452, 29, 56, INK)))
bmp(p, "installer/wix-banner.bmp", 493, 58)

# WiX dialog 493×312 (panel izquierdo de 164 px; el resto lo cubre el texto)
p = write("installer/wix-dialog.svg", svg(493, 312,
          '<rect width="493" height="312" fill="#FFFFFF"/>'
          f'<rect width="164" height="312" fill="{GROUND}"/>'
          + placed_mark(82, 126, 112, CHALK)
          + placed_word(82 - word_width(34) / 2, 212, 34, CHALK)))
bmp(p, "installer/wix-dialog.bmp", 493, 312)

# ---------- 5. Fondo del DMG de macOS 660×400 ----------
# Posiciones por defecto de Tauri: app en (180,170), Aplicaciones en (480,170)
dmg = svg(660, 400,
          f'<rect width="660" height="400" fill="{PAPER}"/>'
          f'<g fill="none" stroke="{INK}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" opacity="0.35">'
          '<line x1="270" y1="170" x2="388" y2="170"/><polyline points="374,156 390,170 374,184"/></g>'
          + lockup(330, 340, 34, INK))
p = write("installer/dmg-background.svg", dmg)
png(p, "installer/dmg-background.png", 660, 400)
png(p, "installer/dmg-background@2x.png", 1320, 800)

# ---------- 6. Imagen social / README (GitHub 1280×640) ----------
p = write("social/og-image.svg", svg(1280, 640,
          f'<rect width="1280" height="640" fill="{GROUND}"/>' + lockup(640, 320, 300, CHALK)))
png(p, "social/og-image.png", 1280, 640)

print("ok")
