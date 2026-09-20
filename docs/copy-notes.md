# Copy that reads as machine-written

Notes for a rewrite. **Nothing here has been changed** — this is a list, and the
rewriting is a separate pass with a person's ear on it.

Criteria came from `ai-writing-signs.md` and `cv-style-guide.md` in the `self-cv`
project rather than from instinct, so a finding can be argued with by pointing
at the rule it cites.

Scope: text a user or visitor reads — the landing page, the feedback form, the
README, and the strings rendered in the app. **Code comments and commit messages
are deliberately out of scope**; they are internal, discursive on purpose, and
judging them by marketing-copy rules would flatten the thing that makes them
useful.

---

## The four habits, which matter more than the instances

Fixing the individual passages below without breaking these will leave the copy
sounding exactly the same.

### A. "X, not Y" is the house tic

**Twenty-two instances** across three files — 14 in `SettingsView.tsx`, 7 in
`site/index.html`, 1 in `README.md`. This is the clearest single signal in the
project.

It is not always wrong. The test: keep it only where a reader would otherwise
hold the wrong belief. "The chart it opens is where the nine grahas stand now,
not a birth chart" earns it — somebody really would assume otherwise. "A lunar
month runs between syzygies, not between calendar dates" does not; nobody
thought it ran between calendar dates, and the clause exists to balance the
sentence.

### B. Colon signposting inside a sentence

`SettingsView.tsx` lines 953, 984, 1088, 1127, 1188, and `feedback.html:73`.
Every one is a clause that wanted to be a full stop. "Off by default: template
icons follow the menu bar" is a heading pretending to be prose.

### C. Hint paragraphs all share one shape — fact, fact, epigram

`index.html` 303 and 343, `SettingsView.tsx` 987, 1089, 1126. The closing
fragment is always shorter than what precedes it and always restates it. "A
reading of now that looks like one" adds cadence and no information.

### D. The em dash as the default subordinate clause

`index.html` 222, 308, 328, 343; `SettingsView.tsx` 956, 985; `feedback.html`
73, 103; `README.md` 14. It has become the only tool used for any aside.

---

## Two concrete inconsistencies, verified

**The same sentence appears on two surfaces.** "why two grahas in one sign sit
where they do" is in `site/index.html:343` and in `SettingsView.tsx`'s degree
grid hint. A phrase surviving verbatim from an app hint into a marketing caption
reads as copy-paste rather than as two descriptions of one feature.

**The serial comma is used in one file and not the other, and inconsistently
inside one of them.** `README.md` has "Rust, Node, and Xcode" and "Amanta,
purnimanta, or"; `site/index.html` has "yoga, karana and the muhurtas", "rashi,
nakshatra and pada", "Size, Advanced and About" — but also "it, authenticate,
and open". Per the checklist this reads as two different hands. Pick one and
hold it.

---

## Passages, worst first

| # | Where | The sign |
|---|---|---|
| 1 | `SettingsView.tsx:982` — the starry sky hint | Colon signposting, negative parallelism and an em dash aside, all inside four sentences, for a toggle that draws a background |
| 2 | `index.html:328` — retrograde caption | "a field you can read, not a mark you have to know" — the textbook negative parallelism, then an em dash restating it |
| 3 | `index.html:276` — "What it does" lede | "what you want is one click away and what you don't is not there at all" — mirrored symmetry reads as constructed |
| 4 | `SettingsView.tsx:955` — degree lines hint | Em dash aside plus a reasoning tail explaining what the reader just read |
| 5 | `index.html:343` — degree grid caption | The duplicate of #4, reworded |
| 6 | `index.html:303` — feature 05 | "A reading of now that looks like one." Epigram-fragment closer; the previous sentence already said it |
| 7 | `SettingsView.tsx:1087` — ingress hint | "Not on the Moon's calendar:" — a heading pretending to be a sentence |
| 8 | `SettingsView.tsx:1125` — compartments hint | Colon signposting, plus "A number is a lookup" as a manufactured aphorism |
| 9 | `feedback.html:71` — the lede | Rule-of-three closed by "all of it is useful", then "Especially the last:" — the most recognisable AI paragraph shape on the site |
| 10 | `index.html:237` — first-run summary | "Here is why, and what to do." Announces its own structure instead of stating the problem |
| 11 | `index.html:240` | "a statement about a certificate, not about the software" — restages "the app is fine" as an epigram |
| 12 | `SettingsView.tsx:284` — month system hint | "between syzygies, not between calendar dates" — contrast nobody needed |
| 13 | `index.html:377` — support lede | "put something behind it", "this is the way" — two evasions of the word *donate* |
| 14 | `index.html:308`, `425` | Tricolon of negatives twice on one page for the same claim |
| 15 | `SettingsView.tsx:1188` | Colon signposting, fourth instance in the file |
| 16 | `SettingsView.tsx:797` | Negative parallelism — though this one is close to earning it |
| 17 | `README.md:48` | "Design and decisions live in the repository, not in chat." Negative parallelism, **and addressed to the wrong reader**: "not in chat" is an instruction to a coding assistant, on the front page of a public repository |

---

## Clean, named rather than passed over in silence

- **The facts table**, `index.html:359`. Ten rows, no adjectives, no framing.
- **The footer licence paragraph**, `index.html:430`.
- **Every error string in `DayDetail.tsx`.** "Not a date." / "That day does not
  exist in the calendar." Short, specific, no apology theatre — the best-written
  strings in the project.
- **The provenance notes.** "Outside 1800–2399. Times here are approximate, by
  about a second." Gives the magnitude instead of a vague warning.
- **`spoken()` and `describe()`** in `Chakra.tsx` — assembled from data, no
  editorial voice, which is right for a screen reader.
- **The whole Location section** of `SettingsView.tsx`, and `LocationGate`.
- **`feedback.html`'s labels, hints and thank-you.** Only its lede needs work.
- **`README.md`'s Building, Ephemeris and Licence sections.** Line 48 is the
  file's only offender.
- **The hero**, `index.html:217`. "The sky, where you already look." is a
  written line, and the lede under it states four concrete things without a
  single contrast construction.
