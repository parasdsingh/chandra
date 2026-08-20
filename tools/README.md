# tools

## swetest

Swiss Ephemeris' own reference command line program, built from the C sources
vendored inside the `swiss-eph` crate. It is the reference implementation used to
generate golden test vectors.

It is a build artifact and is not committed. Rebuild with:

```sh
make swetest
```

## gen_golden.py

Regenerates `crates/ephemeris/tests/golden.json` by driving `swetest`.

```sh
make golden
```

Run this only when the expectations genuinely need to change - a Swiss Ephemeris
version bump, or new cases added to `DATES` / `PLACES`. Regenerating to make a
failing test pass would defeat the point of having it.
