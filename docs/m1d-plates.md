# M1d — Plates That Move

**Goal:** partition the world into plates, give them velocities, and watch them
move.

**No geology.** Crust kind is assigned at the start and thereafter merely travels.
Nothing thickens, nothing subducts, nothing ages. That is [M1e](m1e-geology.md).

The split is deliberate: motion and geology fail in completely different ways, and
debugging them together means never being sure which one is wrong. Get plates
sliding around a wrapping world convincingly first.

Deliverables: `gen/tectonics.rs` with a partition and a step loop, and a flipbook.

---

## Partitioning

Seed 6–12 points and grow plates outward from them.

**Not raw Voronoi.** A Voronoi partition's borders are perpendicular bisectors —
straight lines — and they look exactly like what they are. Two ways out, both
cheap:

- Grow by random adjacent-cell accretion: repeatedly pick a plate, pick an
  unclaimed cell adjacent to it, claim it. Organic, and the growth rates give you
  a size distribution for free.
- Keep Voronoi but perturb the distance metric with fBm. Plates stay compact —
  which accretion doesn't guarantee — and the borders wander. Slightly less code.

Either is fine. The second is easier to make deterministic and easier to tune.

**Assign crust independently of the partition.** The temptation is to declare each
plate oceanic or continental; resist it. Lay continental crust down as
fBm-thresholded blobs across the whole world, then partition on top. Plates that
are part continent and part ocean are where the interesting margins come from,
and a world of uniformly-continental plates has no coastlines worth cropping.

Target land fraction is a knob. Around a third is a reasonable start.

---

## Motion

Each plate gets a velocity. Real plates run 1–10 cm/yr, typically about 5 — call
it 50 km/Myr, or roughly three cells per million years at 16 km/cell. Don't take
that too seriously as a clock: count **steps**, tune by eye, and log the implied
Myr so the numbers stay honest.

### Move cells, not buffers

Keep everything in world-sized arrays. Each step, each plate translates rigidly by
an integer cell offset, so **every cell of a plate moves by the same delta** —
shape is preserved exactly, with no resampling and no interpolation. Scatter each
cell to its destination and you're done in one pass.

Two cells landing on the same destination is a collision. A destination nothing
lands on is a gap. Both are M1e's problem; see the placeholder rules below.

Per-plate local buffers are the other common architecture, and you'd want them if
you ever add plate *rotation* — real plates rotate, and it's a genuine visual
enrichment. It isn't worth it yet, and rigid translation on world arrays is much
harder to get wrong.

### The one gotcha that will bite you

**Sub-cell motion.** At 16 km/cell a typical plate crosses a cell every few
hundred thousand years, so per step it moves a fraction of a cell. If you move
plates by whole cells every step, the only speeds available are one cell per step
and two cells per step — everything is either identical or double, and the
boundaries alias into staircases that never smooth out.

Keep each plate's position as an **`f64` accumulator**. Each step, advance it by
the velocity, and move cells by `round(position) − round(previous position)` —
usually zero, sometimes one. Plate speed is then continuous even though the grid
isn't, and a plate moving at 0.3 cells per step genuinely moves slower than one
at 0.7.

This is the single most likely thing to be quietly wrong, because a world where
all plates move at the same speed still looks broadly plausible.

### Placeholder resolution, and don't polish it

Overlaps and gaps need *some* answer now so the flipbook is coherent, but the
real answers are M1e's. Keep them trivial and obviously provisional:

- **Overlap:** a deterministic winner — lowest plate id, say. Not physical, and
  not meant to be.
- **Gap:** assign to an adjacent plate and call the crust oceanic.

Deterministic matters more than sensible here. A tie broken by hash-map iteration
order will make the flipbook flicker and you'll think the motion is broken.

---

## Wrapping

The world wraps in columns, per [M1c](m1c-layers.md). A plate crossing the seam
should be *invisible* — no tear, no stretch, no duplicate.

This is the acceptance test for M1c as much as for M1d, and it's worth setting up
deliberately: give one plate a velocity that carries it right across the seam
during the run, and watch that frame.

---

## The smoke test worth more than the unit tests

**The flipbook.** Render the plate map every N steps into
`<out>/<seed>/steps/0000.png` and flick through them.

A plate simulation is close to impossible to debug from a single frame and close
to trivial to debug from an animation. A plate tearing, a boundary that has
stalled, two velocities that happen to cancel, a plate that is quietly losing
cells every step — all obvious in one viewing, all invisible in a still.

Use `PixelPerCell` from [M1b](m1b-render.md): the whole world in one screen is
the point, and 1024 × 512 frames are cheap enough to write every step if you want
them.

And the trap that ruins it: **key the palette off plate id.** Colours that shuffle
between frames make the animation unreadable and look exactly like instability in
the simulation.

---

## Tests

- Every cell belongs to exactly one plate; plate ids are contiguous; no plate is
  empty.
- After the partition, plate cell counts sum to the grid size.
- No cell is ever plateless once gap-filling has run — check every step, not just
  the last.
- Under pure translation with no gaps or overlaps in play, a plate's cell count is
  unchanged. Construct that case deliberately: one plate, empty world, let it run.
- The `f64` accumulator produces the right number of whole-cell moves over a run —
  a plate at 0.25 cells per step has moved 25 cells after 100 steps, not 0 and not
  100.
- Determinism: the same seed produces byte-identical frames.

---

## Acceptance

M1d is done when:

1. The plate map shows 6–12 plates with organic, non-straight borders.
2. The flipbook shows coherent motion over the full run — no tearing, no colour
   flicker, no plate quietly shrinking.
3. Plates move at visibly different speeds, and the slow ones are slow rather
   than stationary.
4. A plate crosses the wrap seam with no visible artifact.
5. Continental crust is scattered across plates rather than aligned to them, so
   there are margins to work with at M1e.
6. Same seed, byte-identical frames.

Then start [M1e](m1e-geology.md).
