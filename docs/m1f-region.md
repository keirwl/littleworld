# M1f — Choosing a Region

**Goal:** pick a region-sized window out of the tectonic world, and resample its
structural fields to 1 km.

The last tectonic milestone, and the one that connects M1 to the rest of the
project. Everything from M2 onwards works inside the window this stage chooses.

Deliverables: `gen/region.rs` — window scoring, the crop, the upsample — and a
`--region` override.

---

## Scoring

Score every candidate window and take the best. This is a search, not a filter:
there is no such thing as a rejected window, only a lower-scoring one. That
property is what keeps [M1g](m1g-config.md)'s world-rejection machinery small,
because most of what a world could be "bad" at, this stage simply routes around.

What to score on:

- **Land fraction inside a target band.** All sea is useless, all land is dull.
  Somewhere around a half is a reasonable target, and the band matters more than
  the midpoint.
- **Coastline present.** A country with no sea loses harbours, salt marsh, and a
  natural boundary. Measure it as coast cells, not just "has both land and sea" —
  a single island in a corner shouldn't count.
- **Relief.** Thickness variance within the window, or a convergent boundary
  inside it or near enough that its range reaches in.
- **Natural boundaries on the edges**, per [plan.md](plan.md): sea, high ground or
  marsh at the window's rim. A country ending at an arbitrary straight line looks
  generated; one ending at a coast doesn't.
- **Distance from the top and bottom of the map.** The cylinder's poles are
  artifacts, not geography. Penalise proximity rather than hard-excluding, so a
  genuinely excellent window near the edge can still win.

Weight them, sum, take the maximum. Log the winner's score alongside the
runners-up — when a chosen region disappoints, the interesting question is
always what it beat.

`--region col,row` overrides the search entirely and takes exactly that window.
Keep it: it's how you re-examine a region you liked, and how you check that a
scoring change actually changed anything.

### Latitude comes free

The window's row on a cylindrical world *is* a latitude. Record it.

[plan.md](plan.md) says the region sits in one climate zone and needs a base
temperature, and until now that was going to be a bare random draw. Deriving it
from where the region actually sits is better motivated, costs nothing, and means
a northern world and a southern one differ for a reason. Settle how it feeds
climate at M5; just make sure the number is recorded now.

---

## Upsampling

The window is ~32 tectonic cells across and needs to become ~500 at 1 km. That
is a 16× resample, and how you do it matters more than it sounds.

**Upsample the parameters, not the elevation.**

Take crust kind, crust age, thickness, distance-to-boundary and boundary kind
across to 1 km, then **re-derive** elevation at fine resolution from those, using
the same isostasy and age–depth relations as [M1e](m1e-geology.md). Do not
upsample M1e's coarse elevation preview.

The reason is that distance fields interpolate beautifully and elevation fields
don't. A bilinear-interpolated distance-to-boundary is still smooth and still
monotone; a bilinear-interpolated elevation shows its blocks, and those blocks
survive fBm, survive erosion, and are still faintly visible in the finished map
three milestones later.

Bilinear should be enough given M2's fBm and M4's erosion follow. If ridge lines
still look blocky, the upgrade is to extract boundary polylines from the coarse
world and rasterise them at 1 km, so the sharp thing is sharp by construction —
but try the simple version first.

### Categorical layers don't interpolate

Crust kind and boundary kind are enums. Nearest-neighbour them, and expect
staircase edges at 16× — which is fine, because what consumes them is a distance
falloff, not the edge itself. Interpolating between "oceanic" and "continental"
produces a number that means nothing and will end up rendered as a colour that
implies it does.

### Take the margin from the world

The 30% margin from [plan.md](plan.md) is cut from the tectonic world along with
the rest, not fabricated afterwards. That is most of the point of having a world:
the catchments that feed rivers near the region's edge are real terrain, not
mirrored or faded-out edge padding.

---

## Gotchas

**The window may straddle the seam.** The tectonic world wraps in columns, so a
window near column 0 is legitimate and must read through the wrap rather than
clamping. The region itself does not wrap — it is a flat rectangle cut from a
cylinder, which is fine, because 500 km of a 16,000 km circumference is
essentially flat.

**Scoring is cheap; don't over-engineer the search.** Even at every-cell strides
the candidate count is small and each score is a pass over ~1,000 coarse cells.
Compute it directly before reaching for summed-area tables.

**Don't score on things the region can't keep.** It is tempting to score on
"looks nice", but M2's sea level percentile will move the coastline and M4's
erosion will reshape the relief. Score on structure — where the boundaries and
crust are — not on the preview's exact appearance.

---

## Tests

- The chosen window is entirely inside the grid vertically.
- `--region` returns exactly the window asked for, scoring untouched.
- A window straddling the seam contains the correct cells — construct one
  deliberately at column 0 and compare against the same window taken from a
  rotated copy of the world.
- Upsampled dimensions are right, and every fine cell maps back into the window.
- A constant coarse field upsamples to a constant fine field — catches
  interpolation that leaks at the edges.
- Nearest-neighbour on categorical layers produces only values that existed in
  the source. No invented enum variants.
- Elevation re-derived at 1 km from upsampled parameters agrees with the coarse
  preview when sampled at cell centres, to within interpolation error.

---

## Acceptance

M1f is done when:

1. Auto-selection picks a window containing coastline and relief, and you agree
   with its choice more often than not when you look at the alternatives.
2. `--region` overrides it.
3. The upsampled structural fields render at 1 km with no visible blockiness.
4. The window's latitude is recorded and logged.
5. A window straddling the wrap seam is handled correctly.
6. Same seed, byte-identical output.

Then start [M1g](m1g-config.md) and make it all reproducible — or go straight to
M2 if you'd rather see a coastline first. M1g is housekeeping and it will keep.
