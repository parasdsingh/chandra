# Contributing

## Commits

The history is meant to be read. Each commit says what changed and, more
importantly, **why** — the reasoning that is true when the change is made and
is otherwise lost by the time anyone asks.

- A subject line under 72 characters, lower case, no full stop, in the
  imperative: `fix: the selection ring did not move`.
- A conventional prefix: `feat`, `fix`, `docs`, `chore`, `refactor`, `perf`,
  `build`, `test`.
- A body. Every commit here has one. Say what was wrong, what it cost, and why
  this is the fix rather than another — including the measurement, if the
  decision turned on one. "Fixed a bug" tells the next reader nothing they
  could not see from the diff.
- Where a change is a trade, name what it costs. A commit that only lists
  benefits is a commit that hid something.

## Before committing

```
make check      # fmt, clippy -D warnings, tsc, the token checker, 224 tests
```

`make check` is the whole gate. **There is no CI** — a green run means green on
the machine it ran on, which is why it has to be run.

Changes to the chart's layout need more than that: open `?preview` in a dev
build and look, because the front end has no automated test (see
[I-055](docs/ISSUES.md)). The geometric properties the chart depends on — no
label outside its compartment, none on a caption, none clipped — are checked by
eye and by console scripts that are not in this repository. That is a known gap
and the first thing worth closing.

## Where decisions live

Not in a chat log. `docs/DECISIONS.md` holds one record per decision with what
it cost; `docs/ISSUES.md` tracks work; `docs/design/` holds the specifications
features were built to. A decision that reverses an earlier one says so in both
places.

## Licence

AGPL-3.0, because Chandra links the Swiss Ephemeris under it. Anything
contributed is under the same licence, and the source has to stay reachable
from wherever a binary is offered — see [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).
