#!/bin/sh
# Regenerates the JPEG decoder fixtures in this directory; see README.md.
#
# Needs cjpeg/djpeg from libjpeg-turbo (the committed files were made with
# libjpeg-turbo 3.2.0) and python3 for the deterministic source patterns.
set -eu
cd "$(dirname "$0")"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# pattern WIDTH HEIGHT OUT.ppm
# Horizontal red ramp, vertical green ramp, a 4-pixel blue checkerboard (sharp
# edges in every channel) and one saturated block, so that luma and both
# chroma channels all carry detail at every sampling factor.
pattern() {
    python3 - "$1" "$2" "$3" <<'PY'
import sys
w, h, out = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3]
px = bytearray()
for y in range(h):
    for x in range(w):
        r = (x * 255) // (w - 1)
        g = (y * 255) // (h - 1)
        b = ((x // 4 + y // 4) % 2) * 200 + 30
        if 8 <= x < 20 and 6 <= y < 18:
            r, g, b = 250, 20, 40
        px += bytes((r, g, b))
with open(out, "wb") as f:
    f.write(b"P6\n%d %d\n255\n" % (w, h))
    f.write(px)
PY
}

pattern 37 29 "$tmp/p37.ppm"
pattern 64 48 "$tmp/p64.ppm"
printf '0;\n1;\n2;\n' > "$tmp/sequential-scans.txt"

# enc NAME SOURCE CJPEG-OPTIONS...
enc() {
    name=$1
    src=$2
    shift 2
    cjpeg "$@" -outfile "$name.jpg" "$src"
}

# ref NAME: decoder reference, box (replicating) chroma upsampling, accurate
# integer IDCT. PPM for colour, PGM for greyscale.
ref() {
    djpeg -nosmooth -dct int -pnm -outfile "$1.ref.pnm" "$1.jpg"
}

enc s420-q75      "$tmp/p37.ppm" -quality 75
enc s422-q75      "$tmp/p37.ppm" -quality 75 -sample 2x1
enc s440-q75      "$tmp/p37.ppm" -quality 75 -sample 1x2
enc s444-q75      "$tmp/p37.ppm" -quality 75 -sample 1x1
enc gray-q75      "$tmp/p37.ppm" -quality 75 -grayscale
enc s420-restart1 "$tmp/p37.ppm" -quality 75 -restart 1
enc s420-restart3b "$tmp/p37.ppm" -quality 75 -restart 3B
enc s420-optimize "$tmp/p37.ppm" -quality 75 -optimize
enc s420-q10      "$tmp/p37.ppm" -quality 10
enc s420-q95      "$tmp/p37.ppm" -quality 95
enc s420-q2-sof1  "$tmp/p37.ppm" -quality 2
enc s444-3scans   "$tmp/p37.ppm" -quality 75 -sample 1x1 -scans "$tmp/sequential-scans.txt"
enc s420-3scans   "$tmp/p37.ppm" -quality 75 -scans "$tmp/sequential-scans.txt"
enc big-s420-q85  "$tmp/p64.ppm" -quality 85
enc big-s444-q85  "$tmp/p64.ppm" -quality 85 -sample 1x1
enc big-gray-q85  "$tmp/p64.ppm" -quality 85 -grayscale

for n in s420-q75 s422-q75 s440-q75 s444-q75 gray-q75 s420-restart1 s420-restart3b \
         s420-optimize s420-q10 s420-q95 s420-q2-sof1 s444-3scans s420-3scans \
         big-s420-q85 big-s444-q85 big-gray-q85; do
    ref "$n"
done

# Encodings the decoder must refuse with a named error; no reference output.
enc reject-progressive "$tmp/p37.ppm" -quality 75 -progressive
enc reject-arithmetic  "$tmp/p37.ppm" -quality 75 -arithmetic
enc reject-12bit       "$tmp/p37.ppm" -quality 75 -precision 12
enc reject-lossless    "$tmp/p37.ppm" -lossless 1
