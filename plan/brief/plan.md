# The Brief Plan — a map worth showing, in five sittings

`plan/detailed/` is a good plan for a project with unlimited runway. This is not
that project.

Its critical path to a colour map runs through a plate simulation, a region crop,
hydrology and erosion. Its own table admits M5 is the "first colour map worth
showing anyone", and M1 alone is seven documents. That is a plan you finish in a
year or abandon in a month.

This plan inverts the priority: **get to a map worth showing as fast as possible,
then make it correct in small independent pieces.** Reproducibility, logging,
config and serialisation are demoted from load-bearing to optional. They are good
things. They are not the reason to open the editor tonight.

The route: fBm elevation → a falloff mask that makes it an island → separate
fields for temperature and humidity → land cover → colour → shaded relief.
Roughly, skip to `plan/detailed`'s M5. Afterwards, the detailed plan's layers slot
back in either **upstream** (real hydrology replacing the humidity noise) or
**downstream** (cultures, roads, names). See [next.md](next.md).

`plan/detailed/` is not deleted or edited. It stays as the reference this plan's
menu points back into, and every deferral below names the document it defers to.

---

## Scale

**~500 km across at 1 km per cell — 512 × 512.** Identical to `plan/detailed`, so
M2–M8 slot in later with nothing to rescale.

One consequence has to be stated up front because it shapes B3 and B4:

500 km north–south is 4.5° of latitude. At 0.6–1.0 °C per degree that is a
**2.7–4.5 °C** span across the whole map. The lapse rate over 2000 m of elevation
is **13 °C**. Elevation beats latitude by about **3.6×**.

So temperature at this scale is an inverted elevation map with a gentle tilt. Its
only visible jobs are the snowline and the treeline. It will not produce climate
bands and should not be tuned as though it might. **The map's colour variety comes
from humidity and elevation.**

---

## The milestones

Each ends in a PNG you open. Don't start the next until the current looks right.

| | Deliverable | Ends in |
|---|---|---|
| **[B1](b1-colour.md)** | Colour: ramps, palettes, RGB output, a per-run directory | The existing fBm field, hypsometrically tinted |
| **[B2](b2-land.md)** | Land: falloff mask, percentile sea level, redistribution | **An island that reads as an island** |
| **[B3](b3-climate.md)** | Climate: temperature, humidity, distance to sea | Two tinted fields and a debug distance map |
| **[B4](b4-cover.md)** | Land cover: the classifier and its palette | **The colour map** |
| **[B5](b5-relief.md)** | Relief: hex gradient, hillshade, composite | **The image you show people** |

Then [next.md](next.md) — the menu of independent upgrades.

**The first showable thing is B2 — two sittings in.** B1 is the instrument, not
the artefact; it exists so that B2 can be judged by eye.

---

## The one architectural commitment

Every layer is a `Grid<T>` produced by a free function that takes the layers it
needs and returns a new one. No trait. No registry. No pipeline struct.

That is the entire mechanism that makes "iterate in small pieces later" true.
Swapping `humidity_from_noise` for `humidity_from_rainfall` becomes a one-line
change at one call site, because the two functions have the same shape and neither
knows anything about the other.

Keep it that small. If it starts wanting a trait, that is the signal a genuine
second implementation has arrived — build the abstraction then, with two instances
in front of you, not now with one imagined.

### What survives from the detailed plan's four rules

- **Rule 2 — stages are pure functions over layers.** This *is* the commitment
  above. It's the load-bearing one.
- **Rule 3 — every stage gets its debug renderer the day it's written.** Also
  load-bearing, and more so here than in the detailed plan: the whole brief path is
  navigated by looking at PNGs, so B1 comes first for a reason.
- **Rule 4 — struct-of-arrays.** Free. `Grid<T>` already does it.
- **Rule 5 — only `hex.rs` knows what a coordinate is.** Free, already true, and
  it's what keeps the hex-versus-square question cheap to revisit.

**Rule 1 — one master seed, derived per stage** — survives only in its free half.
`RngMaster::for_stage` is already written, costs nothing to call, and B3 needs it
anyway to stop temperature and humidity coming out as the same field. The
attempt-index and re-roll extension does not survive; it goes to
[m1g-config.md](../detailed/m1g-config.md).

---

## Leave the golden test where it is

`main.rs`'s `golden_hash` pins the **noise primitive** — `RngMaster` plus fBm plus
`Grid` — and not the pipeline's output. That is exactly right and it should stay
there.

It keeps catching the failure that actually costs you something: a dependency bump
silently changing what a seed means. And it will never break because you nudged a
threshold, because it doesn't know the pipeline exists.

**Do not extend it to hash the final map.** That turns every tuning session into a
re-blessing session, and tuning sessions are the whole point of B2 through B5.
Visual regression here means keeping a few seeds and looking at them.

---

## Deferred, deliberately

Not "forgotten" — each of these is specced somewhere in `plan/detailed/` and each
has a trigger that would make it worth doing.

| Deferred | Spec | What would make it worth doing |
|---|---|---|
| Logging build-out | [m1a](../detailed/m1a-logging.md) | Keep whatever `fmt()` subscriber is already in `main.rs`; don't extend it. A stage slow enough that you want per-step timing. |
| Config file | [m1g](../detailed/m1g-config.md) | More than about eight CLI flags, or wanting to keep several tunings side by side. |
| Serialisation | [m1g](../detailed/m1g-config.md) | A stage slow enough that re-running upstream to tune downstream hurts. Erosion will do it. |
| Cylinder wrapping | [m1c](../detailed/m1c-layers.md) | Only tectonics needs it, and tectonics is last. |
| Hex-mode renderer | [m1b](../detailed/m1b-render.md) | Debugging something where you need to see individual cells — or the hex-tile restyle. |
| 30 % simulated margin | [plan.md](../detailed/plan.md) | Rivers. It costs simulated area for a benefit that's invisible until catchments exist. |
| Re-roll on rejection | [m1g](../detailed/m1g-config.md) | Having generated twenty worlds and formed an opinion about which are bad. |

---

## Discipline

The same rule as `m0.md` — each milestone ends in a PNG, don't start the next
until the current one looks right — plus one specific to this plan:

**If a milestone takes more than one sitting, it was too big. Split it.** The
milestones below are sized so that isn't necessary, and if one turns out to be,
that's information about the milestone and not about you.

### Re-entry cost is the thing that actually kills this

A hyperfixation project doesn't die during the fixation. It dies during the first
gap, when picking it up again means remembering where you were. Three things keep
that cheap, and all three are cheap to maintain:

- **Milestone documents stay one page.** The length limit is a feature. If a
  document needs more, the milestone needs splitting.
- **One command writes every layer's PNG, every run.** Not a flag, not a
  subcommand — every run. Re-entry is then `cargo run --release` and opening a
  directory, and the state of the project is a folder of images rather than
  something you have to reconstruct.
- **The output directory is per-run** (B1). So the last five runs are still on
  disk to compare against.
