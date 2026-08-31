#!/usr/bin/env python3
"""Seed mock guides for development and landing-page screenshots.

    python3 scripts/seed_demo.py           # wipe library + seed mock guides
    python3 scripts/seed_demo.py --remove  # delete all projects
"""

from __future__ import annotations

import json
import shutil
import sys
import uuid
from datetime import datetime, timedelta, timezone
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

IDENTIFIER = "app.walkmark.desktop"
FRAME_WIDTH = 1800

PALETTE = [
    ((248, 250, 252), (99, 102, 241), "Acme Billing"),
    ((255, 251, 245), (234, 88, 12), "Northwind Admin"),
    ((245, 253, 250), (13, 148, 136), "Harbor CRM"),
    ((250, 245, 255), (147, 51, 234), "Studio Commerce"),
    ((247, 250, 255), (37, 99, 235), "Pulse Analytics"),
]

GUIDES = [
    {
        "id": "mock-invoice",
        "title": "Send an invoice to a customer",
        "summary": (
            "Create an invoice from the billing area, add the customer and line items, "
            "then send it for payment. Takes about two minutes."
        ),
        "prerequisites": [
            "An account with the Billing permission",
            "The customer already saved in your address book",
        ],
        "sourceLabel": "Acme Billing — Chrome",
        "theme": 0,
        "steps": [
            (
                "Open the billing settings",
                "From the sidebar, select **Settings**, then choose **Billing**. The billing "
                "overview lists your current plan along with the next renewal date.",
            ),
            (
                "Start a new invoice",
                "Click **New invoice** in the top right. A draft is created immediately, so you "
                "can leave and come back without losing your work.",
            ),
            (
                "Pick the customer",
                "Type the customer's name into the **Bill to** field and select them from the "
                "suggestions. Their saved address and tax ID fill in automatically.",
            ),
            (
                "Add the line items",
                "For each item, enter a description, quantity and unit price. The subtotal, tax "
                "and total update as you type.",
            ),
            (
                "Send the invoice",
                "Review the preview on the right, then click **Send**. The customer receives an "
                "email with a payment link, and the invoice moves from *Draft* to *Open*.",
            ),
        ],
        "redact_step": 2,
        "locked_step": 1,
    },
    {
        "id": "mock-slack",
        "title": "Connect Slack to your workspace",
        "summary": (
            "Link Slack so your team gets notified when invoices are paid, deals close, "
            "or a customer needs a reply."
        ),
        "prerequisites": ["Admin access to the workspace", "Permission to install apps in Slack"],
        "sourceLabel": "Northwind Admin",
        "theme": 1,
        "steps": [
            (
                "Open Integrations",
                "In the left sidebar, click **Integrations**. The page lists every service "
                "you can connect and whether it is already active.",
            ),
            (
                "Choose Slack",
                "Find **Slack** in the catalog and click **Connect**. A short description "
                "explains which events will be posted to your channels.",
            ),
            (
                "Authorize the app",
                "Click **Add to Slack** and sign in if prompted. Pick the workspace and channel "
                "where notifications should land, then confirm.",
            ),
            (
                "Pick the events",
                "Back in Northwind, toggle the events you care about — for example *Invoice paid* "
                "and *New customer*. Click **Save**.",
            ),
            (
                "Send a test message",
                "Click **Send test** to verify the connection. You should see a message appear "
                "in Slack within a few seconds.",
            ),
        ],
        "highlight_step": 2,
    },
    {
        "id": "mock-password",
        "title": "Reset a user's password",
        "summary": (
            "Help someone who is locked out get back into their account without sharing "
            "passwords over email or chat."
        ),
        "prerequisites": ["Admin or Support role", "The user's registered email address"],
        "sourceLabel": "Harbor CRM",
        "theme": 2,
        "steps": [
            (
                "Open the user directory",
                "Go to **People** in the sidebar, then switch to the **Users** tab. Search for "
                "the person by name or email.",
            ),
            (
                "Open their profile",
                "Click the user's row to open their profile. The **Access** section shows their "
                "role, last login and MFA status.",
            ),
            (
                "Send a reset link",
                "Click **Reset password**. Choose **Send email** so they receive a secure link, "
                "or **Set temporary password** if they are on a call with you.",
            ),
            (
                "Confirm the action",
                "Review the notice, then click **Reset**. The user receives instructions "
                "immediately and their old password stops working.",
            ),
        ],
    },
    {
        "id": "mock-product",
        "title": "Add a product to your storefront",
        "summary": (
            "Publish a new item with photos, pricing and inventory so customers can buy it "
            "from your shop today."
        ),
        "prerequisites": ["Editor access to the catalog", "Product photos ready to upload"],
        "sourceLabel": "Studio Commerce",
        "theme": 3,
        "steps": [
            (
                "Open the catalog",
                "Click **Products** in the sidebar. The list shows everything that is live, "
                "scheduled or still in draft.",
            ),
            (
                "Create a new product",
                "Click **Add product**. Enter a name and short description customers will see "
                "on the collection page.",
            ),
            (
                "Upload photos",
                "Drag images into the **Media** panel or click **Upload**. The first image "
                "becomes the cover; reorder by dragging the thumbnails.",
            ),
            (
                "Set price and stock",
                "Enter the price, compare-at price if you run sales, and the quantity on hand. "
                "Turn on **Track inventory** to get low-stock alerts.",
            ),
            (
                "Publish",
                "Choose a sales channel, then click **Publish**. The product goes live "
                "immediately unless you picked a future date.",
            ),
        ],
        "locked_step": 3,
    },
    {
        "id": "mock-report",
        "title": "Export a monthly sales report",
        "summary": (
            "Pull revenue, orders and top products for the month into a spreadsheet "
            "you can share with finance."
        ),
        "prerequisites": ["Viewer access to Analytics", "The date range you want to export"],
        "sourceLabel": "Pulse Analytics",
        "theme": 4,
        "steps": [
            (
                "Open Analytics",
                "Click **Analytics** in the sidebar, then choose **Sales**. The overview chart "
                "defaults to the last thirty days.",
            ),
            (
                "Set the date range",
                "Click the date picker and choose **Last month**, or enter custom start and "
                "end dates. The charts and table refresh automatically.",
            ),
            (
                "Choose the breakdown",
                "Open the **Group by** menu and pick **Product** to see which items drove "
                "revenue, or **Channel** to compare storefront and marketplace.",
            ),
            (
                "Export to CSV",
                "Click **Export**, then **CSV**. Pick the columns you need and click "
                "**Download**. The file lands in your Downloads folder.",
            ),
        ],
    },
]


def data_dir() -> Path:
    if sys.platform == "darwin":
        return Path.home() / "Library" / "Application Support" / IDENTIFIER
    if sys.platform == "win32":
        import os

        return Path(os.environ["APPDATA"]) / IDENTIFIER
    return Path.home() / ".local" / "share" / IDENTIFIER


def projects_dir() -> Path:
    return data_dir() / "projects"


def font(size: int) -> ImageFont.FreeTypeFont:
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
    return ImageFont.load_default()


def synthetic_frame(theme_index: int, step_index: int, title: str, app_name: str) -> Image.Image:
    bg, accent, _ = PALETTE[theme_index % len(PALETTE)]
    width, height = FRAME_WIDTH, round(FRAME_WIDTH * 9 / 16)
    img = Image.new("RGB", (width, height), bg)
    d = ImageDraw.Draw(img)

    chrome = 56
    d.rectangle((0, 0, width, chrome), fill=(255, 255, 255))
    d.line((0, chrome, width, chrome), fill=(226, 232, 240), width=2)
    for i, dot in enumerate([(255, 95, 87), (255, 189, 46), (39, 201, 63)]):
        cx = 28 + i * 26
        d.ellipse((cx - 7, chrome // 2 - 7, cx + 7, chrome // 2 + 7), fill=dot)
    d.text((92, chrome // 2 - 10), app_name, font=font(18), fill=(100, 116, 139))

    sidebar = 300
    d.rectangle((0, chrome, sidebar, height), fill=(255, 255, 255))
    d.line((sidebar, chrome, sidebar, height), fill=(226, 232, 240), width=2)
    for row in range(7):
        y = chrome + 46 + row * 52
        selected = row == step_index % 7
        if selected:
            d.rounded_rectangle((16, y - 12, sidebar - 16, y + 26), 10, fill=accent)
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

    d.rounded_rectangle((width - 300, chrome + 44, width - 48, chrome + 96), 12, fill=accent)
    d.rounded_rectangle(
        (sidebar + 48, card_top + 300, width - 48, height - 48), 18, fill=(255, 255, 255)
    )
    return img


def wipe_projects() -> None:
    root = projects_dir()
    if root.exists():
        shutil.rmtree(root)
    root.mkdir(parents=True)


def seed_guide(spec: dict, *, age_hours: int) -> None:
    root = projects_dir() / spec["id"]
    frames = root / "frames"
    frames.mkdir(parents=True)

    theme = spec["theme"]
    app_name = PALETTE[theme % len(PALETTE)][2]
    created = datetime.now(timezone.utc) - timedelta(hours=age_hours)
    updated = datetime.now(timezone.utc) - timedelta(hours=max(0, age_hours - 1))

    steps = []
    for index, (title, body) in enumerate(spec["steps"]):
        offset_ms = 3200 + index * 6800
        name = f"{offset_ms:09}-{uuid.uuid4().hex[:6]}.png"
        synthetic_frame(theme, index, title, app_name).save(frames / name)

        alternates = []
        for delta in (-800, 800):
            alt_name = f"{offset_ms + delta:09}-{uuid.uuid4().hex[:6]}.png"
            synthetic_frame(theme, index + 1, title, app_name).save(frames / alt_name)
            alternates.append(alt_name)

        annotations = []
        if spec.get("redact_step") == index:
            annotations.append(
                {
                    "id": uuid.uuid4().hex[:12],
                    "kind": "redact",
                    "rect": {"x": 0.58, "y": 0.18, "w": 0.28, "h": 0.07},
                }
            )
        if spec.get("highlight_step") == index:
            annotations.append(
                {
                    "id": uuid.uuid4().hex[:12],
                    "kind": "highlight",
                    "rect": {"x": 0.62, "y": 0.12, "w": 0.22, "h": 0.09},
                    "color": "#6366f1",
                    "stroke": "medium",
                }
            )

        steps.append(
            {
                "id": uuid.uuid4().hex[:12],
                "title": title,
                "body": body,
                "offsetMs": offset_ms,
                "frame": name,
                "alternates": alternates,
                "annotations": annotations,
                "include": True,
                "locked": spec.get("locked_step") == index,
                "status": "ready",
                "error": None,
                "manual": False,
            }
        )

    project = {
        "id": spec["id"],
        "title": spec["title"],
        "summary": spec["summary"],
        "prerequisites": spec["prerequisites"],
        "createdAt": created.isoformat().replace("+00:00", "Z"),
        "updatedAt": updated.isoformat().replace("+00:00", "Z"),
        "sourceLabel": spec["sourceLabel"],
        "steps": steps,
    }
    (root / "project.json").write_text(json.dumps(project, indent=2) + "\n")
    print(f"  • {spec['title']} ({len(steps)} steps)")


def main() -> None:
    if "--remove" in sys.argv:
        wipe_projects()
        print(f"removed all projects under {projects_dir()}")
        return

    wipe_projects()
    print(f"Seeding mock guides in {projects_dir()}…")
    for index, guide in enumerate(GUIDES):
        seed_guide(guide, age_hours=72 - index * 14)
    print(f"Done — {len(GUIDES)} guides ready for screenshots.")


if __name__ == "__main__":
    main()
