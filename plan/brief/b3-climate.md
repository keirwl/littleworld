# B3 — Climate

**Goal:** a temperature field and a humidity field, plus the distance-to-sea map
that makes the humidity mean something.

Neither of these is worth showing on its own. They exist so that B4 has something
to classify. One sitting.

---

## Two fields need two seeds

The same seed sampled at the same coordinates gives the same field. Ask a noise
function for temperature and then for humidity without changing anything and you
get one field twice, and a land-cover map with only a diagonal band of variation
in it.

`for_stage("temperature")` and `for_stage("humidity")`. This is the moment
`RngMaster` — already written at M0 — pays for itself, and it is the only part of
rule 1 this plan keeps.

---

## Elevation has to be in metres now

The lapse rate is a number of degrees per kilometre. It cannot be applied to a
field that runs 0 to 1.

So B3 forces the decision: map normalised elevation to metres, sea level at 0 and
the maximum at something like 2000 m. One line. It's worth naming because it's the
point where the numbers stop being arbitrary and start being physical, and every
threshold from here on — snowline, treeline, the alpine cover — is quotable in
metres and checkable against somewhere real.

---

## Temperature

Base temperature, minus the lapse rate against elevation, plus a gentle
north–south tilt, plus a little noise so the contours aren't perfectly smooth.

**Elevation dominates by about 3.6×**, per the arithmetic in [plan.md](plan.md):
4.5° of latitude across the map is 2.7–4.5 °C, while 2000 m of relief at
6.5 °C/km is 13 °C.

Two things follow, and both are worth accepting rather than fighting:

- **Temperature will look like an inverted elevation map.** That is correct, not a
  bug, and no amount of tuning the latitude term will change it at 500 km.
- **Temperature's visible job is the snowline and the treeline**, and nothing
  else. Don't expect it to produce bands of biome across the map and don't tune it
  as though it might. The colour variety in B4 comes from humidity and elevation.

Keir's original sketch had temperature as its own noise field. That's a fine
stand-in and it will work, but the model above is barely more code and produces
snow on the mountains, which is a visible win for roughly nothing.

---

## Humidity, and why it must not be pure noise

An independent noise field puts desert next to marsh. The map comes out as
confetti — locally plausible everywhere and globally meaningless, which is
somehow worse-looking than a map with no humidity model at all, because the eye
reads the incoherence as noise rather than as climate.

Humidity needs a cause. The cheap one that looks right:

**Distance to sea, by multi-source BFS.** Seed a queue with every ocean cell at
zero and flood outward. `Grid::neighbours(idx)` already exists and already returns
indices, so this is a `VecDeque` and about fifteen lines. Then blend: mostly the
distance term, with some noise on top to break up the contours.

**Convert distance to humidity with a fixed length scale, not by normalising to
the map's maximum.** An exponential falloff over a length you choose in kilometres
is stable across seeds. Normalising by the largest distance found is not: an
archipelago and one compact continent would get the same humidity range despite
being completely different climates, exactly the failure the pinned ramp domain in
[B1](b1-colour.md) avoids. Same principle, different layer.

**Rain shadow is deliberately not here.** It's the obvious next term and it's the
first entry under climate in [next.md](next.md). When it arrives, carry
`plan/detailed`'s warning with it: flat-top hexes have no due-east or due-west
neighbour, so a westerly wind can't be walked along a row — it zigzags and leaves
a herringbone artifact in the rainfall. Advect along a true direction vector and
sample, or pick a prevailing wind along one of the six axes.

---

## Acceptance

1. Temperature and humidity render as tinted PNGs with pinned domains.
2. **The two fields are visibly different from each other.** If they aren't, the
   per-stage seeding isn't doing what you think.
3. Temperature is coldest on the peaks and slightly colder in the north.
4. Humidity is highest at the coast and falls inland, smoothly.
5. `distance_to_sea` renders as its own debug PNG and shows clean contours parallel
   to the coastline.

That last one matters more than it looks. The BFS is the piece here most likely to
be quietly wrong — a missed seed cell, a visited-check in the wrong place — and it
is instantly obvious by eye and nearly invisible any other way. Keep the renderer.
