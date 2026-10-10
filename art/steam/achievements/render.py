from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parent
SIZE = 256

GLYPHS = {
    "empty-handed": '''<circle cx="128" cy="145" r="47"/><path d="M111 97V72h34v25M120 72V58h16v14M146 60l20-14M55 209 201 63"/>''',
    "trust-your-ears": '''<path d="M105 179c-1 18 8 28 23 28 16 0 25-11 25-27 0-14 8-20 19-31 11-11 17-26 17-42 0-37-25-61-58-61-35 0-60 25-60 59 0 13 4 23 11 31"/><path d="M104 115c-4-20 7-38 26-40 18-2 32 10 32 27 0 14-8 22-19 32-9 8-15 16-15 29"/><path d="M101 144c9 4 18 3 26-4M53 202 203 54"/>''',
    "buy-some-time": '''<path d="M128 53 143 101 184 72 157 113 203 128 157 143 184 184 143 157 128 203 113 157 72 184 99 143 53 128 99 113 72 72 113 101Z"/>''',
    "in-the-dark": '''<path d="M128 69c-26 0-44 18-44 43 0 17 9 26 20 37 6 6 7 11 7 18h34c0-7 1-12 7-18 11-11 20-20 20-37 0-25-18-43-44-43ZM110 179h36M116 191h24M123 201h10M55 203 201 57"/>''',
    "let-there-be-light": '''<path d="M128 78c-26 0-44 18-44 43 0 17 9 26 20 37 6 6 7 11 7 18h34c0-7 1-12 7-18 11-11 20-20 20-37 0-25-18-43-44-43ZM110 188h36M116 200h24M123 210h10M82 57l-9-13M128 55V37M174 57l9-13"/>''',
    "so-close": '''<path d="M104 205V53h90v152M104 205h90M104 53 52 84v90l52 31V53Z"/><circle cx="66" cy="129" r="3"/>''',
    "unseen": '''<path d="M40 128s33-45 88-45 88 45 88 45-33 45-88 45-88-45-88-45Z"/><circle cx="128" cy="128" r="23"/><path d="M52 204 204 52"/>''',
}


def svg(key: str, locked: bool) -> str:
    ink = "#687477" if locked else "#ecdfbf"
    edge = "#3b494e" if locked else "#88b9a3"
    background = "#11191d" if locked else "#0b1115"
    label = "LOCKED" if locked else "THE SPECIMEN"
    return f'''<svg xmlns="http://www.w3.org/2000/svg" width="{SIZE}" height="{SIZE}" viewBox="0 0 256 256">
<rect width="256" height="256" fill="{background}"/>
<rect x="10" y="10" width="236" height="236" rx="12" fill="none" stroke="{edge}" stroke-width="3"/>
<path d="M31 32h46M179 32h46M31 224h46M179 224h46" stroke="{edge}" stroke-width="2"/>
<g fill="none" stroke="{ink}" stroke-width="7" stroke-linecap="round" stroke-linejoin="round">{GLYPHS[key]}</g>
<text x="128" y="231" text-anchor="middle" fill="{edge}" font-family="sans-serif" font-size="11" letter-spacing="2">{label}</text>
</svg>'''


def main() -> None:
    for key in GLYPHS:
        for locked in (False, True):
            name = f"{key}-{'locked' if locked else 'unlocked'}"
            source = ROOT / f"{name}.svg"
            source.write_text(svg(key, locked) + "\n")
            subprocess.run(
                ["inkscape", str(source), "--export-type=png", f"--export-filename={ROOT / (name + '.png')}"],
                check=True,
                capture_output=True,
            )


if __name__ == "__main__":
    main()
