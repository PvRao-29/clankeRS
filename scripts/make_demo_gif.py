#!/usr/bin/env python3
"""Render the README demo GIF for `cargo run --release --example camera_replay`.

The real run completes in ~260 ms — too fast to record live — so this renders a
paced, terminal-styled animation of the *real* captured output (the numbers here
are from an actual run on this machine). Reproducible:

    pip install pillow
    python3 scripts/make_demo_gif.py

Output: docs/assets/camera_replay.gif
"""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

REPO = Path(__file__).resolve().parent.parent
OUT = REPO / "docs" / "assets" / "camera_replay.gif"

# ---- palette (GitHub-dark inspired) -----------------------------------------
OUTER = (1, 4, 9)
BG = (13, 17, 23)
BAR = (22, 27, 34)
FG = (201, 209, 217)
DIM = (110, 118, 129)
GREEN = (63, 185, 80)
BGREEN = (86, 211, 100)
BLUE = (88, 166, 255)
CYAN = (121, 192, 255)
AMBER = (224, 176, 92)
TITLE = (139, 148, 158)
LIGHTS = [(255, 95, 86), (255, 189, 46), (39, 201, 63)]

# ---- layout -----------------------------------------------------------------
FS = 21
LH = 30
PAD_X = 30
PAD_TOP = 18
BAR_H = 46
MARGIN = 18
RADIUS = 12


def load_font(names, size):
    for n in names:
        for base in ("/Library/Fonts/IBM-Plex-Mono/", "/System/Library/Fonts/", ""):
            try:
                return ImageFont.truetype(base + n, size)
            except OSError:
                continue
    return ImageFont.load_default()


REG = load_font(["IBMPlexMono-Regular.otf", "Menlo.ttc", "Monaco.ttf"], FS)
BOLD = load_font(["IBMPlexMono-Bold.otf", "Menlo.ttc", "Monaco.ttf"], FS)
TFONT = load_font(["IBMPlexMono-Medium.otf", "Menlo.ttc"], 15)

CHARW = REG.getlength("M")

# A "line" is a list of (text, color, bold) spans.
CHECK = "\x00CHECK\x00"  # sentinel: draw a crisp vector checkmark, not a glyph


def span(text, color=FG, bold=False):
    return (text, color, bold)


def build_final():
    """The complete terminal screen, as an ordered list of lines."""
    return [
        [span("$ ", GREEN, True), span("cargo run --release --example camera_replay")],
        [],
        [span("Loading "), span("detector.onnx", CYAN), span("...")],
        [span("Opening "), span("camera_log.mcap", CYAN), span("...")],
        [span("Running replay...")],
        [],
        [span("Frame ", DIM), span("200", AMBER), span("/", DIM), span("200", AMBER)],
        [],
        [span("Published "), span("200", AMBER), span(" detections to "), span("/detections", CYAN)],
        [span("Replay complete.")],
        [],
        [span("Replay Summary", BLUE, True)],
        [span("  Frames:    ", DIM), span("200", AMBER)],
        [span("  FPS:       ", DIM), span("781.2", AMBER)],
        [span("  Detections received on ", DIM), span("/detections", CYAN), span(": ", DIM), span("200", AMBER)],
        [span("  Dropped:   ", DIM), span("0", AMBER)],
        [],
        [span("Latency:", BLUE, True)],
        [span("  p50: ", DIM), span("1.2 ms", GREEN)],
        [span("  p95: ", DIM), span("1.6 ms", GREEN)],
        [span("  p99: ", DIM), span("1.7 ms", GREEN)],
        [],
        [span(CHECK, BGREEN, True), span(" Replay passed", BGREEN, True)],
    ]


FINAL = build_final()
NCOLS = max((sum(len(t) for t, _, _ in ln) for ln in FINAL), default=40)
NROWS = len(FINAL)

W = int(PAD_X * 2 + (NCOLS + 4) * CHARW)
INNER_H = int(PAD_TOP * 2 + NROWS * LH)
H = BAR_H + INNER_H
CW = W + MARGIN * 2
CH = H + MARGIN * 2


def render(lines, cursor=False):
    """Render a full frame given the visible lines (with an optional cursor)."""
    img = Image.new("RGB", (CW, CH), OUTER)
    d = ImageDraw.Draw(img)
    # window
    d.rounded_rectangle([MARGIN, MARGIN, MARGIN + W, MARGIN + H], RADIUS, fill=BG)
    d.rounded_rectangle([MARGIN, MARGIN, MARGIN + W, MARGIN + BAR_H], RADIUS, fill=BAR)
    d.rectangle([MARGIN, MARGIN + BAR_H - RADIUS, MARGIN + W, MARGIN + BAR_H], fill=BAR)
    # traffic lights
    cy = MARGIN + BAR_H // 2
    for i, col in enumerate(LIGHTS):
        cx = MARGIN + 22 + i * 22
        d.ellipse([cx - 7, cy - 7, cx + 7, cy + 7], fill=col)
    # title
    title = "camera_replay — clankeRS"
    tw = TFONT.getlength(title)
    d.text((MARGIN + (W - tw) / 2, cy - 9), title, font=TFONT, fill=TITLE)

    ox = MARGIN + PAD_X
    oy = MARGIN + BAR_H + PAD_TOP
    for r, line in enumerate(lines):
        x = ox
        y = oy + r * LH
        for text, color, bold in line:
            if text == CHECK:
                d.line(
                    [
                        (x + 2, y + FS * 0.55),
                        (x + CHARW * 0.38, y + FS * 0.82),
                        (x + CHARW * 0.95, y + FS * 0.22),
                    ],
                    fill=color,
                    width=3,
                    joint="curve",
                )
                x += CHARW
                continue
            d.text((x, y), text, font=(BOLD if bold else REG), fill=color)
            x += CHARW * len(text)
        if cursor and r == len(lines) - 1:
            d.rectangle([x + 2, y + 3, x + 2 + CHARW * 0.9, y + FS + 2], fill=FG)
    return img


def main():
    OUT.parent.mkdir(parents=True, exist_ok=True)
    frames, durations = [], []

    def add(lines, dur, cursor=False):
        frames.append(render(lines, cursor))
        durations.append(dur)

    # Phase A: type the command.
    cmd = "cargo run --release --example camera_replay"
    add([[span("$ ", GREEN, True)]], 500, cursor=True)
    step = 3
    for k in range(step, len(cmd) + step, step):
        typed = cmd[:k]
        add([[span("$ ", GREEN, True), span(typed)]], 45, cursor=True)
    add([FINAL[0]], 350, cursor=True)

    # Phase B: reveal the header lines one at a time.
    shown = [FINAL[0]]
    for idx in range(1, 6):  # blank, Loading, Opening, Running, blank
        shown = FINAL[:idx + 1]
        add(shown, 320 if idx in (2, 3, 4) else 120)

    # Phase C: animate the frame counter in place.
    frame_line_idx = 6
    base = FINAL[:frame_line_idx]
    for val in (1, 34, 78, 121, 165, 200):
        counter = [
            [span("Frame ", DIM), span(str(val), AMBER), span("/", DIM), span("200", AMBER)]
        ]
        add(base + counter, 70 if val != 200 else 260)

    # Phase D: reveal the rest, line by line.
    for idx in range(frame_line_idx + 1, len(FINAL) - 1):
        add(FINAL[:idx + 1], 90)

    # Phase E: the payoff line.
    add(FINAL, 200)
    add(FINAL, 2600)  # hold so it is readable before the loop

    frames[0].save(
        OUT,
        save_all=True,
        append_images=frames[1:],
        duration=durations,
        loop=0,
        optimize=True,
        disposal=2,
    )
    size_kb = OUT.stat().st_size / 1024
    print(f"wrote {OUT.relative_to(REPO)}  ({W}x{H}, {len(frames)} frames, {size_kb:.0f} KB)")


if __name__ == "__main__":
    main()
