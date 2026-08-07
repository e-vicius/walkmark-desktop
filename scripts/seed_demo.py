#!/usr/bin/env python3
"""Seed a demo project so the library and editor can be exercised without
recording and calling Gemini first.

Development aid only — nothing in the app depends on it.

    python3 scripts/seed_demo.py           # create/replace the demo project
    python3 scripts/seed_demo.py --remove  # delete it again
"""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
import uuid
from datetime import datetime, timedelta, timezone
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

IDENTIFIER = "app.steppy.desktop"
PROJECT_ID = "demo-guide"
FRAME_WIDTH = 1800


def data_dir() -> Path:
    if sys.platform == "darwin":
        return Path.home() / "Library" / "Application Support" / IDENTIFIER
    if sys.platform == "win32":
        import os

        return Path(os.environ["APPDATA"]) / IDENTIFIER
    return Path.home() / ".local" / "share" / IDENTIFIER


STEPS = [
    (
        "Open the billing settings",
        "From the sidebar, select **Settings**, then choose **Billing**. The billing "
        "overview lists your current plan along with the next renewal date.",
        "ready",
    ),
    (
        "Start a new invoice",
        "Click **New invoice** in the top right. A draft is created immediately, so you "
        "can leave and come back without losing your work.",
        "ready",
    ),
    (
        "Pick the customer",
        "Type the customer's name into the **Bill to** field and select them from the "
        "suggestions. Their saved address and tax ID fill in automatically.",
        "ready",
    ),
    (
        "Add the line items",
        "For each item, enter a description, quantity and unit price. The subtotal, tax "
        "and total update as you type.",
        "ready",
    ),
    (
        "Set the payment terms",
        "Choose a due date from the **Terms** dropdown. *Net 30* is the default; pick "
        "**On receipt** if the customer should pay straight away.",
        "draft",
    ),
    (
        "Send the invoice",
        "Review the preview on the right, then click **Send**. The customer receives an "
        "email with a payment link, and the invoice moves from *Draft* to *Open*.",
        "ready",
    ),
]

PALETTE = [
    ((248, 250, 252), (99, 102, 241)),
    ((255, 251, 245), (234, 88, 12)),
    ((245, 253, 250), (13, 148, 136)),
    ((250, 245, 255), (147, 51, 234)),
    ((247, 250, 255), (37, 99, 235)),
    ((254, 247, 250), (219, 39, 119)),
]


def font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont:
    candidates = [
        "/System/Library/Fonts/SFNSDisplay.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "/Library/Fonts/Arial.ttf",
    ]
    for path in candidates:
        if Path(path).exists():
            try:
                return ImageFont.truetype(path, size)
            except OSError:
                continue
    return ImageFont.load_default(size)


def synthetic_frame(index: int, title: str) -> Image.Image:
    """A plausible app window, so the editor is judged on realistic content."""
    bg, accent = PALETTE[index % len(PALETTE)]
    width, height = FRAME_WIDTH, round(FRAME_WIDTH * 9 / 16)
    img = Image.new("RGB", (width, height), bg)
    d = ImageDraw.Draw(img)

    chrome = 56
    d.rectangle((0, 0, width, chrome), fill=(255, 255, 255))
    d.line((0, chrome, width, chrome), fill=(226, 232, 240), width=2)
    for i, dot in enumerate([(255, 95, 87), (255, 189, 46), (39, 201, 63)]):
        cx = 28 + i * 26
        d.ellipse((cx - 7, chrome // 2 - 7, cx + 7, chrome // 2 + 7), fill=dot)

    sidebar = 300
    d.rectangle((0, chrome, sidebar, height), fill=(255, 255, 255))
    d.line((sidebar, chrome, sidebar, height), fill=(226, 232, 240), width=2)
    for row in range(7):
        y = chrome + 46 + row * 52
        selected = row == index % 7
        if selected:
            d.rounded_rectangle((16, y - 12, sidebar - 16, y + 26), 10, fill=(*accent, ))
        d.rounded_rectangle(
            (34, y + 1, 34 + (120 + (row * 37) % 90), y + 13),
            6,
            fill=(255, 255, 255) if selected else (203, 213, 225),
        )

    d.text((sidebar + 48, chrome + 48), title, font=font(40), fill=(15, 23, 42))

    card_top = chrome + 130
    d.rounded_rectangle(
        (sidebar + 48, card_top, width - 48, card_top + 260), 18, fill=(255, 255, 255)
    )
    for row in range(4):
        y = card_top + 44 + row * 52
        d.rounded_rectangle(
            (sidebar + 88, y, sidebar + 88 + (420 + (row * 91) % 320), y + 16),
            8,
            fill=(226, 232, 240),
        )
        d.rounded_rectangle((width - 260, y, width - 120, y + 16), 8, fill=(241, 245, 249))

    button_left = width - 300
    d.rounded_rectangle((button_left, chrome + 44, width - 48, chrome + 96), 12, fill=accent)

    d.rounded_rectangle(
        (sidebar + 48, card_top + 300, width - 48, height - 48), 18, fill=(255, 255, 255)
    )
    return img


def capture_frame() -> Image.Image | None:
    """Prefer a real screenshot for the first frame; falls back to synthetic."""
    tmp = Path("/tmp/steppy-seed-capture.png")
    try:
        subprocess.run(["screencapture", "-x", "-o", str(tmp)], check=True, timeout=15)
    except Exception:
        return None
    if not tmp.exists():
        return None
    img = Image.open(tmp).convert("RGB")
    tmp.unlink(missing_ok=True)
    return img


def resize(img: Image.Image) -> Image.Image:
    if img.width <= FRAME_WIDTH:
        return img
    height = round(img.height * FRAME_WIDTH / img.width)
    return img.resize((FRAME_WIDTH, height), Image.Resampling.LANCZOS)


def main() -> None:
    root = data_dir() / "projects" / PROJECT_ID
    if "--remove" in sys.argv:
        shutil.rmtree(root, ignore_errors=True)
        print(f"removed {root}")
        return

    frames = root / "frames"
    shutil.rmtree(root, ignore_errors=True)
    frames.mkdir(parents=True)

    real = capture_frame()
    created = datetime.now(timezone.utc) - timedelta(hours=3)

    steps = []
    for index, (title, body, status) in enumerate(STEPS):
        offset_ms = 4200 + index * 7300
        name = f"{offset_ms:08}.png"
        image = resize(real) if (real and index == 0) else synthetic_frame(index, title)
        image.save(frames / name)

        # A couple of neighbouring frames so the "different moment" picker has
        # something to offer.
        alternates = []
        for delta in (-900, 900):
            alt_name = f"{offset_ms + delta:08}.png"
            synthetic_frame(index + 1, title).save(frames / alt_name)
            alternates.append(alt_name)

        steps.append(
            {
                "id": uuid.uuid4().hex[:12],
                "title": title if status == "ready" else "",
                "body": body if status == "ready" else "",
                "offsetMs": offset_ms,
                "frame": name,
                "alternates": alternates,
                "annotations": (
                    [
                        {
                            "id": uuid.uuid4().hex[:12],
                            "kind": "redact",
                            "rect": {"x": 0.62, "y": 0.19, "w": 0.24, "h": 0.06},
                        }
                    ]
                    if index == 2
                    else []
                ),
                "include": True,
                "locked": index == 1,
                "status": status,
                "error": None,
                "manual": index == 4,
            }
        )

    project = {
        "id": PROJECT_ID,
        "title": "Send an invoice to a customer",
        "summary": (
            "Create an invoice from the billing area, add the customer and line items, "
            "then send it for payment. Takes about two minutes."
        ),
        "prerequisites": [
            "An account with the Billing permission",
            "The customer already saved in your address book",
        ],
        "createdAt": created.isoformat().replace("+00:00", "Z"),
        "updatedAt": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "sourceLabel": "Built-in Retina Display",
        "steps": steps,
    }
    (root / "project.json").write_text(json.dumps(project, indent=2))
    print(f"seeded {root} with {len(steps)} steps")


if __name__ == "__main__":
    main()
