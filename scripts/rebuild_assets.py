from __future__ import annotations

import json
from pathlib import Path

from PIL import Image, ImageDraw


ROOT = Path(__file__).resolve().parent.parent
FINAL_DIR = ROOT / "output" / "imagegen" / "final"
FINAL_DIR.mkdir(parents=True, exist_ok=True)

ORDER = [
    "jeep_up",
    "jeep_right",
    "jeep_down",
    "jeep_left",
    "enemy_soldier",
    "hostage",
    "explosion",
    "grass_tile",
    "road_tile",
    "water_tile",
    "wall_tile",
    "cage_tile",
    "extraction_tile",
]


def rgba(color: str) -> tuple[int, int, int, int]:
    color = color.lstrip("#")
    return (
        int(color[0:2], 16),
        int(color[2:4], 16),
        int(color[4:6], 16),
        255,
    )


PALETTE = {
    "clear": (0, 0, 0, 0),
    "outline": rgba("#101010"),
    "shadow": rgba("#3A2B18"),
    "sand_dark": rgba("#A96D18"),
    "sand_mid": rgba("#C88420"),
    "sand_light": rgba("#E2A436"),
    "concrete_dark": rgba("#545454"),
    "concrete_mid": rgba("#7B7B7B"),
    "concrete_light": rgba("#A8A8A8"),
    "green_dark": rgba("#0E4A20"),
    "green_mid": rgba("#0F7A33"),
    "green_light": rgba("#35C54A"),
    "olive": rgba("#5C6B1E"),
    "tan": rgba("#F0D880"),
    "rust": rgba("#9B3820"),
    "red": rgba("#D44828"),
    "orange": rgba("#F0A018"),
    "yellow": rgba("#FFE060"),
    "blue_dark": rgba("#204070"),
    "blue_light": rgba("#6CB8FF"),
    "white": rgba("#F8F8F8"),
}


def canvas() -> tuple[Image.Image, ImageDraw.ImageDraw]:
    image = Image.new("RGBA", (16, 16), PALETTE["clear"])
    return image, ImageDraw.Draw(image)


def put(draw: ImageDraw.ImageDraw, x: int, y: int, color: str) -> None:
    draw.point((x, y), fill=PALETTE[color])


def fill_rect(
    draw: ImageDraw.ImageDraw,
    x0: int,
    y0: int,
    x1: int,
    y1: int,
    color: str,
) -> None:
    draw.rectangle((x0, y0, x1, y1), fill=PALETTE[color])


def outline_rect(
    draw: ImageDraw.ImageDraw,
    x0: int,
    y0: int,
    x1: int,
    y1: int,
    color: str,
) -> None:
    draw.rectangle((x0, y0, x1, y1), outline=PALETTE[color])


def save(name: str, image: Image.Image) -> None:
    image.save(FINAL_DIR / f"{name}.png")


def sand_tile() -> Image.Image:
    image, draw = canvas()
    fill_rect(draw, 0, 0, 15, 15, "sand_mid")
    for y in range(16):
        for x in range(16):
            if (x * 5 + y * 3) % 11 in (0, 1):
                put(draw, x, y, "sand_dark")
            elif (x * 7 + y * 5) % 13 == 0:
                put(draw, x, y, "sand_light")
    return image


def road_tile() -> Image.Image:
    image, draw = canvas()
    fill_rect(draw, 0, 0, 15, 15, "concrete_mid")
    for y in (3, 7, 11):
        fill_rect(draw, 0, y, 15, y, "concrete_dark")
    for x in (4, 9, 14):
        fill_rect(draw, x, 0, x, 15, "concrete_light")
    cracks = [
        [(2, 2), (3, 3), (4, 4), (3, 4)],
        [(11, 2), (10, 3), (9, 4), (10, 4)],
        [(5, 9), (6, 10), (7, 11), (6, 11)],
        [(12, 9), (11, 10), (10, 11), (11, 11)],
    ]
    for crack in cracks:
        for x, y in crack:
            put(draw, x, y, "outline")
    return image


def water_tile() -> Image.Image:
    image, draw = canvas()
    fill_rect(draw, 0, 0, 15, 15, "blue_dark")
    for y in (2, 6, 10, 14):
        for x in range((y // 2) % 4, 16, 4):
            fill_rect(draw, x, y, min(x + 1, 15), y, "blue_light")
            if y + 1 < 16:
                put(draw, (x + 2) % 16, y + 1, "white")
    return image


def wall_tile() -> Image.Image:
    image, draw = canvas()
    fill_rect(draw, 0, 0, 15, 15, "concrete_mid")
    for y in range(0, 16, 4):
        fill_rect(draw, 0, y, 15, min(y + 1, 15), "concrete_dark")
    for x in range(0, 16, 5):
        fill_rect(draw, x, 0, min(x + 1, 15), 15, "concrete_light")
    for x in (3, 8, 13):
        for y in (3, 8, 13):
            put(draw, x, y, "outline")
    outline_rect(draw, 0, 0, 15, 15, "outline")
    return image


def cage_tile() -> Image.Image:
    image, draw = canvas()
    fill_rect(draw, 0, 0, 15, 15, "sand_mid")
    fill_rect(draw, 2, 2, 13, 13, "concrete_mid")
    for x in (4, 7, 10):
        fill_rect(draw, x, 3, x, 12, "tan")
    fill_rect(draw, 3, 4, 12, 4, "outline")
    fill_rect(draw, 3, 11, 12, 11, "outline")
    outline_rect(draw, 2, 2, 13, 13, "outline")
    return image


def extraction_tile() -> Image.Image:
    image, draw = canvas()
    fill_rect(draw, 0, 0, 15, 15, "green_mid")
    fill_rect(draw, 2, 2, 13, 13, "sand_light")
    fill_rect(draw, 7, 3, 8, 12, "concrete_dark")
    fill_rect(draw, 3, 7, 12, 8, "concrete_dark")
    outline_rect(draw, 2, 2, 13, 13, "outline")
    return image


def jeep(direction: str) -> Image.Image:
    image, draw = canvas()
    fill_rect(draw, 4, 3, 11, 12, "green_mid")
    fill_rect(draw, 5, 4, 10, 11, "green_light")
    fill_rect(draw, 5, 6, 10, 8, "white")
    fill_rect(draw, 3, 5, 3, 10, "outline")
    fill_rect(draw, 12, 5, 12, 10, "outline")
    fill_rect(draw, 4, 12, 11, 13, "rust")
    if direction == "up":
        fill_rect(draw, 6, 2, 9, 2, "tan")
        fill_rect(draw, 6, 1, 9, 1, "outline")
        fill_rect(draw, 7, 13, 8, 14, "rust")
    elif direction == "down":
        fill_rect(draw, 6, 13, 9, 13, "tan")
        fill_rect(draw, 6, 14, 9, 14, "outline")
        fill_rect(draw, 7, 1, 8, 2, "rust")
    elif direction == "left":
        fill_rect(draw, 2, 6, 2, 9, "tan")
        fill_rect(draw, 1, 6, 1, 9, "outline")
        fill_rect(draw, 13, 7, 14, 8, "rust")
    else:
        fill_rect(draw, 13, 6, 13, 9, "tan")
        fill_rect(draw, 14, 6, 14, 9, "outline")
        fill_rect(draw, 1, 7, 2, 8, "rust")
    outline_rect(draw, 4, 3, 11, 12, "outline")
    return image


def enemy_soldier() -> Image.Image:
    image, draw = canvas()
    fill_rect(draw, 6, 3, 9, 5, "tan")
    fill_rect(draw, 5, 6, 10, 10, "olive")
    fill_rect(draw, 7, 11, 8, 13, "outline")
    fill_rect(draw, 4, 8, 4, 11, "outline")
    fill_rect(draw, 11, 7, 12, 8, "outline")
    fill_rect(draw, 5, 2, 10, 3, "outline")
    put(draw, 7, 7, "red")
    put(draw, 8, 7, "red")
    return image


def hostage() -> Image.Image:
    image, draw = canvas()
    fill_rect(draw, 6, 3, 9, 5, "tan")
    fill_rect(draw, 5, 6, 10, 10, "white")
    fill_rect(draw, 6, 11, 7, 13, "blue_dark")
    fill_rect(draw, 8, 11, 9, 13, "blue_dark")
    fill_rect(draw, 4, 7, 4, 9, "outline")
    fill_rect(draw, 11, 7, 11, 9, "outline")
    fill_rect(draw, 5, 2, 10, 3, "outline")
    return image


def explosion() -> Image.Image:
    image, draw = canvas()
    for x, y in [
        (7, 2),
        (8, 2),
        (5, 4),
        (10, 4),
        (4, 7),
        (11, 7),
        (5, 10),
        (10, 10),
        (7, 12),
        (8, 12),
    ]:
        put(draw, x, y, "orange")
    fill_rect(draw, 6, 5, 9, 9, "yellow")
    fill_rect(draw, 7, 6, 8, 8, "white")
    for x, y in [(6, 3), (9, 3), (3, 7), (12, 7), (6, 11), (9, 11)]:
        put(draw, x, y, "red")
    return image


def write_sheet() -> None:
    cols = 7
    rows = (len(ORDER) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * 16, rows * 16), (0, 0, 0, 0))
    metadata: dict[str, dict[str, int]] = {}

    for idx, name in enumerate(ORDER):
        x = (idx % cols) * 16
        y = (idx // cols) * 16
        image = Image.open(FINAL_DIR / f"{name}.png").convert("RGBA")
        sheet.paste(image, (x, y), image)
        metadata[name] = {"x": x, "y": y, "w": 16, "h": 16}

    sheet.save(FINAL_DIR / "nes_sprite_sheet.png")
    sheet.resize((sheet.width * 8, sheet.height * 8), Image.Resampling.NEAREST).save(
        FINAL_DIR / "nes_sprite_sheet_preview.png"
    )
    (FINAL_DIR / "nes_sprite_sheet.json").write_text(json.dumps(metadata, indent=2))


def main() -> None:
    save("grass_tile", sand_tile())
    save("road_tile", road_tile())
    save("water_tile", water_tile())
    save("wall_tile", wall_tile())
    save("cage_tile", cage_tile())
    save("extraction_tile", extraction_tile())
    save("jeep_up", jeep("up"))
    save("jeep_right", jeep("right"))
    save("jeep_down", jeep("down"))
    save("jeep_left", jeep("left"))
    save("enemy_soldier", enemy_soldier())
    save("hostage", hostage())
    save("explosion", explosion())
    write_sheet()


if __name__ == "__main__":
    main()
