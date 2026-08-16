# After B5 — the menu

Once B5 is done there is a working map, and every entry below is an independent
upgrade to it. **There is no order you have to follow.** Pick whichever is
interesting on the night; that property is the whole reason the brief plan was
worth writing.

They divide the way the plan was framed: things that go **before** — replacing an
input, with nothing downstream needing to change — and things that go **after** —
adding a layer on top of what's there.

The "payoff" column is visible improvement per hour spent, judged honestly. It is
not a measure of how interesting the problem is, and for at least one entry the two
point in opposite directions.

---

## Upstream — replace an input

The architectural commitment in [plan.md](plan.md) is what makes these cheap: each
is a function with the same shape as the one it replaces, swapped at one call site.

| | Replaces | Spec | Payoff |
|---|---|---|---|
| **1. Rivers and lakes** | nothing — inserts between B2 and B3 | [M3](../detailed/plan.md) | **Highest** |
| **2. Erosion** | the fBm's uniform roughness | [M4](../detailed/plan.md) | High, slow |
| **3. Orographic rain** | the distance-to-sea humidity | [M5](../detailed/plan.md) | Moderate |
| **4. Tectonics** | the falloff mask | [m1d](../detailed/m1d-plates.md)–[m1f](../detailed/m1f-region.md) | Low per hour |

**1. Rivers and lakes.** Recommended first by a wide margin. Priority-flood
depression filling, steepest-descent flow direction, then flow accumulation in
descending elevation order. `plan/detailed` calls this "where it stops looking like
noise" and that is not overstated — nothing else on this list changes the map as
much.

It also pays twice. Rivers give [B3](b3-climate.md)'s humidity a real inland source
instead of a distance heuristic, and [B4](b4-cover.md)'s classifier a
river-adjacency term, so fen and salt marsh stop being purely humidity-driven. Keep
accumulation as a real number rather than a boolean — river width and mill
placement both key off it later.

**2. Erosion.** Stream-power incision, a few hundred iterations. Produces dendritic
drainage — the branching tree pattern of real river systems, and the strongest
single cue that terrain is real rather than generated. Needs rivers first. It is
the most expensive stage in the project and the one that most rewards tuning time,
so it's a good choice for a long session and a bad one for a short one.

**3. Orographic rain.** Advect moisture inland from the sea, dropping it on upslope.
Rain shadows behind mountains become emergent rather than absent. Carry the flat-top
warning from [B3](b3-climate.md) — no due-east neighbour, so advect along a true
vector rather than hopping cell to cell.

**4. Tectonics.** Seven documents of plate simulation, collision tables, isostasy
and region scoring, to replace one falloff function. Listed last deliberately: it is
by far the largest item here and the one whose result a viewer of the map would be
least able to point to.

That is not an argument against doing it. It is the most interesting problem on the
list and it is the reason the detailed plan exists. It is an argument against doing
it *for the map*, and for being honest with yourself about which motive is
operating when you start.

---

## Downstream — add a layer

| | What it adds | Spec | Payoff |
|---|---|---|---|
| **Settlements and cultures** | villages, towns, castles, mills; culture borders | [M6](../detailed/plan.md) | High |
| **Roads, bridges, fords** | the network between them | [M7](../detailed/plan.md) | High |
| **Languages and place names** | names on the map | [M8](../detailed/plan.md) | **Highest per hour** |
| **Hex-tile restyle** | the tiles in `resources/` instead of flat colour | — | High, unknown cost |

**Place names are the cheapest large payoff in the project.** A map with names on it
reads as a place; the same map without them reads as output. The substrate trick in
M8 — an older people's language on the rivers and hills, the current one on the
settlements — implies a deep past without simulating any of it, and it only works at
this scale.

Settlements want rivers first: confluences, fords and harbours are most of what
makes a placement score interesting, and none of them exist yet.

**The hex-tile restyle** uses the tilesheets already in `resources/`. Wanted, and
deliberately not specced here — it needs the hex-mode renderer from
[m1b](../detailed/m1b-render.md) rather than the pixel-per-cell one, and the sensible
time to work out what it costs is when the land cover has settled down. Note that it
is a *restyle*, not a replacement: keep the naturalistic renderer, because it is also
the debugging instrument for everything above.

---

## Infrastructure — when it starts hurting

Nothing here improves the map. Each entry has a trigger; do it when the trigger
fires and not before.

| | Spec | Trigger |
|---|---|---|
| Serialisation | [m1g](../detailed/m1g-config.md) | Re-running erosion to tune land cover becomes annoying. It will. |
| Config file | [m1g](../detailed/m1g-config.md) | More than about eight CLI flags, or wanting two tunings side by side. |
| Logging build-out | [m1a](../detailed/m1a-logging.md) | Wanting per-stage timings, or a run log to read after the fact. |
| Hex-mode renderer | [m1b](../detailed/m1b-render.md) | Debugging something cell-by-cell, or the tile restyle. |
| Cylinder wrapping | [m1c](../detailed/m1c-layers.md) | Tectonics, and nothing else. |
| Re-roll on rejection | [m1g](../detailed/m1g-config.md) | Having generated twenty worlds and formed an opinion about which are bad. |

Serialisation is the one most likely to fire first, and it fires the moment erosion
lands. Everything upstream of a slow stage becomes worth caching the day that stage
exists.
