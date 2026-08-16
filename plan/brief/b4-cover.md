# B4 — Land cover

**Goal:** the colour map. This is the artefact the whole plan exists to reach.

A `Cover` enum, a `match`, and a palette. The code is the easy part. One sitting,
of which most goes on the palette.

---

## The covers

Eight to twelve, all appropriate to a single temperate climate zone — you will not
have tundra and jungle in the same country. The list is already in
`plan/detailed`: oak wood, pine forest, heath, moor, fen, chalk downland, arable,
rough pasture, salt marsh, bare rock, alpine.

This is more medieval than a Whittaker diagram would be, and more useful later:
settlement scoring at M6 wants to know about arable and marsh, not about
"temperate broadleaf biome".

## The classifier

Order the `match` by hard overrides first, then fall through to the table:

1. **Water** — below sea level.
2. **Ice and bare rock** — above the snowline, then above the treeline.
3. **Wetland** — high humidity and near-zero slope. Fen inland, salt marsh at the
   coast.
4. **Everything else** — a table on temperature × humidity, modified by elevation.

Slope is needed here, so the hex gradient gets written in this milestone. [B5](b5-relief.md)
needs the same gradient for hillshading — write it once, here, and B5 becomes
almost free.

---

## Gotchas

**A threshold `match` produces hard-edged blobs.** Every boundary is a clean
contour line, because that's exactly what a threshold on a smooth field is. Real
land cover has ragged, interpenetrating edges — a wood thins into heath over a
kilometre rather than stopping.

The fix costs nothing: **perturb the classifier's inputs with a small
high-frequency noise before matching.** Add a little jitter to humidity and
temperature at each cell, then classify. Never perturb the output — swapping the
resulting cover at random gives you salt-and-pepper speckle, which looks like a
bug, whereas jittering the inputs moves the boundary and looks like a treeline.

This one change is most of the difference between "generated" and "map".

**The palette is the milestone.** Ten covers in one temperate zone are all
plausibly green. If adjacent covers differ only in hue and not in **lightness**,
the map is a single green smear no matter how good the simulation underneath is,
and you will conclude the classifier is broken when it isn't.

So: check the palette in greyscale. If two neighbouring covers vanish into each
other there, they'll be mush in colour too. And take the colours from an atlas or
an existing scheme rather than inventing eleven of them — this is the actual
"nice-looking" lever in the entire plan, and it is a colour problem, not a code
problem.

**Water is not one colour.** A flat blue sea next to a detailed coast looks unfinished
for very little reason. A shallow-to-deep ramp keyed off depth below sea level costs
one ramp you already have from B1, and it makes the coastline read as a coastline.

---

## Acceptance

1. The map is legible as a map: you can point at a region and say what it is.
2. Cover boundaries are ragged, not contour lines.
3. The palette survives being viewed in greyscale — adjacent covers stay distinct.
4. The distribution is plausible. Mostly arable and pasture in the lowlands, wood
   on the slopes, moor and rock high up. If it's 60 % fen, the humidity thresholds
   are wrong, not the classifier.
5. Two seeds produce maps that feel like different countries.

**Show someone who isn't you.** This is the one.
