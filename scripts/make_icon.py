#!/usr/bin/env python3
"""Render the Walkmark app icon.

Kept as a script rather than a checked-in binary so the mark can be adjusted
without a design tool. Produces a 1024x1024 PNG; run `pnpm tauri icon` on the
result to generate every platform size.

    python3 scripts/make_icon.py src-tauri/icons/source.png
"""

from __future__ import annotations

import sys
from pathlib import Path

from PIL import Image, ImageDraw

SIZE = 1024
# Supersample, then downscale, so the curves come out clean without needing
# antialiased primitives.
SS = 4
CANVAS = SIZE * SS

BG = (17, 17, 17)
STEP_FILL = (255, 255, 255)


def rounded_mask(size: int, radius: int) -> Image.Image:
    mask = Image.new("L", (size, size), 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, size - 1, size - 1), radius, fill=255)
    return mask


def main(destination: Path) -> None:
    # macOS "squircle" proportion: roughly 22% of the edge.
    radius = round(CANVAS * 0.225)

    icon = Image.new("RGBA", (CANVAS, CANVAS), (0, 0, 0, 0))
    background = Image.new("RGBA", (CANVAS, CANVAS), (*BG, 255))
    icon.paste(background, (0, 0), rounded_mask(CANVAS, radius))

    draw = ImageDraw.Draw(icon)

    # A staircase climbing to the right: three treads, each offset up and over.
    unit = CANVAS / 100
    tread_radius = round(unit * 2.4)
    tread_width = unit * 30
    tread_height = unit * 10.5
    rise = unit * 17
    run = unit * 17

    origin_x = (CANVAS - (2 * run + tread_width)) / 2
    origin_y = (CANVAS - (2 * rise + tread_height)) / 2 + 2 * rise

    for index in range(3):
        x0 = origin_x + index * run
        y0 = origin_y - index * rise
        draw.rounded_rectangle(
            (x0, y0, x0 + tread_width, y0 + tread_height),
            tread_radius,
            fill=STEP_FILL if index == 2 else (*STEP_FILL, 190 + index * 25),
        )

    icon = icon.resize((SIZE, SIZE), Image.Resampling.LANCZOS)
    destination.parent.mkdir(parents=True, exist_ok=True)
    icon.save(destination)
    print(f"wrote {destination} ({SIZE}x{SIZE})")


if __name__ == "__main__":
    main(Path(sys.argv[1] if len(sys.argv) > 1 else "src-tauri/icons/source.png"))
