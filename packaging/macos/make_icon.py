# Renders disktree's macOS icon: a treemap on a dark squircle, at 4x then downsampled.
from PIL import Image, ImageDraw, ImageFilter
S = 4
N = 1024 * S
def s(v): return int(v * S)

img = Image.new("RGBA", (N, N), (0, 0, 0, 0))

# Drop shadow under the body (Apple grid: 824pt body at 100pt inset).
shadow = Image.new("RGBA", (N, N), (0, 0, 0, 0))
ImageDraw.Draw(shadow).rounded_rectangle(
    [s(100), s(112), s(924), s(936)], radius=s(185), fill=(0, 0, 0, 110))
img = Image.alpha_composite(img, shadow.filter(ImageFilter.GaussianBlur(s(14))))

# Body: vertical navy gradient.
grad = Image.new("RGBA", (N, N))
gd = ImageDraw.Draw(grad)
top, bot = (44, 48, 72), (18, 19, 30)
for y in range(N):
    t = y / N
    gd.line([(0, y), (N, y)], fill=tuple(int(a + (b - a) * t) for a, b in zip(top, bot)) + (255,))
mask = Image.new("L", (N, N), 0)
ImageDraw.Draw(mask).rounded_rectangle([s(100), s(100), s(924), s(924)], radius=s(185), fill=255)
img.paste(grad, (0, 0), mask)
d = ImageDraw.Draw(img)

# Subtle top highlight rim.
d.rounded_rectangle([s(100), s(100), s(924), s(924)], radius=s(185),
                    outline=(255, 255, 255, 28), width=s(3))

R = 26  # tile corner radius
BAND = 46

def tile(box, base, band, hatch=False):
    x0, y0, x1, y1 = box
    d.rounded_rectangle([s(x0), s(y0), s(x1), s(y1)], radius=s(R), fill=base)
    # Name band along the top, as the app draws for directories.
    layer = Image.new("RGBA", (N, N), (0, 0, 0, 0))
    ld = ImageDraw.Draw(layer)
    ld.rectangle([s(x0), s(y0), s(x1), s(y0 + BAND)], fill=band)
    if hatch:  # reclaimable space: diagonal hatch
        for k in range(-int(y1 - y0), int(x1 - x0), 34):
            ld.line([(s(x0 + k), s(y1)), (s(x0 + k + (y1 - y0)), s(y0))],
                    fill=(255, 255, 255, 38), width=s(9))
    m = Image.new("L", (N, N), 0)
    ImageDraw.Draw(m).rounded_rectangle([s(x0), s(y0), s(x1), s(y1)], radius=s(R), fill=255)
    img.paste(Image.alpha_composite(img, layer), (0, 0), m)

G = 20
L, T, Rt, B = 186, 186, 838, 838
mid = 556
# Left column
tile((L, T, mid - G/2, 560), (52, 70, 118, 255), (122, 162, 247, 255))              # blue
tile((L, 560 + G, mid - G/2, B), (61, 50, 104, 255), (187, 154, 247, 255))           # purple
# Right column
rx = mid + G/2
tile((rx, T, Rt, 430), (38, 92, 58, 255), (158, 206, 106, 255))                      # green
tile((rx, 430 + G, Rt, 660), (32, 84, 92, 255), (125, 207, 255, 255), hatch=True)    # teal, reclaimable
# Bottom-right small tiles
by = 660 + G
cx = rx + (Rt - rx) * 0.55
tile((rx, by, cx - G/2, B), (98, 72, 40, 255), (224, 175, 104, 255))                 # amber
tile((cx + G/2, by, Rt, B), (92, 46, 58, 255), (247, 118, 142, 255))                 # rose

# Selection outline in the app's amber accent, around the blue tile.
d = ImageDraw.Draw(img)
d.rounded_rectangle([s(L - 9), s(T - 9), s(mid - G/2 + 9), s(560 + 9)],
                    radius=s(R + 9), outline=(255, 199, 99, 255), width=s(10))

out = img.resize((1024, 1024), Image.LANCZOS)
import sys
out.save(sys.argv[1] if len(sys.argv) > 1 else "icon_1024.png")
