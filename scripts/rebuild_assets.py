from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from PIL import Image, ImageDraw
from PIL.ImageFilter import MaxFilter


ROOT = Path(__file__).resolve().parent.parent
RAW_DIR = ROOT / "output" / "imagegen" / "raw"
FINAL_DIR = ROOT / "output" / "imagegen" / "final"
FINAL_DIR.mkdir(parents=True, exist_ok=True)

SOURCE_EXTENSIONS = (".png", ".jpg", ".jpeg", ".webp")
PREVIEW_SCALE = 3
GROUND_VARIANT_COUNT = 6
ROAD_VARIANT_COUNT = 4
WATER_VARIANT_COUNT = 4
CARDINAL_MASKS = range(16)
OBSOLETE_OUTPUTS = (
    "metasprite_preview.png",
    "metasprite_preview_x6.png",
    "metasprites.json",
    "nes_sprite_sheet.json",
    "nes_sprite_sheet.png",
    "nes_sprite_sheet_preview.png",
    "subtile_defs.json",
    "subtile_sheet.png",
)


@dataclass(frozen=True)
class AssetSpec:
    source: str
    draw_size: tuple[int, int]
    anchor: tuple[int, int]


ASSET_SPECS: dict[str, AssetSpec] = {
    "jeep_up": AssetSpec("jeep_up", (48, 48), (24, 24)),
    "jeep_right": AssetSpec("jeep_right", (48, 48), (24, 24)),
    "jeep_down": AssetSpec("jeep_down", (48, 48), (24, 24)),
    "jeep_left": AssetSpec("jeep_left", (48, 48), (24, 24)),
    "enemy_commando": AssetSpec("enemy_commando", (16, 16), (8, 8)),
    "enemy_rocketeer": AssetSpec("enemy_rocketeer", (16, 16), (8, 8)),
    "enemy_soldier": AssetSpec("enemy_soldier", (16, 16), (8, 8)),
    "hostage": AssetSpec("hostage", (16, 16), (8, 8)),
    "explosion": AssetSpec("explosion", (24, 24), (12, 12)),
    "grass_tile": AssetSpec("grass_tile", (32, 32), (0, 0)),
    "road_tile": AssetSpec("road_tile", (32, 32), (0, 0)),
    "water_tile": AssetSpec("water_tile", (32, 32), (0, 0)),
    "wall_tile": AssetSpec("wall_tile", (32, 32), (0, 0)),
    "cage_tile": AssetSpec("cage_tile", (32, 32), (0, 0)),
    "extraction_tile": AssetSpec("extraction_tile", (32, 32), (0, 0)),
    "bunker_turret": AssetSpec("bunker_turret", (64, 64), (32, 32)),
    "palm_tree": AssetSpec("palm_tree", (48, 64), (24, 55)),
}


def find_source_path(name: str) -> Path | None:
    for ext in SOURCE_EXTENSIONS:
        path = RAW_DIR / f"{name}{ext}"
        if path.exists():
            return path
    return None


def load_image_from_path(
    path: Path, alpha_threshold: int = 64, preserve_size: tuple[int, int] | None = None
) -> Image.Image:
    image = Image.open(path).convert("RGBA")
    if preserve_size is not None and image.size == preserve_size:
        image.putalpha(
            image.getchannel("A").point(lambda a: 255 if a >= alpha_threshold else 0)
        )
        return image

    alpha = image.getchannel("A")
    hard_alpha = alpha.point(lambda a: 255 if a >= alpha_threshold else 0)
    bbox = hard_alpha.getbbox()
    if bbox is None:
        return image
    cropped = image.crop(bbox)
    cropped.putalpha(cropped.getchannel("A").point(lambda a: 255 if a >= alpha_threshold else 0))
    return cropped


def trim_dark_matte(image: Image.Image, darkness_threshold: int = 12) -> Image.Image:
    corners = [
        image.getpixel((0, 0)),
        image.getpixel((image.width - 1, 0)),
        image.getpixel((0, image.height - 1)),
        image.getpixel((image.width - 1, image.height - 1)),
    ]
    if not all(max(r, g, b) <= darkness_threshold and a >= 250 for r, g, b, a in corners):
        return image

    bbox: tuple[int, int, int, int] | None = None
    for y in range(image.height):
        for x in range(image.width):
            r, g, b, a = image.getpixel((x, y))
            if a < 16:
                continue
            if max(r, g, b) <= darkness_threshold:
                continue
            if bbox is None:
                bbox = (x, y, x + 1, y + 1)
            else:
                left, top, right, bottom = bbox
                bbox = (
                    min(left, x),
                    min(top, y),
                    max(right, x + 1),
                    max(bottom, y + 1),
                )

    if bbox is None:
        return image
    return image.crop(bbox)


def inset_tile_edges(image: Image.Image, inset_ratio: float = 0.025) -> Image.Image:
    inset = max(1, int(round(min(image.width, image.height) * inset_ratio)))
    if inset * 2 >= image.width or inset * 2 >= image.height:
        return image
    return image.crop((inset, inset, image.width - inset, image.height - inset))


def crop_to_aspect(image: Image.Image, size: tuple[int, int]) -> Image.Image:
    target_aspect = size[0] / size[1]
    image_aspect = image.width / image.height

    if abs(image_aspect - target_aspect) < 0.0001:
        return image

    if image_aspect > target_aspect:
        new_width = max(1, int(round(image.height * target_aspect)))
        left = max(0, (image.width - new_width) // 2)
        return image.crop((left, 0, left + new_width, image.height))

    new_height = max(1, int(round(image.width / target_aspect)))
    top = max(0, (image.height - new_height) // 2)
    return image.crop((0, top, image.width, top + new_height))


def load_source_image(name: str, target_size: tuple[int, int]) -> Image.Image:
    path = find_source_path(name)
    if path is not None:
        image = load_image_from_path(path, preserve_size=target_size)
        if name.endswith("_tile"):
            image = trim_dark_matte(image)
            image = inset_tile_edges(image)
        return image

    if name == "bunker_turret":
        fallback = find_source_path("wall_tile")
        if fallback is not None:
            return load_image_from_path(fallback, preserve_size=target_size)

    if name == "palm_tree":
        fallback = find_source_path("grass_tile")
        if fallback is not None:
            return load_image_from_path(fallback, preserve_size=target_size)

    if name in {"enemy_commando", "enemy_rocketeer"}:
        fallback = find_source_path("enemy_soldier")
        if fallback is not None:
            return load_image_from_path(fallback, preserve_size=target_size)

    if name.startswith("jeep_"):
        base_path = find_source_path("jeep_up")
        if base_path is None:
            raise FileNotFoundError("missing raw source: expected jeep_up.[png|jpg|jpeg|webp]")
        base = load_image_from_path(base_path, preserve_size=target_size)
        if name == "jeep_right":
            return base.transpose(Image.Transpose.ROTATE_270)
        if name == "jeep_down":
            return base.transpose(Image.Transpose.ROTATE_180)
        if name == "jeep_left":
            return base.transpose(Image.Transpose.ROTATE_90)
        if name == "jeep_up":
            return base

    raise FileNotFoundError(f"missing raw source for {name}")


def fit_to_size(image: Image.Image, size: tuple[int, int]) -> Image.Image:
    if image.size == size:
        return image.copy()

    scale = min(size[0] / image.width, size[1] / image.height)
    resized = image.resize(
        (
            max(1, int(round(image.width * scale))),
            max(1, int(round(image.height * scale))),
        ),
        Image.Resampling.NEAREST,
    )
    canvas = Image.new("RGBA", size, (0, 0, 0, 0))
    offset = (
        (size[0] - resized.width) // 2,
        (size[1] - resized.height) // 2,
    )
    canvas.paste(resized, offset, resized)
    return canvas


def fit_tile_to_size(image: Image.Image, size: tuple[int, int]) -> Image.Image:
    image = crop_to_aspect(image, size)
    if image.size == size:
        return image.copy()
    return image.resize(size, Image.Resampling.NEAREST)


def load_tile_variant_set(
    names: list[str], size: tuple[int, int], count: int
) -> list[Image.Image]:
    variants: list[Image.Image] = []
    seen: set[bytes] = set()

    for name in names:
        path = find_source_path(name)
        if path is None:
            continue
        base = fit_tile_to_size(load_source_image(name, size), size)
        for image in build_variants(base):
            key = image.tobytes()
            if key in seen:
                continue
            seen.add(key)
            variants.append(image)
            if len(variants) >= count:
                return variants

    if not variants:
        raise FileNotFoundError(f"missing tile sources for {names[0]}")
    return variants


def color_shift(image: Image.Image, red: float, green: float, blue: float) -> Image.Image:
    shifted = image.copy().convert("RGBA")
    pixels = shifted.load()
    for y in range(shifted.height):
        for x in range(shifted.width):
            r, g, b, a = pixels[x, y]
            pixels[x, y] = (
                max(0, min(255, int(round(r * red)))),
                max(0, min(255, int(round(g * green)))),
                max(0, min(255, int(round(b * blue)))),
                a,
            )
    return shifted


def cleanup_jeep_image(image: Image.Image) -> Image.Image:
    rgba = image.convert("RGBA")
    core = Image.new("L", rgba.size, 0)
    core_pixels = core.load()

    for y in range(rgba.height):
        for x in range(rgba.width):
            r, g, b, a = rgba.getpixel((x, y))
            if a == 0:
                continue
            brightness = r + g + b
            if brightness >= 150 or (g > r + 12 and g > b + 12):
                core_pixels[x, y] = 255

    influence = core
    for _ in range(4):
        influence = influence.filter(MaxFilter(5))

    cleaned = rgba.copy()
    pixels = cleaned.load()
    for y in range(cleaned.height):
        for x in range(cleaned.width):
            r, g, b, a = pixels[x, y]
            if a == 0:
                continue
            if influence.getpixel((x, y)) == 0:
                pixels[x, y] = (0, 0, 0, 0)

    return cleaned


def build_variants(base: Image.Image) -> list[Image.Image]:
    return [
        base,
        base.transpose(Image.Transpose.FLIP_LEFT_RIGHT),
        base.transpose(Image.Transpose.FLIP_TOP_BOTTOM),
        base.transpose(Image.Transpose.ROTATE_180),
    ]


def build_transition_mask(
    size: tuple[int, int],
    mask: int,
    *,
    inset: int,
    radius: int,
) -> Image.Image:
    if mask == 0b1111:
        return Image.new("L", size, 255)

    width, height = size
    mask_image = Image.new("L", size, 255)
    draw = ImageDraw.Draw(mask_image)

    if mask & 0b0001 == 0:
        draw.rectangle((0, 0, width, inset), fill=0)
    if mask & 0b0010 == 0:
        draw.rectangle((width - inset, 0, width, height), fill=0)
    if mask & 0b0100 == 0:
        draw.rectangle((0, height - inset, width, height), fill=0)
    if mask & 0b1000 == 0:
        draw.rectangle((0, 0, inset, height), fill=0)

    diameter = radius * 2
    if mask & 0b0001 == 0 and mask & 0b1000 == 0:
        draw.pieslice((0, 0, diameter, diameter), 180, 270, fill=0)
    if mask & 0b0001 == 0 and mask & 0b0010 == 0:
        draw.pieslice((width - diameter, 0, width, diameter), 270, 360, fill=0)
    if mask & 0b0100 == 0 and mask & 0b1000 == 0:
        draw.pieslice((0, height - diameter, diameter, height), 90, 180, fill=0)
    if mask & 0b0100 == 0 and mask & 0b0010 == 0:
        draw.pieslice((width - diameter, height - diameter, width, height), 0, 90, fill=0)

    return mask_image


def build_transition_overlay(
    texture: Image.Image,
    mask: int,
    *,
    inset: int,
    radius: int,
) -> Image.Image:
    alpha = build_transition_mask(texture.size, mask, inset=inset, radius=radius)
    overlay = Image.new("RGBA", texture.size, (0, 0, 0, 0))
    overlay.paste(texture, (0, 0), alpha)
    return overlay


def save(name: str, image: Image.Image) -> str:
    path = FINAL_DIR / f"{name}.png"
    image.save(path)
    return str(path.relative_to(ROOT))


def manifest_entry(file: str, draw_size: tuple[int, int], anchor: tuple[int, int]) -> dict[str, float | str]:
    return {
        "file": file,
        "draw_width": draw_size[0],
        "draw_height": draw_size[1],
        "anchor_x": anchor[0],
        "anchor_y": anchor[1],
    }


def export_assets() -> dict[str, dict[str, float | str]]:
    manifest: dict[str, dict[str, float | str]] = {}

    for name, spec in ASSET_SPECS.items():
        if name.endswith("_tile"):
            image = fit_tile_to_size(
                load_source_image(spec.source, spec.draw_size), spec.draw_size
            )
        else:
            image = fit_to_size(load_source_image(spec.source, spec.draw_size), spec.draw_size)
        if name.startswith("jeep_"):
            image = cleanup_jeep_image(image)
        manifest[name] = manifest_entry(save(name, image), spec.draw_size, spec.anchor)

    grass_base = fit_tile_to_size(
        load_source_image("grass_tile", ASSET_SPECS["grass_tile"].draw_size),
        ASSET_SPECS["grass_tile"].draw_size,
    )
    grass_variants = [
        grass_base,
        grass_base.transpose(Image.Transpose.FLIP_LEFT_RIGHT),
        grass_base.transpose(Image.Transpose.FLIP_TOP_BOTTOM),
        grass_base.transpose(Image.Transpose.ROTATE_180),
        color_shift(grass_base, 0.96, 1.00, 0.95),
        color_shift(grass_base, 1.04, 1.03, 0.98),
    ][:GROUND_VARIANT_COUNT]
    road_variants = load_tile_variant_set(
        ["road_tile", "road_tile_alt_1"],
        ASSET_SPECS["road_tile"].draw_size,
        ROAD_VARIANT_COUNT,
    )
    water_variants = load_tile_variant_set(
        ["water_tile", "water_tile_alt_1"],
        ASSET_SPECS["water_tile"].draw_size,
        WATER_VARIANT_COUNT,
    )
    wall = fit_tile_to_size(load_source_image("wall_tile", ASSET_SPECS["wall_tile"].draw_size), ASSET_SPECS["wall_tile"].draw_size)

    for idx, image in enumerate(grass_variants):
        manifest[f"ground_{idx}"] = manifest_entry(save(f"ground_{idx}", image), ASSET_SPECS["grass_tile"].draw_size, (0, 0))
    for variant, image in enumerate(road_variants):
        manifest[f"road_fill_{variant}"] = manifest_entry(
            save(f"road_fill_{variant}", image),
            ASSET_SPECS["road_tile"].draw_size,
            (0, 0),
        )
        for mask in CARDINAL_MASKS:
            overlay = build_transition_overlay(image, mask, inset=7, radius=12)
            manifest[f"road_overlay_{variant}_{mask}"] = manifest_entry(
                save(f"road_overlay_{variant}_{mask}", overlay),
                ASSET_SPECS["road_tile"].draw_size,
                (0, 0),
            )
    for variant, image in enumerate(water_variants):
        manifest[f"water_fill_{variant}"] = manifest_entry(
            save(f"water_fill_{variant}", image),
            ASSET_SPECS["water_tile"].draw_size,
            (0, 0),
        )
        for mask in CARDINAL_MASKS:
            overlay = build_transition_overlay(image, mask, inset=5, radius=15)
            manifest[f"water_overlay_{variant}_{mask}"] = manifest_entry(
                save(f"water_overlay_{variant}_{mask}", overlay),
                ASSET_SPECS["water_tile"].draw_size,
                (0, 0),
            )
    for idx, image in enumerate(build_variants(wall)[:2]):
        manifest[f"wall_{idx}"] = manifest_entry(save(f"wall_{idx}", image), ASSET_SPECS["wall_tile"].draw_size, (0, 0))

    return manifest


def build_preview(manifest: dict[str, dict[str, float | str]]) -> None:
    preview_names = [
        "jeep_up",
        "jeep_right",
        "jeep_down",
        "jeep_left",
        "enemy_commando",
        "enemy_rocketeer",
        "enemy_soldier",
        "hostage",
        "explosion",
        "ground_0",
        "ground_1",
        "road_fill_0",
        "road_overlay_0_9",
        "water_fill_0",
        "water_overlay_0_3",
        "wall_0",
        "wall_1",
        "cage_tile",
        "extraction_tile",
        "bunker_turret",
        "palm_tree",
    ]
    total_width = sum(int(manifest[name]["draw_width"]) for name in preview_names) + (len(preview_names) - 1) * 24
    total_height = max(int(manifest[name]["draw_height"]) for name in preview_names)
    preview = Image.new("RGBA", (total_width, total_height), (0, 0, 0, 0))

    cursor_x = 0
    for name in preview_names:
        image = Image.open(ROOT / str(manifest[name]["file"])).convert("RGBA")
        y = (total_height - image.height) // 2
        preview.paste(image, (cursor_x, y), image)
        cursor_x += image.width + 24

    preview.save(FINAL_DIR / "asset_preview.png")
    preview.resize(
        (preview.width * PREVIEW_SCALE, preview.height * PREVIEW_SCALE),
        Image.Resampling.NEAREST,
    ).save(FINAL_DIR / "asset_preview_x3.png")


def cleanup_obsolete_outputs() -> None:
    for name in OBSOLETE_OUTPUTS:
        path = FINAL_DIR / name
        if path.exists():
            path.unlink()


def main() -> None:
    cleanup_obsolete_outputs()
    manifest = export_assets()
    (FINAL_DIR / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True))
    build_preview(manifest)


if __name__ == "__main__":
    main()
