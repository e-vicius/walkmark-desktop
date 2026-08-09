#!/usr/bin/env python3
"""Generate PNG assets for static HTML screenshot mocks."""

from __future__ import annotations

from pathlib import Path

from PIL import Image

from seed_demo import GUIDES, PALETTE, synthetic_frame

ROOT = Path(__file__).resolve().parent
FRAMES = ROOT / "static" / "images" / "frames"
COVERS = ROOT / "static" / "images" / "covers"

# Filenames referenced by scripts/screenshots/*.html
INVOICE_FRAMES = [
    ("000003200-462a84.png", 0, "Open the billing settings"),
    ("000010000-ab454c.png", 1, "Start a new invoice"),
    ("000016800-393e9c.png", 2, "Pick the customer"),
    ("000023600-4526e0.png", 3, "Add the line items"),
    ("000030400-80e335.png", 4, "Send the invoice"),
]

COVER_IDS = ["mock-invoice", "mock-slack", "mock-password", "mock-product", "mock-report"]
COVER_FILES = ["invoice.png", "slack.png", "password.png", "product.png", "report.png"]


def save_frame(path: Path, theme: int, step_index: int, title: str, app_name: str) -> None:
    img = synthetic_frame(theme, step_index, title, app_name)
    # Smaller files; mocks only need preview resolution.
    width = 1200
    height = round(width * 9 / 16)
    img = img.resize((width, height), Image.Resampling.LANCZOS)
    img.save(path, optimize=True)


def main() -> None:
    FRAMES.mkdir(parents=True, exist_ok=True)
    COVERS.mkdir(parents=True, exist_ok=True)

    invoice = GUIDES[0]
    theme = invoice["theme"]
    app_name = PALETTE[theme][2]

    for filename, step_index, title in INVOICE_FRAMES:
        save_frame(FRAMES / filename, theme, step_index, title, app_name)

    slack = GUIDES[1]
    slack_theme = slack["theme"]
    slack_app = PALETTE[slack_theme][2]
    save_frame(FRAMES / "slack-step-01.png", slack_theme, 0, slack["steps"][0][0], slack_app)

    for guide_id, cover_file in zip(COVER_IDS, COVER_FILES):
        guide = next(g for g in GUIDES if g["id"] == guide_id)
        theme = guide["theme"]
        app_name = PALETTE[theme][2]
        title = guide["steps"][0][0]
        img = synthetic_frame(theme, 0, title, app_name)
        img = img.resize((640, round(640 * 9 / 16)), Image.Resampling.LANCZOS)
        img.save(COVERS / cover_file, optimize=True)

    print(f"Wrote screenshot assets under {ROOT / 'static' / 'images'}")


if __name__ == "__main__":
    main()
