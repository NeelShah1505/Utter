#!/usr/bin/env python3
"""
Generate pixel-perfect, minimalist macOS Menu Bar Microphone Icon.
Produces crisp template icons (monochrome with alpha) matching macOS native design.
"""
from PIL import Image, ImageDraw
import os

def render_mic_icon(size):
    # Render at 8x scale for smooth antialiased curves, then downsample with LANCZOS
    scale = 8
    canvas_size = size * scale
    im = Image.new('RGBA', (canvas_size, canvas_size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(im)

    # Coordinate mapping from 22pt standard macOS menu bar grid
    factor = canvas_size / 22.0
    stroke_w = int(1.6 * factor)

    # 1. Capsule (Microphone body)
    cap_w = 6.4 * factor
    cap_h = 10.0 * factor
    cap_x0 = (canvas_size - cap_w) / 2.0
    cap_y0 = 2.0 * factor
    cap_x1 = cap_x0 + cap_w
    cap_y1 = cap_y0 + cap_h
    radius = cap_w / 2.0

    # Draw outlined rounded rectangle (capsule)
    draw.rounded_rectangle(
        [cap_x0, cap_y0, cap_x1, cap_y1],
        radius=radius,
        outline=(255, 255, 255, 255),
        width=stroke_w
    )

    # 2. U-shaped Cradle
    cradle_w = 10.8 * factor
    cradle_x0 = (canvas_size - cradle_w) / 2.0
    cradle_x1 = cradle_x0 + cradle_w
    cradle_top_y = 6.2 * factor
    cradle_bot_y = 13.0 * factor
    cradle_r = cradle_w / 2.0

    # Left vertical arm
    draw.line([(cradle_x0, cradle_top_y), (cradle_x0, cradle_bot_y - cradle_r)], fill=(255, 255, 255, 255), width=stroke_w)
    # Right vertical arm
    draw.line([(cradle_x1, cradle_top_y), (cradle_x1, cradle_bot_y - cradle_r)], fill=(255, 255, 255, 255), width=stroke_w)
    # Bottom arc
    arc_box = [cradle_x0, cradle_bot_y - 2 * cradle_r, cradle_x1, cradle_bot_y]
    draw.arc(arc_box, start=0, end=180, fill=(255, 255, 255, 255), width=stroke_w)

    # 3. Stem (vertical line from cradle center)
    stem_top_y = cradle_bot_y
    stem_bot_y = 17.6 * factor
    mid_x = canvas_size / 2.0
    draw.line([(mid_x, stem_top_y), (mid_x, stem_bot_y)], fill=(255, 255, 255, 255), width=stroke_w)

    # 4. Base (horizontal foot line)
    base_w = 7.6 * factor
    base_x0 = (canvas_size - base_w) / 2.0
    base_x1 = base_x0 + base_w
    draw.line([(base_x0, stem_bot_y), (base_x1, stem_bot_y)], fill=(255, 255, 255, 255), width=stroke_w)

    # Downsample cleanly
    return im.resize((size, size), Image.Resampling.LANCZOS)

def main():
    icons_dir = "apps/desktop/src-tauri/icons"
    os.makedirs(icons_dir, exist_ok=True)

    # Standard 1x: 22x22
    im_1x = render_mic_icon(22)
    im_1x.save(os.path.join(icons_dir, "tray-icon.png"))
    im_1x.save(os.path.join(icons_dir, "tray-iconTemplate.png"))

    # High-DPI 2x: 44x44
    im_2x = render_mic_icon(44)
    im_2x.save(os.path.join(icons_dir, "tray-icon@2x.png"))
    im_2x.save(os.path.join(icons_dir, "tray-iconTemplate@2x.png"))

    # Ultra 3x: 66x66
    im_3x = render_mic_icon(66)
    im_3x.save(os.path.join(icons_dir, "tray-icon@3x.png"))
    im_3x.save(os.path.join(icons_dir, "tray-iconTemplate@3x.png"))

    print("✓ Successfully generated minimalist macOS menu bar mic icons (1x, 2x, 3x)!")

if __name__ == "__main__":
    main()
