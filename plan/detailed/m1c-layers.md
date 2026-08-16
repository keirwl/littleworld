# M1c — Layers and Wrapping

**Goal:** decide how a set of layers is held and passed around, and teach `Grid`
to wrap.

Two small structural pieces that both have to land before [M1d](m1d-plates.md).
Neither is much code. The wrapping half carries this milestone's real tests.

---

## A warning about the first half

The normal rule for abstraction is: build it once you have two instances of the
thing. Right now there is one layer — M0's fBm field — and the second set
arrives in M1d. **You are designing this one instance early, which is the classic
way to design it wrong.**

The mitigation is to keep this deliberately thin. What follows is a set of
**conventions**, not a framework: a struct, a rule about dimensions, and a shape
for stage functions. All of it is a morning's work and all of it is cheap to
change.

**If it starts wanting traits, generic pipeline machinery, or a registry of
layers looked up by name at runtime, stop — that is the signal it should have
waited for M1d.** Write the tectonics code against a plain struct, see what shape
it actually wants, and generalise afterwards if it still seems worth it.

---

## The conventions

**A world is a struct of named grids that share dimensions.** Not a map of names
to grids, not a `Vec<Layer>` — named fields, per rule four in
[plan.md](plan.md), so a layer has a type and adding one doesn't touch a
god-struct.

**Dimensions are validated once, at construction, not assumed.** Every `Grid`
already carries its own width and height, which means a world holding several can
hold several that disagree. Check on the way in and it can never happen; skip the
check and you get a panic three stages later with no clue where the mismatch came
from.

**A stage is a pure function over layers**, per rule two: it reads earlier layers,
takes config and an RNG, and returns new ones. Not a method that mutates a world
in place. The point is that a stage can be run twice with the same inputs and
compared, which is most of how you will debug M1d and M1e.

**Layer names drive output filenames.** If the field is `crust_age`, the render
is `crust_age.png`. It sounds trivial and it is the difference between a run
directory you can navigate and forty PNGs called `output3.png`.

**Enum-valued layers pair with `Palette`**, continuous ones with `Ramp`, from
[M1b](m1b-render.md). Worth deciding now so the enums get defined with rendering
in mind rather than retrofitted.

### Explicitly deferred

Trait-based stage pipelines. Dynamic layer registration. Any mechanism for
iterating layers generically — the renderer takes one grid, and the orchestration
code loops explicitly, which is three lines and perfectly readable.

---

## Cylinder wrapping

The tectonic grid wraps in columns. The region grid must not — its edges are real
edges, which is what the 30% margin exists to handle.

**Set wrapping at construction, not per call.** It is a property of the grid, and
a boolean threaded through `neighbours`, `ring` and `spiral` is a boolean you will
eventually pass wrongly.

### It is one `rem_euclid`, and it needs an even width

Wrapping an odd-q hex grid in columns looks like it should be painful, because
the half-cell shear accumulates with `q`. It isn't.

For hexes `A = (q, r)` and `B = (q + w, r − w/2)`, with `w` even so the parity of
`q` is unchanged:

```
to_oddq(A) = (q,     r + (q − p)/2)
to_oddq(B) = (q + w, r − w/2 + (q − p)/2 + w/2)
           = (q + w, r + (q − p)/2)
```

The offset **rows are identical**. So the entire wrap is `col.rem_euclid(w)`
applied inside `index()` after `to_oddq`, and nothing else. `neighbours`, `ring`
and `spiral` all route through `index()`, so they inherit it for free.

This was checked, not reasoned. On a wrapping 1024 × 64 grid every interior cell
has exactly six distinct in-bounds neighbours, and a full lap returns to the
starting cell from every cell on the grid.

### The one gotcha that will bite you

**An odd-width wrapping grid does not fail loudly. It fails by one row, on half
the cells.**

With `w` odd, `parity(q + w) = 1 − parity(q)` and `w/2` isn't an integer, so a
lap lands you one row off — but only for cells of one parity. Measured on a
1023 × 200 wrapping grid: row drift after a lap is `0` for some cells and `−1`
for others, and **101,888 of 204,600 cells land on a real but wrong cell.**

Not out of bounds. Not fewer neighbours. Every interior cell still reports
exactly six neighbours, so no neighbour-counting test catches it. What you would
see is a seam down one meridian where plates tear and, three milestones later,
rivers step sideways — and you would blame the plate simulation.

**Assert even width in the wrapping grid's constructor.** One line, and it closes
the whole category.

This is [m0.md](m0.md)'s "never do neighbour arithmetic in offset space" wearing a
different hat: the parity-dependent bug that looks fine until it doesn't.

---

## Tests

The wrapping identities are this milestone's equivalent of M0's hex identities —
cheap, and they catch the invisible one.

- On a wrapping grid of even width, every cell not in the top or bottom row has
  exactly six neighbours, all distinct and in bounds.
- A lap of `w` steps eastward returns to the starting cell. **Run it from every
  cell** — the odd-width failure is parity-dependent and a single sample misses
  half of them.
- Constructing a wrapping grid of odd width fails.
- A non-wrapping grid behaves exactly as it did at M0. Don't let the wrap leak
  into the default path; the M0 tests passing unchanged is the check.
- Ring and spiral cell counts (`6n`, and `1 + 3n(n+1)`) hold on a wrapping grid
  for rings that cross the seam — on a non-wrapping grid an edge cell yields
  fewer, and the difference is the point.
- A world constructed from grids of mismatched dimensions is rejected.

---

## Acceptance

M1c is done when:

1. M0's noise field and ring grid live in a world struct, are rendered by looping
   over named layers, and land in `<out>/<seed>/` named after those layers.
2. The wrap tests above pass, including the every-cell lap.
3. An odd-width wrapping grid cannot be constructed.
4. The M0 golden-seed test is unaffected.
5. Nothing in this milestone required a trait.

Then start [M1d](m1d-plates.md), which is where it gets interesting.
