# B5 — Relief

**Goal:** the same map as B4, shaded, so that it looks like terrain rather than a
classification.

Estimate the surface normal from the six-neighbour gradient, take its dot product
with a light direction, and multiply the result into the cover colour. The gradient
already exists from [B4](b4-cover.md), so this is perhaps thirty lines.

**The highest visual payoff per line in the project.** One sitting, and probably
less.

---

## The gradient

Sum, over the six neighbours, the elevation difference times the unit vector
pointing at that neighbour in pixel space. The six flat-top directions and their
pixel layout are already pinned in `hex.rs` and in `m0.md`; use them rather than
re-deriving, and remember that image *y* increases downward.

---

## Gotchas

**Vertical exaggeration is mandatory, not a refinement.** This is the one that will
cost you an evening if it isn't said out loud.

At 1 km cells the true slopes are gentle. Measured, with a light at 45° altitude:

| Drop across one cell | Exaggeration | Slope | Illumination swing |
|---|---|---|---|
| 20 m | 1× | 1.15° | **3.9 %** |
| 20 m | 3× | 3.43° | 11.3 % |
| 20 m | 5× | 5.71° | 18.2 % |
| 50 m | 1× | 2.86° | 9.5 % |
| 50 m | 5× | 14.0° | **40.0 %** |

A 3.9 % swing is invisible. So unexaggerated hillshading doesn't look subtle, it
looks *broken* — and the natural response is to go hunting for a bug in the
gradient that isn't there. Multiply elevation by 3–8× before taking the gradient
and it appears immediately.

**Light from the north-west.** Light it from the south-east and the terrain inverts
perceptually: ridges read as valleys and valleys as ridges. This is a genuine and
well-documented illusion, not a matter of taste, and it is unmistakable once seen.
North-west at about 45° altitude is the cartographic convention for exactly this
reason.

**Multiply and clamp.** Clamp the shade factor to roughly 0.55–1.25 rather than
letting it run 0–1. Unclamped, the shadowed faces go to black and the map loses
every colour B4 was spent on — you get a relief render with a hint of green, when
what you want is a land-cover map with relief in it.

**Don't shade the sea.** Flat colour, or the depth ramp from B4. Shading the noise
under the water looks like a bug, because it is one — that's residual fBm below sea
level, and it has no business being visible.

---

## Optional, if the sitting is going well

**A coastline stroke.** One or two pixels of a darker line where a land cell has an
ocean neighbour. It costs a neighbour check and it does a surprising amount of
work: it separates land from sea crisply and reads as deliberate cartography rather
than as a rendering artefact.

---

## Acceptance

1. Ridges and valleys are visible as three-dimensional form.
2. Ridges read as ridges. If the terrain looks inside-out, the light is coming from
   the wrong side.
3. The land-cover colours from B4 are still clearly readable through the shading.
4. The sea is flat.

This is the image you show people. It's also the point at which
[next.md](next.md) becomes the plan — everything after this is an independent
upgrade to a map that already works.
