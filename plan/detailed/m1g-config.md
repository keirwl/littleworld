# M1g — Config and Reproducibility

**Goal:** every tunable in a file, every resolved value written back out, and a
world you can ask for by name a year from now.

Last of M1, and last deliberately. An earlier draft of [m1.md](m1.md) put config
first, in the housekeeping block. That was wrong: **you cannot design a config
schema for parameters that don't exist yet.** By the time you arrive here you
have spent three milestones turning knobs by editing constants, and you know
exactly which ones deserve to be in a file.

Deliverables: `config.rs`, `<out>/<seed>/params.toml`, and safe world rejection.

---

## Two phases, and the distinction is the whole design

**Declared** config is what you asked for: defaults, overlaid by a file, overlaid
by CLI flags. It may contain ranges, or holes meaning "pick something sensible".

**Resolved** config is what actually happened: every value concrete, after every
random draw has been made.

Dump the resolved form to `<out>/<seed>/params.toml`. And the requirement that
makes it worth writing at all:

> **The dump must load back in as declared config**, and reproduce the same world.

A run must be reproducible from its config alone, not only from its seed. Without
that property the file is a receipt; with it, the file is an address.

Precedence is defaults < file < CLI flags. `Option<T>` fields plus `Default`
impls is the standard way to hand-roll the layering; crates like `figment` exist
and would do it for you, but this is a project for understanding what you built.

### Why TOML

[plan.md](plan.md) originally said RON. RON round-trips Rust enums and newtypes
exactly, which matters for structured data dumps — settlements, cultures — and
doesn't matter for a file you edit by hand, where what you want is comments and
familiarity. Use TOML here; keep RON later for the structured data if it earns its
place.

Nest by stage: `[tectonics]`, `[region]`, and so on. It mirrors the pipeline and
it means a diff between two runs points at the stage that differed.

### Config and logging are not the same thing

They answer different questions and it's worth being deliberate about it:

> **The log records what happened. The params file records what was chosen.**

It is tempting to answer "what did this run pick?" by logging the parameters, and
it half works. But a log line is grep-bait and a params file is an input — one of
them lets you re-run the world and the other doesn't. Write both; don't let
either pretend to be the other.

---

## When a world is unsuitable

Dwarf Fortress rejects worlds and re-rolls. The mechanism worth copying is the
iterative seed — `hash(master + stage + attempt)`, incrementing on rejection —
and it lands here rather than in [M1e](m1e-geology.md) because it is only safe
once `params.toml` exists to record which attempt was accepted.

But reach for it last. There are four responses to an unsuitable world and
rejection is the worst of them:

1. **Constrain the generator** so unsuitable worlds are rare by construction.
   Choosing plate velocities such that at least one convergent boundary exists is
   better than generating freely and discarding the worlds without mountains.
2. **Search within the world.** [M1f](m1f-region.md) already scores windows and
   takes the best, so a window is never rejected. A world that is poor on average
   is usually still good *somewhere*, and this removes most of DF's reasons to
   re-roll before they arise.
3. **Repair.** [plan.md](plan.md) picks sea level as a percentile of the
   elevation histogram precisely so land fraction is a knob rather than an
   accident. That is repair, and it beats rejecting a world for having the wrong
   amount of sea.
4. **Reject and re-roll**, for genuinely global failures only — the simulation
   collapsed into a single supercontinent, say.

DF needs rejection because its generator is fixed and its constraints are set by
the user. Here, most of what it rejects for is something to search for or dial.

### Where you do reject

- The attempt index is part of the stage label, so `RngMaster::for_stage` needs no
  new API — but give it an explicit method anyway so the convention can't drift
  into ad-hoc string formatting at call sites.
- **Write the accepted attempt index into `params.toml`.** This is the part that
  makes the scheme safe. A bare seed means "give me a good world"; the params file
  means "give me *this* world". Without it, tightening a constraint six months
  from now silently makes `Urist` resolve to a different world and invalidates
  every reference render downstream — which is exactly the failure rule one in
  [plan.md](plan.md) exists to prevent.
- Only the rejecting stage's seed advances. Later stages keep their own streams.
- Reject as early as it's cheap to. Check the partition before spending five
  hundred steps on it, not after.
- **Cap the attempts and fail loudly**, naming the constraint that failed most
  often and its measured value. An uncapped retry loop is how DF earned its
  reputation for rejecting a world three hundred times. Log every rejection with
  its reason and its number — that log is your tuning data.

And the discipline point that matters more than any of the above: **write no
rejection criteria until you have generated twenty worlds and looked at them.**
Constraints invented in advance reject worlds that were fine and miss the failures
that actually occur. Arriving here after M1d–M1f, you will have.

---

## Saving stages: designed here, built later

Still not the time. Nothing in M1 is slow enough to need resume.

Do add serde derives to `Grid<T>` while you're in the area, with one caution:
a derived `Deserialize` bypasses the constructor and with it the
`store.len() == w × h` invariant, which would let a malformed file produce a
`Grid` that panics on first access rather than failing to load. Derive over a
plain representation struct and convert with `#[serde(try_from = ...)]` so the
invariant is checked on the way in.

When you do build save/resume — erosion at M4 is the natural moment — note that
the argument for persisting the *tectonic* world isn't speed. It's that "world
Urist, region (410, 180)" becomes a durable address, and tectonics sits upstream
of every reference render you will ever make.

---

## Gotchas

**Resolved config must be complete.** If a value is drawn from the RNG rather than
read from config, it still has to appear in the dump — otherwise loading the dump
re-draws it and you get a different world, which breaks the one property the file
exists to provide. This is the failure mode to test for explicitly.

**Loading a dump must not re-randomise.** Related, and the other half of the same
trap: a declared config with every field populated should make no random draws for
those fields at all.

**Version the schema.** Put a version field in from the start. The first time you
rename a parameter, every params file you have is silently misread — a missing
field takes its default and the world changes. A version number turns that into an
error message.

**Don't config things that aren't tunable.** Grid dimensions, cell size and plate
count belong in the file. The FNV offset basis does not. A config file that
exposes everything is one nobody reads.

---

## Tests

- Resolved config round-trips: resolve → TOML → parse → identical.
- **Load a dumped `params.toml` with no `--seed` and get a byte-identical world.**
  The headline test; everything else here is detail.
- Precedence: a value set in defaults, file and flag resolves to the flag's; set
  in defaults and file resolves to the file's.
- A resolved config has no `None` fields.
- Loading a config with an unknown or missing version fails with a clear error.
- Rejection: a stage forced to reject twice produces the third attempt's world,
  and its `params.toml` records attempt 2.
- The attempt index changes only the rejecting stage's stream — a downstream
  stage's draws are unchanged for a fixed input.

---

## Acceptance

M1g is done when:

1. Every tunable in M1d–M1f reads from `config/default.toml`, overridable by
   `--config` and by flags.
2. `<out>/<seed>/params.toml` is written on every run and contains every resolved
   value, including those drawn randomly.
3. Re-running with `--config <that file>` and no `--seed` reproduces the world
   byte for byte.
4. A deliberately over-tight constraint fails after the attempt cap with a message
   naming the constraint and its measured value.
5. The golden-seed test still passes.

That closes M1. Start M2, and turn the region's structure into a coastline.
