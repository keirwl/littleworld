# M1b — The Renderer

**Goal:** turn any `Grid<T>` into a PNG, in colour, and optionally as actual
hexagons.

M0's renderer takes a `Grid<f64>` or a `Grid<u8>` and writes greyscale, one pixel
per cell, with the odd-q shear ignored. That was the right call at M0 and it has
run out of road: [M1d](m1d-plates.md) needs categorical colour for plate ids,
[M1e](m1e-geology.md) needs a ramp for crust age, and both are far easier to read
as hexagons than as a square lattice pretending to be one.

Deliverables: colour mapping in `render.rs`, two draw modes, and two small
additions to `hex.rs`.

This can be built before the world struct in [M1c](m1c-layers.md) and won't need
reworking afterwards, because a renderer takes **one grid and one colour
mapping**. It never needs to know that grids come in named sets — the caller
loops.

---

## Colour mapping

Two kinds, and the distinction is the whole design:

**`Ramp`** — for continuous layers. Control points with colours, interpolated
between. Elevation, crust age, thickness, the fBm field.

**`Palette`** — for categorical layers. An index to a colour, no interpolation,
because interpolating between plate 3 and plate 4 is meaningless. Plate id, crust
kind, boundary kind, land cover later.

Greyscale becomes a two-point `Ramp` and stops being a special case.

### Fix the ramp's domain explicitly

A ramp needs to know what value maps to each end. The tempting thing is to
compute min and max from the grid each time. **Don't make that the default.**

Auto-normalising per image means every frame of a flipbook is scaled to its own
extremes, so a colour changes meaning between frames and a field that is slowly
warming looks perfectly static. You will misread it, and you will misread it
confidently.

Provide a helper that computes a domain from data, then **pin it for the run**
and pass it to every render. Auto-normalise only for one-off inspection of a
single image, and know that's what you're doing.

### Keep palette colours stable

Key a palette off the value — plate id — not off iteration order or first-seen
order. If colours shuffle between frames the flipbook is unreadable, and you will
spend an afternoon convinced the simulation is unstable when it is the renderer.

---

## Two draw modes

**`PixelPerCell`** is the workhorse. One cell, one pixel, shear ignored, exactly
as M0. A 1024 × 512 world is a 1024 × 512 PNG that opens instantly, and for
watching a plate simulation that is precisely what you want.

**`Hex { size_px }`** is for inspection. Real flat-top hexagons, correct shear,
optional cell borders.

Both, sharing all the mapping code. Hex mode cannot be the default for the
tectonic grid — at 8 px per hex a 1024 × 512 world is roughly 12,288 × 7,094
pixels, an 87-megapixel PNG — and pixel mode cannot show you what a ring actually
looks like. They are for different jobs.

### The two modes disagree, on purpose

[m0.md](m0.md) says to ignore the half-cell column shift at one pixel per cell
because the shear is invisible at map scale. That still stands. But hex mode must
**not** ignore it, or the hexagons won't tile.

So the same grid renders slightly differently in the two modes, by a shear that
grows across the image. That is expected and worth writing down where you'll find
it again: **pixel mode is a projection, hex mode is the truth.**

---

## Drawing hexagons: invert, don't rasterise

The obvious approach is to walk the cells and fill a polygon each. Don't.
Polygon filling needs a scanline rasteriser, and adjacent hexes either overlap on
their shared edge or leave a seam of unpainted pixels, depending on your rounding
— and which one you get varies around the hexagon.

**Instead, walk the output pixels and ask which cell each falls in:**

> pixel → fractional axial → cube round → offset → index → colour

Every pixel gets exactly one owner by construction, so gaps and overlaps are not
merely unlikely, they are unrepresentable. It works at any zoom, and it is a few
lines rather than a rasteriser.

This was prototyped before being written down. On a 33 × 33 grid at 14 px:

| | |
|---|---|
| cells receiving no pixels (gaps) | 0 of 1089 |
| pixels per interior cell | 506–508 |
| theoretical hexagon area at that size | 509.2 |

Interior cells land within half a percent of the exact area, which is as close as
a pixel grid can get.

### Two small additions to `hex.rs`

Neither is currently present — `hex.rs` today has `cube`, `from_oddq`, `to_oddq`,
`length`, `length_max`, `distance`, `neighbour` and `neighbours`. Both are
straight off Red Blob's page, which the module already follows.

**Flat-top layout**, hex centre to pixel: `x = size · 3/2 · q` and
`y = size · √3 · (r + q/2)`. Inverted: `q = (2/3 · x) / size` and
`r = (−1/3 · x + √3/3 · y) / size`.

**Cube rounding**, fractional axial to nearest cell: convert to cube, round all
three components, then correct the one that moved furthest by re-deriving it from
the other two so the constraint `x + y + z = 0` holds. Rounding each component
independently is the classic mistake here — it produces coordinates that aren't
valid cells, and the resulting picture has stray misplaced hexes rather than an
obvious failure.

Compute the image bounds from the actual cell centres plus half a hex, rather
than deriving a formula. It's less thinking and it's right for both parities.

### Borders come free

If a pixel's four-neighbour pixels don't all resolve to the same cell, paint it
the border colour. No geometry, no edge equations, and it works identically at
every zoom level. Verified in the prototype.

Borders are what make the hex mode worth having — without them adjacent cells of
similar colour merge and you're back to guessing.

---

## What this fixes

Render M0's concentric-ring grid in hex mode with borders on. This is the point
of the sub-milestone.

The M0 ring debugging was painful for exactly one reason: hexagons were being
read as square pixels, and a hand-count of a pixel dump went wrong by one row.
With hexagons drawn as hexagons, the answer is visible in a second — which is
what [m0.md](m0.md) claimed the smoke test would give you, and didn't quite,
through no fault of its own.

One thing to expect, so it doesn't read as a bug: **a ring of radius *n* is a
hexagon rotated 30° from the cells it's made of.** On a flat-top grid, rings come
out pointy-top — single cells due north and south, flat runs to east and west.
That is correct, and it's why the south vertex of the M0 ring was a lone cell.

---

## Tests

Rendering is checked by looking at it, so keep the assertions to the parts that
are genuinely mechanical.

- Cube rounding: every result satisfies `x + y + z = 0`. Round a cell's exact
  centre and get that cell back, for every cell in a small grid.
- Layout round-trip: hex → pixel → hex is the identity for every cell in a
  non-square grid, at several sizes. Non-square for the same reason as M0 — it
  catches transposition, and a square grid hides it.
- Ramp endpoints map to the endpoint colours, and a midpoint lands between them.
- A palette returns the same colour for the same index across separate
  constructions.
- Hex mode: every cell in the grid receives at least one pixel. This is the gap
  test, and it is cheap.
- Pixel mode still produces byte-identical output to M0 for the same input, so
  the golden-seed test is unaffected.

---

## Acceptance

M1b is done when:

1. The M0 ring grid renders in hex mode with visible borders, showing
   unambiguously hexagonal cells and clean concentric rings.
2. The same grid renders in pixel mode as it did at M0.
3. The M0 fBm field renders through a `Ramp` in colour.
4. A categorical `Palette` renders a small integer grid with stable, distinct
   colours across two separate runs.
5. A 1024 × 512 grid renders in pixel mode fast enough that you'd happily do it
   every step of a simulation.

Then start [M1c](m1c-layers.md).
