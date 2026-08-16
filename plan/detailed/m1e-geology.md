# M1e — Geology

**Goal:** replace M1d's placeholder collision rules with real ones, and give crust
an age and a thickness.

This is where the plate map stops being a Voronoi diagram in motion and starts
producing mountains, trenches, island arcs and ocean basins — and where the
output first looks like a world.

Deliverables: collision resolution, ridge crust, ageing, thickness, and a coarse
elevation preview.

---

## What crust carries

Two new layers beyond M1d's plate id and crust kind:

**Age**, in steps or Myr, meaningful for oceanic crust. Reset to zero when crust
is created at a ridge; incremented every step thereafter.

**Thickness**, in km. Continental crust starts around 35 km, oceanic around 7.
Collisions add to it; nothing much takes it away until erosion at M4.

Those two, plus kind, are enough to derive elevation. Everything else is
decoration.

---

## Collision resolution

Where two plates claim a cell:

| | Result |
|---|---|
| continental + continental | Both thicken. **Nothing subducts** — continental crust is too buoyant. This is orogeny, and it is where your mountains come from. |
| continental + oceanic | The oceanic crust subducts and is destroyed. The continental margin thickens in a band a few cells inland — a volcanic arc — and a trench forms seaward. |
| oceanic + oceanic | The older, colder, denser one subducts. The survivor gains an island arc. |

Where no plate claims a cell, new oceanic crust forms at age zero and minimum
thickness: a spreading ridge.

The asymmetry in the middle row is the important one. Ocean–continent convergence
produces a *coastal* range with a trench offshore — the Andes — while
continent–continent produces an *interior* range with no trench — the Himalayas.
Getting both means your region can be either kind of place, which is worth more
than either alone.

### Rifting, and the supercontinent problem

With fixed velocities and enough steps, plates converge and never separate. You
get one blob. That is real — Pangaea happened — but it is not a useful output,
and it is the most likely way for a run to end up boring.

Either stop at a few hundred steps, or periodically **rift**: pick a large
continental plate, split it along a line, and give the halves diverging
velocities. Rifting is also the only way to get a young, narrow ocean between two
matching coastlines, which is a good thing for a region to sit beside.

---

## Elevation is a consequence, not a free variable

Elevation proper is M2, but the fields above are meaningless until you know what
they become, and rule three wants a debug renderer today. Two real relations do
the work, and both are nearly free.

**Airy isostasy.** Crust floats on mantle, so surface height follows thickness:
height above datum is proportional to `thickness × (1 − ρ_crust / ρ_mantle)`.
With continental crust around 35 km at density 2.7, oceanic around 7 km at 3.0,
and mantle at 3.3, that gives roughly 6.4 km and 0.6 km — **about 5.7 km of
continent-over-ocean relief with no tuning at all**, which is close to the real
figure. Better still, crust thickened by collision becomes mountainous
automatically, because that is genuinely why mountains are high.

**Age–depth.** Oceanic crust cools and sinks as it ages:
`depth ≈ 2500 + 350·√(age in Myr)` metres, from half-space cooling. Good to about
80 Myr, after which it flattens. Ridges come out shallow and abyssal plains deep,
for free.

Between them, the simulation's output is already a plausible elevation field
before M2 adds a single octave of noise. Render it through a diverging
bathymetry/topography ramp from [M1b](m1b-render.md) — that render is how you
will judge every tuning change you make here.

---

## Deriving boundaries

Once the run finishes, extract what the region stage will actually consume:
**distance to the nearest boundary**, and **boundary kind** — convergent,
divergent, transform — from the relative velocities of the plates either side.

Do this at the end, not per step. It is a couple of flood fills over the finished
world, and it is the form [M1f](m1f-region.md) upsamples.

---

## Gotchas

**Conservation is your best assertion.** Oceanic crust is created at ridges and
destroyed at trenches, so its area fluctuates. **Continental crust is essentially
never destroyed** — it is too buoyant to subduct. That is a real fact about the
Earth and it makes an excellent invariant: log total continental area every step,
and it should never fall. If it drifts down, your collision resolution is losing
cells; if it climbs, you are duplicating them. This single check catches most bugs
in this milestone.

**Thickening needs a cap.** Two continents converging for four hundred steps will
produce crust a hundred kilometres thick and a mountain range fifteen kilometres
high if you let them. Cap thickness, or bleed it sideways into neighbouring cells
so ranges spread rather than spike. Real orogens do the latter.

**Arcs are a band, not a line.** Thickening exactly the one cell adjacent to a
subduction zone gives you a one-cell-wide wall. Spread the uplift over a few cells
inland with a falloff — the arc sits some distance behind the trench, which is
why there's a forearc basin between them.

**Don't tune with the region in mind.** It is tempting to keep adjusting until one
particular window looks good. Tune for a plausible *world*; picking a good window
is [M1f](m1f-region.md)'s job and it is much better at it than you are.

---

## The smoke test worth more than the unit tests

**Seafloor age stripes.** Render crust age through a ramp and look at the ridges.

Real seafloor shows symmetric bands of age either side of a spreading ridge — the
magnetic anomaly stripes that confirmed plate tectonics in the first place. If
your ridges produce symmetric stripes, your gap-filling and your ageing are both
right. If they are one-sided, crust is only being created on one flank. If they
are ragged, your gap detection is intermittent.

No assertion available here tells you as much as one look at that image.

---

## Tests

- Continental cell count is non-decreasing across the entire run.
- Total cell count is invariant; no cell is ever unassigned after gap-fill.
- Crust created at a gap has age zero and minimum thickness.
- Age increments for every surviving oceanic cell every step.
- A hand-built two-plate convergence produces thickening in the contact zone and
  nowhere else.
- A hand-built two-plate divergence produces a ridge of age-zero crust down the
  middle, symmetric to within a cell.
- Isostasy: continental crust at default thickness is above the datum and oceanic
  below; age-0 oceanic is shallower than age-80 oceanic.
- Thickness never exceeds the cap.
- Determinism: same seed, byte-identical fields.

---

## Acceptance

M1e is done when:

1. Continental crust clumps into recognisable continents rather than staying
   scattered, and has not collapsed into a single supercontinent.
2. Continent–continent convergence shows a thickened interior range with no
   trench; ocean–continent shows a coastal range *with* a trench.
3. Spreading ridges show symmetric age stripes.
4. Total continental area is conserved across the run, logged and asserted.
5. The elevation preview, rendered through a bathymetry ramp, looks like a world
   — continents, shelves, deep basins, mountain belts in sensible places.
6. Same seed, byte-identical output.

Then start [M1f](m1f-region.md), and go and find somewhere to live.
