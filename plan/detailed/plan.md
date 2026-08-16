# Medieval — Procedural World Generator

A deterministic, headless generator for a medieval fantasy landscape. Written in
Rust as a plain library plus a CLI. No game engine. Output is inspected as PNGs.

Scope: physical landscape (terrain, hydrology, climate, land cover) →
civilisations and settlements → generated languages and place names.
**History simulation is out of scope** — no wars, dynasties, or legends mode.

This is a learning project. Optimise for interesting algorithms and for
understanding what you built, not for shipping.

---

## Scale: a country, not a planet

The target is a single country-sized region — roughly **500 km across at 1 km
per cell**, so a 512×512 grid. For reference, medieval England is about 500 km
north to south. This is a deliberate change from planetary scale and it makes
the project both easier and better; see [Consequences of regional scale](#consequences-of-regional-scale).

**Amended at M1:** the *output* is still a single region, but it is now cut from a
simulated tectonic world rather than authored in isolation. See
[m1.md](m1.md) — the world exists to give the region a structural history, and is
never itself the product.

Two rules that follow from it:

- **Generate with a margin, then crop.** Simulate maybe 30% wider than the
  country you intend to show. Rivers near the edge otherwise have truncated
  catchments and come out the wrong size, and settlement placement gets
  edge-biased. The margin is simulated but never displayed.
- **Prefer natural boundaries.** Bias the large-scale structure so the region
  tends to be bounded by sea, mountain, or marsh. A country ending at an
  arbitrary straight line looks generated; one ending at a coast doesn't.

---

## Architecture: the four rules

These matter more than any individual algorithm.

**1. One master seed, derived per stage.**
Each stage gets its RNG from `hash(master_seed, "hydrology")`, not from one
shared stream. Otherwise adding a single random draw to stage 3 reshuffles
stages 4–8 and you can never tell whether a change improved anything. Highest
value decision in the project.

Extended at M1: where a stage can *reject* its own output and re-roll, the
attempt index joins the derivation — `hash(master_seed, "tectonics", 3)` — and
the accepted attempt must be recorded in the run's resolved config. Otherwise
tightening a constraint later silently changes what a seed means, which is the
exact failure this rule exists to prevent. See [m1.md](m1.md).

**2. Stages are pure functions over layers.**
Each stage reads earlier layers and returns new ones. Independently testable,
snapshot-able, and re-runnable without regenerating everything upstream.

**3. Every stage gets a debug renderer the day it's written.**
Non-negotiable. A broken flow-accumulation pass looks exactly like a working one
in a debugger. The PNG dump is the primary instrument, not a nicety.

**4. Struct-of-arrays, not array-of-structs.**
`elevation: Vec<f32>`, `temperature: Vec<f32>`, `cover: Vec<Cover>` — not
`Vec<Tile>` with fields. Cache-friendly for the full-grid passes that dominate,
and a new layer doesn't touch a god-struct. Settlements and cultures live in
their own `Vec`s, referenced by index.

### A fifth rule, specific to the hex question

**The pipeline works in flat `usize` indices. Only `hex.rs` knows what a
coordinate is.**

Every other module asks `hex.rs` for a cell's neighbours and gets back indices.
Flow routing, diffusion, flood fill and Dijkstra never see an axial coordinate.
This is what keeps the hex-versus-square decision genuinely reversible: swapping
lattice means rewriting one module and the few places that assume a neighbour
count. Let axial coordinates leak into the pipeline and that stops being true.

---

## Project shape

Single crate, library plus binary. Not a workspace.

```
src/
  lib.rs
  hex.rs            axial coords, neighbours, distance, ring, spiral, line,
                    Grid<T> (grid.rs was folded in here at M0), cylinder wrap
  rng.rs            master seed -> per-stage sub-seeds
  world.rs          TectonicWorld and Region: all layers + entity tables
  config.rs         every tunable, serde-loadable, dumped with each run
  gen/
    mod.rs          pipeline orchestration
    tectonics.rs    plate simulation on the coarse world grid
    region.rs       window scoring, crop, upsample to 1 km
    elevation.rs
    hydrology.rs
    climate.rs
    cover.rs        land cover / vegetation
    culture.rs
    settlement.rs
    roads.rs
  lang/mod.rs       phonology, lexicon, sound change, name composition
  render/png.rs     one debug renderer per layer
src/bin/worldgen.rs
config/default.toml
```

### Dependencies

| Crate | Why |
|---|---|
| `rand` + `rand_chacha` | **`ChaCha8Rng`, not `StdRng`.** `StdRng`'s algorithm isn't stability-guaranteed across `rand` versions; a dependency bump would silently invalidate every saved world. |
| `noise` | fBm/simplex. Worth rolling your own value noise later as an exercise; don't start there. |
| `image` | PNG output. |
| `serde` + `toml` | Config. RON at M1 for small structured data (settlements, cultures) if its exact enum round-tripping earns its place; TOML for anything hand-edited. |
| `bincode` or `postcard` | Large grid layers. A text format for a 512×512 `f32` field is unusably slow and huge. |
| `tracing` + `tracing-subscriber` | Added at M1: stderr plus a per-world log file, and free per-stage timing from spans. |
| ~~`clap`~~ `argh` | CLI args. `argh` chosen at M0. |
| `rayon` | Later, when erosion gets slow. Not on day one. |

Carry over from `../space`: the mold linker config and
`[profile.dev.package."*"] opt-level = 3`. Add `[profile.release] debug = true` —
you will profile erosion and will want symbols.

---

## Milestones

> **Superseded as the working plan.** The route below is the reference, not the
> schedule — its critical path to a colour map runs through tectonics, hydrology
> and erosion, which is a year of evenings. [`plan/brief/`](../brief/plan.md) is
> what's actually being followed: fBm → island → climate → land cover → relief,
> five milestones to a showable map, after which the layers below slot back in
> individually. This document is not deprecated — `plan/brief/next.md` points into
> it for every stage it defers, and the specs here are what get followed when one
> of those stages is picked up.

Each ends in a PNG. Don't start the next until the current one looks right.

| | Deliverable | Why it matters |
|---|---|---|
| **M0** | Hex coords, `Grid<T>`, seeded RNG, PNG of an fBm field, golden-seed test | Proves the instrument. See [m0.md](m0.md). |
| **M1** | Plate tectonics on a coarse wrapping world; region selection; renderer, logging and config. Split into [a](m1a-logging.md) logging, [b](m1b-render.md) renderer, [c](m1c-layers.md) layers and wrapping, [d](m1d-plates.md) plates, [e](m1e-geology.md) geology, [f](m1f-region.md) region, [g](m1g-config.md) config | Gives the region a structural history. See [m1.md](m1.md). |
| **M2** | Structure → elevation → sea level, at 1 km | First recognisable landmass |
| **M3** | Hydrology: depression fill, flow direction, accumulation, rivers, lakes | **Where it stops looking like noise** |
| **M4** | Erosion: stream-power incision, a few hundred iterations | Where it starts looking like terrain |
| **M5** | Climate + land cover | First colour map worth showing anyone |
| **M6** | Cultures, settlements, demographics | Where it becomes a *place* |
| **M7** | Roads, bridges, fords | Where the places connect |
| **M8** | Language, place names, a substrate layer | Where it feels inhabited |
| **M9** | *(optional)* Viewer — Bevy, Godot via `gdext`, or `egui` | Decide when you get here |

Erosion is new relative to the original sketch, and it's promoted to its own
milestone because at 1 km cells it stops being a texture and becomes the
dominant visual feature.

Tectonics is newer still. The original plan authored the large-scale structure
outright; M1 simulates it instead, which inserts a whole second resolution
upstream of everything else. That is a real cost, taken deliberately — see
[m1.md](m1.md) for the argument and the price.

---

## The pipeline

### M1 — Tectonics and region selection (`gen/tectonics.rs`, `gen/region.rs`)

Fully specified in [m1.md](m1.md); the summary is that the earlier conclusion
here was wrong.

That conclusion was: at regional scale the map sits inside one plate, so there is
nothing to simulate — author three to five structural features instead. The
premise is correct and the conclusion doesn't follow. The right response to "the
region is too small to simulate" is not to author the region, it's to **simulate
at a scale where simulation means something and then crop.**

So M1 runs a plate simulation on a coarse, column-wrapping world — a cylinder,
not a sphere, so plate motion stays 2-D translation and `hex.rs` is reused
unchanged but for a `rem_euclid`. Roughly 1024 × 512 cells at 16 km, 6–12 plates,
a few hundred steps of motion, collision and rifting. It produces crust kind,
crust age and crust thickness, from which elevation follows by Airy isostasy and
age–depth rather than being invented.

Then it scores windows and crops one region out, sized so it spans 20–60 coarse
cells — enough that the tectonic setting varies across it, few enough that fBm
and erosion still do the detail work.

The honest cost: an extra resolution level in the pipeline, and an upstream stage
whose retuning invalidates every downstream reference world. The gain is that the
mountains have a reason to be where they are.

### M2 — Elevation and sea level (`elevation.rs`)

Take the region's upsampled structural fields — crust kind, age, thickness,
distance to boundary — and re-derive elevation at 1 km, rather than upsampling a
coarse elevation field, which would show its blocks. Blend with a few octaves of
fBm for detail, and pick sea level as a percentile of the elevation histogram so
land fraction is a tunable rather than an accident.

### M3 — Hydrology (`hydrology.rs`)

The stage most projects fake, and the one everything downstream needs.

1. **Depression filling** — priority-flood (Barnes et al. 2013). A binary heap
   seeded from the map edge, popping lowest-first, raising each cell to at least
   its parent's level. Guarantees water can always reach an outlet.
2. **Flow direction** — steepest descent to one of six neighbours.
3. **Flow accumulation** — process cells in *descending* elevation order,
   pushing each cell's water (1 + everything upstream) downstream. Single pass, O(n).
4. Accumulation above a threshold is a **river**; the threshold tunes river
   density. Filled sinks that never drained are **lakes**. Walking flow
   directions upstream from a mouth gives **watersheds** free.

Keep accumulation as a real number, not a boolean — river *width* and mill
placement both key off it later.

### M4 — Erosion (`hydrology.rs` or its own module)

Stream-power incision: erosion rate proportional to `drainage_area^m * slope^n`,
with `m ≈ 0.5`, `n ≈ 1` as a starting point. Iterate: compute flow, incise,
recompute flow, repeat a few hundred times. Add hillslope diffusion (smooth
elevation slightly toward neighbours) so ridges round off and don't stay knife-edged.

This is what produces **dendritic drainage networks**, the branching tree
pattern of real river systems, and it's the single strongest cue that terrain is
real rather than noise. It's also the most expensive stage — the first place
`rayon` earns its place, and the reason for `debug = true` in release.

Expect to spend real time tuning here. That's the point.

### M5 — Climate and land cover (`climate.rs`, `cover.rs`)

At this scale the whole region is in one climate zone, so the planetary latitude
machinery disappears. What remains:

- **Temperature**: one base value for the region, minus an elevation lapse rate
  (~6.5 °C per 1000 m), plus a mild north–south gradient.
- **Continentality**: distance from the sea widens the seasonal range.
- **Precipitation**: pick a prevailing wind direction. Advect moisture from the
  sea across the map, dropping rain in proportion to *upward* elevation change
  and replenishing over water. Rain shadows behind mountains are emergent.

  Note the lattice interacts here. Flat-top hexes have no due east or west
  neighbour, so a westerly wind can't be walked along a row — it zigzags NE/SE
  and leaves a herringbone artifact in the rainfall field. Two clean outs: pick
  the prevailing wind along one of the six axes, or advect along a true
  direction vector and sample, rather than hopping cell to cell. The second is
  better and not much harder.

Orographic rainfall is now the *only* source of precipitation variation, so this
model has to be decent even though it's simpler than the planetary version.

**Land cover, not biomes.** You won't have tundra and jungle in the same
country. Use 8–12 covers appropriate to one climate: oak wood, pine forest,
heath, moor, fen, chalk downland, arable, rough pasture, salt marsh, bare rock,
alpine. A `match` on temperature/moisture/elevation/river-adjacency. This is far
more medieval than a Whittaker diagram, and more useful to settlement scoring.

### M6 — Cultures and settlements (`culture.rs`, `settlement.rs`)

Regional scale is where this stage gets *good*.

**Cultures**: one to three, not dozens. Seed at high-habitability points,
flood-fill outward with Dijkstra where movement cost is terrain difficulty.
Borders end up hugging ridgelines and rivers, which is what real ones do.

**Settlements**: score every land cell for fresh water, river confluence,
coastal harbour, arable cover, defensibility, and river-crossing potential.
Then place a *hierarchy*, largest first, each tier with its own minimum spacing:

- **Villages** every few km on arable land near water — the dense base layer.
- **Market towns** roughly a day's round trip apart. Central Place Theory puts
  this near 10–15 km, and Christaller's market catchments are literally
  hexagonal — a pleasing accident given the lattice.
- **A few towns**, and likely **one city**.
- **Castles** at chokepoints: passes, fords, river bends, coastal approaches.
- **Mills** on river cells above a flow-accumulation threshold.

*Medieval Demographics Made Easy* (`resources/`) is a **realm**-scale system, so
it now fits the project natively rather than being stretched over a planet. Note
the direction problem: Ross distributes a top-down realm total, but you're
placing sites bottom-up from terrain. Expect to invert it — derive realm
population from the sites and their land quality, and use Ross's ratios as a
*check* on the resulting hierarchy rather than as the generator.

Port the `d(s)` and `dn(n, d)` dice helpers from the old `src/main.rs`, but
taking `&mut ChaCha8Rng` instead of calling `rand::rng()`.

### M7 — Roads (`roads.rs`)

Dijkstra between settlement pairs with terrain movement cost. **Give existing
road segments a large cost discount** so later routes prefer joining earlier
ones — trunk roads and networks then emerge instead of a naive star of
point-to-point lines.

At 1 km cells this produces genuinely medieval structure: everything funnels to
the few viable river crossings, and those crossings are exactly where the
castles and market towns from M6 already want to be. Where a road meets a river,
emit a **ford** on low flow and a **bridge** on high.

### M8 — Language and place names (`lang/`)

Cheapest stage, largest payoff for the world feeling inhabited.

Per culture: a phoneme inventory, a syllable structure with probabilities, and
phonotactic constraints. From those, a lexicon of morphemes bound to meanings —
*river*, *hill*, *ford*, *stone*, *dark*, *king*. Place names compose from an
actual mapped feature plus morphemes, so the settlement at a crossing genuinely
carries the culture's word for "ford".

For related cultures, generate one proto-language and apply ordered sound-change
rules to derive daughters. Related peoples then have visibly related names.

**The substrate layer.** Regional scale unlocks the best trick available here.
Generate an *earlier* people's language, name the major rivers and hills with
it, then let the current culture name the settlements. Big natural features keep
the old names; human settlements get the new ones. This is exactly what happened
in Britain — Celtic river names (Avon, Ouse, Thames) under English settlement
names — and it implies a deep past without simulating a minute of it. It only
works at regional scale, where one people plausibly displaced another.

---

## Consequences of regional scale

Recorded because they justify several choices above.

| | Planetary | Regional (this project) |
|---|---|---|
| Tectonics | Simulate plates | ~~Author 3–5 structural features.~~ **Reversed at M1: simulate plates on a coarse wrapping world, then crop.** The premise below was right — nothing interacts inside one plate — but the conclusion was wrong. Simulate at a scale where it means something. |
| Erosion | A texture at 10–50 km/cell | **The dominant visual feature** at 1 km/cell. Promoted to its own milestone. |
| Climate | Latitude bands, trade winds, westerlies | **One climate zone.** Base temperature + lapse rate + orographic rain. Smaller code, but the orographic part must be better. |
| Biomes | Full Whittaker range | **8–12 land covers** within one zone. More medieval, more useful. |
| Settlements | Abstract cities on a continent | **The full medieval texture** — villages, mills, fords, castles, market towns. |
| Demographics | Ross's system stretched thin | **Fits natively.** It was always a realm-scale system. |
| Roads | Abstract long-distance links | **Real local networks** funnelling to river crossings. |
| Names | One culture per region | **Substrate layer** — older names on rivers and hills, newer on settlements. |
| Edges | Wraps, or ends in ocean | **A real problem.** Solved by a simulated margin plus natural boundaries. |

Net effect: strictly less simulation machinery, and a much richer, more
characterfully medieval output. This was the right call.

**It did not change M0's code — only its parameters.** M1 is a different matter:
the tectonics reversal above adds a whole stage and a second resolution. Every
other row here still stands.

---

## Verification

**Primary loop.** `cargo run --release -- --seed 42 --out out/` writes a
directory of PNGs, one per layer, plus the resolved config and the world binary.
A `--stage hydrology` flag stops early so you aren't waiting on downstream
stages while tuning erosion.

**Golden-seed determinism test.** Generate a small world, hash the layers,
assert against a stored digest. Catches the failure that will actually hurt:
a dependency bump or a stray `rand::rng()` silently making saved worlds
unreproducible. **Write this at M0**, while there is one layer to hash — it is
annoying to retrofit at M5.

**Unit tests over pure functions.** Hex maths against Red Blob's known values;
depression filling on a hand-built grid with one pit; flow accumulation on a
known slope where the outlet total must equal the cell count; the dice helpers;
serialisation round-trips.

**Visual regression.** Keep a handful of reference seeds and re-render after
every stage change. Worldgen bugs are far more often visible than assertable.

---

## Open questions

- **Exact cell size.** 1 km is the working assumption. 500 m at 1024² covers the
  same ground with four times the cells and resolves individual valleys — still
  fast in Rust, worth trying once M4 works. Settle at M6, when settlement
  spacing makes it concrete.
- **Whether hex survives.** Kept for now. Rule five above is what makes dropping
  it cheap later. Revisit after M4, when you know whether the six-neighbour
  flow routing actually looked better than eight would have.
- **How many cultures.** One makes the language work sharper; three makes the
  borders interesting. Decide at M6.
- **How long to run the tectonic sim.** Long enough for continents to form,
  short enough that they haven't all collided into one supercontinent. Tune by
  eye against the flipbook; if there's no window that gives both, the sim needs
  periodic rifting rather than more steps.
- **Which constraints, if any, justify rejecting a world.** Decide after
  generating twenty and looking at them, not before — see [m1.md](m1.md).
- **Whether the region's latitude drives climate.** A cylindrical world gives the
  chosen window a latitude for free, which is a better source for base
  temperature than a bare random draw. Settle at M5.
