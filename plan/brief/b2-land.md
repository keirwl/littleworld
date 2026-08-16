# B2 — Land

**Goal:** an island, or a coastline, that reads as land at a glance.

This is the milestone that turns noise into a place, and it is the first thing
worth showing anyone. Three independent tricks, each a few lines, each with a
disproportionate effect. One sitting.

---

## Sea level is a percentile, not a threshold

Copy the elevation values, sort them, and take the one at
`land_fraction × count`. That value is sea level.

Land fraction becomes a knob you turn, instead of an accident you discover. With a
fixed threshold every seed gives a different amount of land, so tuning feels random
rather than directed and you can never tell whether a change to the *shape* helped,
because the *quantity* moved at the same time.

This was promised at M2 in `plan/detailed`. It arrives here because nothing after
it works without it.

**Why a fixed threshold is especially wrong for fBm.** fBm is a sum of octaves at
decreasing amplitude, so by the usual argument its output piles up near zero rather
than spreading evenly across ±1. A threshold at 0.0 is near the densest part of the
distribution, where a tiny change in the constant moves a great deal of coastline;
a threshold at 0.5 is out in the tail, where most seeds give you almost no land at
all. The percentile is immune to all of this because it reads the distribution
instead of assuming one.

---

## The falloff mask

Subtract a falloff from the elevation: highest at the map edge, zero in the middle,
so the edges are pushed below sea level and the middle isn't touched. Use a
smoothstep rather than a linear ramp, or the interior comes out domed and every
island has its peak dead centre.

**A radial falloff makes a circle.** This is the gotcha, and it is the difference
between the milestone working and not. A circle with a rough edge is still a
circle, and it reads as generated instantly — nobody can say why, but everyone can
tell.

The fix is one line: **perturb the distance with a low-frequency noise before
applying the falloff.** Not the elevation, not the result — the distance itself.
The falloff is then still a clean smoothstep, but the contour it's measuring from
wanders, and you get bays and peninsulas at the scale of the perturbing noise
rather than at the scale of the fBm's detail. That distinction is most of what
separates a coastline from a blob with fjords.

**Make the mask a small enum**, not a single shape. Island (falloff from centre),
coast (falloff from one edge only, so land runs off the other side of the map),
archipelago (a couple of offset centres). It's a twenty-line function and it buys
visible variety on day one, which matters far more in this plan than it would in
the detailed one. The coast variant is also the one that best matches
`plan/detailed`'s "prefer natural boundaries" note — a country bounded by sea on
two sides and continuing off-map on the others.

### Order matters: mask first, then percentile

Do it in that order and the two knobs are independent — the mask decides **where**
land is, the percentile decides **how much**.

Do it the other way and they interact. Sea level would be fixed against the
unmasked field, then the mask would push cells below it, so you'd get less land
than you asked for by an amount that depends on the mask strength. Neither knob
then does what its name says.

**A consequence worth expecting**, because it will look like a bug the first time:
with the correct order, strengthening the mask does *not* give you less land. The
percentile compensates and hands back exactly the fraction you asked for, just
concentrated into a smaller, more compact shape. **Mask strength is a compactness
knob, not a quantity knob.**

---

## Redistribution

Normalise elevation to 0–1 and raise it to a power. Above 1, values shrink toward
zero: broad lowlands with a few peaks rising out of them.

Raw fBm gives uniform rolling texture at every point on the map, and that
uniformity is the single most recognisable "this is noise" tell — real terrain is
mostly flat with occasional mountains, not evenly bumpy everywhere. One line, and
it is the cheapest large improvement in the whole plan.

Do this before taking the percentile, since it changes the distribution the
percentile reads.

---

## Acceptance

1. The map reads as an island (or a coast) at a glance, without squinting.
2. `--land-fraction` visibly moves the coastline, and the shape stays recognisably
   the same seed while it does.
3. Two seeds give landmasses that differ in outline, not just in surface detail.
4. The coastline isn't a circle. If it is, the distance perturbation isn't working
   — check that it's perturbing the distance and not the elevation.

**Show someone.** This is the first image that earns it.
