# M1a — Logging

**Goal:** a run that tells you what it is doing, on screen and in a file you can
read afterwards.

Small, and first, because it is how you will watch everything in
[M1b](m1b-render.md) through [M1g](m1g-config.md). A five-hundred-step plate
simulation with no output is a program that appears to have hung.

Deliverables: `tracing` wired up in `main.rs`, a per-run output directory, and
`#[instrument]` on whatever counts as a stage today.

---

## The decision this unlocks

**Each run gets its own directory: `<out>/<seed>/`.**

M0 writes `<out>/<seed>.noise.png` and `<out>/<seed>.rings.png` — seed as a
filename prefix. That doesn't survive contact with M1, which wants a log, several
layer renders, a flipbook of a few hundred frames, and eventually a params file.
Move to a directory per run and name files for their contents: `noise.png`,
`rings.png`, `run.log`, `steps/0000.png`.

It belongs in this milestone rather than the next because the log file has to go
somewhere, and that somewhere is the same decision.

Keep the seed as the directory name. It is the one identifier that means anything
to you at a glance, and this is the last milestone at which "seed" and "world"
are the same thing — [M1g](m1g-config.md) complicates that, deliberately.

---

## Why `tracing` rather than `log`

`log` plus `env_logger` is less setup and would do for M0. Two things earn the
extra weight here.

**Spans.** `#[instrument]` on a stage function attaches that stage's name to
every message emitted inside it, however deep, and gives you its duration for
free. With a pipeline of seven stages and a step loop inside one of them, that is
the difference between a log you read and a log you grep.

**Two sinks, different filters.** You want terse on screen and verbose in the
file, from the same call sites. `tracing-subscriber` composes layers; `log` makes
you choose.

---

## What to build

Two `fmt` layers on one subscriber:

- **stderr**, filtered by a repeatable `-v` flag, with `RUST_LOG` overriding it
  when set. Default should be quiet enough that a successful run is a handful of
  lines.
- **`<out>/<seed>/run.log`**, filtered separately and more permissively. Disk is
  cheap and the whole point is to still have the detail tomorrow.

Note stderr, not stdout. Logs are diagnostics; if anything is ever piped out of
this program it should be data.

### What goes at which level

| Level | For |
|---|---|
| `error` / `warn` | Something is wrong or suspicious. A rejected world, a clamped parameter, a stage that produced a degenerate result. |
| `info` | Stage boundaries, timings, and the handful of values you'd want in a bug report — seed, grid size, plate count, land fraction. |
| `debug` | Per-step: iteration number, and the one or two aggregates you'd watch to see a sim converging. |
| `trace` | Per-cell. Off by default, and expect it to be unusably large when on. That's fine — that's what it's for. |

The rule that matters: **never log per-cell at `info`.** A 1024 × 512 grid is
half a million cells and one stray `info!` inside a full-grid pass will make the
log useless and the run slow.

---

## Gotchas

**Installing the subscriber requires arguments you haven't parsed yet.** The file
layer needs the seed and the output directory, and both come from the CLI. So the
order is: parse arguments, create the directory, install logging. Anything that
fails before that — a bad flag, an unwritable output path — reports itself on
plain stderr with no formatting, and that is acceptable. Don't contort the
program to log its own startup.

**Logging must not touch the output.** It must not consume the RNG, must not
depend on iteration order in a way that changes it, and must not be load-bearing
for any calculation. This sounds obvious and is exactly the sort of thing that
creeps in via a `debug!` that formats a value it also mutates.

**Silent no-ops in tests.** With no subscriber installed, every macro is a no-op.
That is the right default, but it means a test cannot accidentally assert on log
output — and shouldn't try to. If you ever do want to test a message, use
`tracing-subscriber`'s test writer explicitly rather than relying on the global.

**Timing a step loop.** `#[instrument]` on a function called five hundred times
gives you five hundred spans, which is noise. Instrument the stage, not the step;
log an aggregate every N steps at `info` and the per-step detail at `debug`.

---

## Tests

Thin, deliberately — this is plumbing, and most of it is better checked by
running the program than by asserting on it.

- The output directory is created if absent, and an existing one is reused rather
  than failing.
- A run with logging configured writes a non-empty `run.log`.
- **The golden-seed test from M0 passes identically with logging off and with
  logging at `trace`.** This is the one that matters. It is the assertion that
  logging is inert with respect to output, and it costs nothing to run both ways.

---

## Acceptance

M1a is done when:

1. `cargo run --release -- --seed 42` writes `out/42/` containing `noise.png`,
   `rings.png` and `run.log`.
2. A default run prints a few readable lines to stderr; `-vv` prints
   substantially more; `RUST_LOG` overrides both.
3. `run.log` contains stage names and durations without you having written any
   timing code.
4. The golden-seed test passes at every verbosity, including off.

Then start [M1b](m1b-render.md).
