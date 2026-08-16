# B1 — Colour

**Goal:** the M0 fBm field as a colour PNG, in a per-run directory.

This is the instrument, not the artefact. It comes first because every milestone
after it is judged by eye, and greyscale can't show a coastline, a land cover or a
hillshade. One sitting.

Replaces the two greyscale writers in `render.rs`, which have served their purpose.

---

## What to build

**A ramp** — sorted control points, each a value and a colour, interpolated
between. This is what continuous layers use: elevation, temperature, humidity.

**A palette** — an index or enum variant to a colour, no interpolation. This is
what categorical layers use, which from B4 means land cover.

Both feed one writer that takes a `Grid<T>` and a mapping and produces an
`image::RgbImage`. The mapping is the parameter; the writer shouldn't care which
kind it got.

**A per-run output directory.** `<out>/<seed>/noise.png` rather than the current
`<out>/<seed>.noise.png`. Three lines, taken from
[m1a-logging.md](../detailed/m1a-logging.md) while leaving the rest of that
document deferred. From B3 onward every run writes several PNGs and a flat prefix
scheme stops working; it also means the last few runs stay on disk to compare.

**A `--scale N` flag**, nearest-neighbour upscale at write time. 512 × 512 is
postage-stamp sized on a modern display, and the difference between a screenshot
that looks intentional and one that looks like a test fixture is often just that.
Nearest neighbour, not smooth — you want to see cells, and later you'll want to
count them when something looks wrong.

---

## Gotchas

**Fix the ramp's domain explicitly.** The tempting thing is to normalise each
image to its own min and max. Don't. Colours then mean different things between
one run and the next, and between one layer and another, so two images can't be
compared — which is the only thing you'll actually want to do with them. Compute a
domain from the data once, then pin it for the run.

This matters more than it sounds. Half of tuning is "did that change help?", and
that question is unanswerable if the colours moved too.

**Interpolating in sRGB goes muddy through the middle.** Lerping between two
saturated colours passes through a desaturated grey rather than the hue you
expected. With a hypsometric ramp's many close control points it won't show, so
don't fix it now — just recognise it if a two-point ramp ever looks wrong for no
apparent reason.

**Palette colours key off the value, not iteration order.** Obvious with an enum,
easy to get wrong the moment anything is sorted or filtered upstream.

---

## Acceptance

1. The M0 fBm field renders as a colour PNG with a pinned domain.
2. Output lands in `<out>/<seed>/`, and a second run with a different seed doesn't
   overwrite it.
3. `--scale 3` produces a 1536 × 1536 image with visible square cells, not a
   blurred one.

The result won't be worth showing anyone. That's expected — it's noise with
colours on it. B2 is what this was built for.
