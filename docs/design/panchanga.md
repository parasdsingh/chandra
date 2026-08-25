# Panchanga and jyotisha fields

The day view gains yoga, karana, muhurtas, dignity, drishti, planetary war and
the nakshatra lord. That roughly doubles what a day holds, on a surface D-025
has just made readable, so this document settles the shape before any of it is
built.

Status: **architecture, not yet approved.**

---

## 1. What is being added

| Field | Tier | Gate |
|---|---|---|
| Yoga | cheap — `(Sun + Moon) / 13°20′` | settings toggle |
| Karana | cheap — half a tithi | settings toggle |
| Muhurtas | needs sunset and the next sunrise | settings toggle |
| Dignity — exalted, debilitated, moolatrikona, own sign | free — a lookup | always on |
| Nakshatra lord | free — already on `NakshatraSpan` | always on |
| Drishti — aspects cast, aspects received | cheap — nine longitudes at one instant | always on |
| Planetary war | cheap — the same nine longitudes | always on |

---

## 2. The split: two panes

Decided. The day view carries a segmented control under the title.

| DAY | POSITION |
|---|---|
| Tithi | Rashi |
| Nakshatra | Nakshatra lord |
| Yoga | Motion |
| Karana | Combust |
| Vara | Dignity |
| Sunrise, sunset | Drishti |
| Muhurtas | Planetary war |
| | Rise and set |

The split is on **what the fields are about**, not on how many there are:

- **Day** is a property of the civil day. Every one of these nine panels shows
  the identical Day pane, because a tithi does not depend on which graha is
  being read on it. That is already true of the Tithi field today.
- **Position** is a property of the subject. Nothing in it is the same for two
  subjects.

That makes the pane a fact about the data rather than a bin, so a field added
later has one obvious home.

**The pane is per-panel state, not per-day.** Opening a second day keeps the
pane you were in; a pane that reset on every day would make comparing the same
field across days a two-click operation.

**Combustion and retrograde stay on the surface** (§5.1 of `TODO.md`), so both
states are visible from the Day pane as well. That is why they were built as a
wash and a rail rather than as rows: a row is only visible in the pane it is in.

---

## 3. Payload

### 3.1 `DayPanchanga` stops being optional

Today it is `Option`, present only in a lunar month, because D-010 reasoned that
a solar grid states no tithi and so has no number to explain.

That reasoning was about the **grid**, where 42 cells each cost a sunrise. It
does not carry to a **day**, which costs one. And yoga, karana and muhurta are
facts about a civil day whichever calendar names the month it sits in — a solar
Day pane that could not say when Rahu Kaal is would be missing the fields for a
reason that has nothing to do with them.

So: `panchanga: DayPanchanga`, always present, in both modes and both subjects.

```rust
pub struct DayPanchanga {
    pub tithis: Vec<TithiSpan>,
    pub yogas: Vec<YogaSpan>,        // empty when the setting is off
    pub karanas: Vec<KaranaSpan>,    // empty when the setting is off
    pub muhurtas: Vec<Muhurta>,      // empty when the setting is off
    pub sunrise: Option<Moment>,
    pub sunset: Option<Moment>,
    pub reference: Reference,
    pub vara_name: String,
}
```

Empty rather than `Option` for the three gated lists: "off" and "none today" are
the same thing to a reader — the row is absent either way — and a `Vec` cannot
be `Some(vec![])` by mistake.

### 3.2 `Standing`, on both subjects

```rust
pub struct Standing {
    pub dignity: Option<Dignity>,      // None where the graha has no dignity table
    pub nakshatra_lord: Graha,
    pub aspects: Vec<Graha>,           // cast by the subject
    pub aspected_by: Vec<Graha>,
    pub war: Option<War>,
}
```

`MoonDay` and `GrahaDay` both carry it. The Moon has dignity (exalted in
Vrishabha, debilitated in Vrishchika), casts and receives drishti, and is
excluded only from planetary war.

### 3.3 Cost

One extra `engine.positions` call at the reference instant for all nine
longitudes, which drishti, war and dignity all read from. One extra `rise_set`
for the following day's sunrise, only when muhurtas are on. Yoga and karana add
root searches only for their own boundaries, and only when on.

---

## 4. The astronomy

### 4.1 Yoga

`(Sun + Moon) / 13°20′`, 27 of them, named. The sum of two longitudes that both
only increase, so like a tithi and unlike a rashi it has no retrograde case and
the index only ascends — `tithi.rs`'s search machinery applies unchanged.

### 4.2 Karana

Half a tithi: `(Moon − Sun) / 6°`, 60 per lunar month. Eleven names — seven
movable repeating eight times from karana 1, and four fixed occupying the last
three and the first of the cycle. Derived from the tithi boundary search rather
than searched again: a karana boundary is either a tithi boundary or its
midpoint.

### 4.3 Muhurtas

All divide the day or the night into equal parts, so all need sunrise, sunset
and the next sunrise. `Engine::rise_set` already computes sunset; nothing
called it.

| Window | Definition |
|---|---|
| Rahu Kaal | day in 8 parts, part chosen by vara |
| Yamaganda | day in 8 parts, part chosen by vara |
| Gulika | day in 8 parts, part chosen by vara |
| Abhijit | the 8th of 15 equal parts of the day |
| Brahma Muhurta | the 14th of 15 equal parts of the night |
| Durmuhurtam | day in 15 parts, one or two chosen by vara — **table unverified, see §7** |

Where the Sun does not rise or set, the muhurtas are absent rather than computed
against a substitute instant. A window defined as a fraction of daylight has no
meaning on a day with no daylight, and `Reference::LocalNoon` already exists to
say why.

### 4.4 Dignity

A lookup on the rashi the graha occupies: exaltation, debilitation, moolatrikona
and own sign, from the standard table. Read at the day's reference instant, like
everything else in the Position pane.

### 4.5 Drishti

Classical Parashari graha drishti, whole-sign:

| Graha | Aspects the |
|---|---|
| all | 7th |
| Mangala | 4th, 7th, 8th |
| Guru | 5th, 7th, 9th |
| Shani | 3rd, 7th, 10th |
| Rahu, Ketu | 5th, 7th, 9th |

Whole-sign rather than by degree, because that is what the classical rule is:
a graha aspects a *house*, and the grahas in it. Counting is inclusive from the
occupied rashi.

Rahu and Ketu's aspects are not universally agreed. The 5/7/9 set is the common
one and is what is used; the disagreement is noted here rather than resolved.

### 4.6 Planetary war

Two of the five tara grahas — Mangala, Budha, Guru, Shukra, Shani — within one
degree of longitude. The Sun, Moon and nodes are excluded by the classical
definition.

The **winner is not reported.** Which graha wins a graha yuddha is decided
differently by different authorities — by latitude, by brightness, by whose
longitude is greater — and choosing one and printing it as a fact would be this
app asserting an interpretation. The pairing and the separation are the
observation; that is what is shown.

---

## 5. Settings

Three toggles, in a `Panchanga` section: **Yogas**, **Karanas**, **Muhurtas**.
Off by default — the day view is readable today and these are for someone who
came looking for them.

The toggles reach the back end: `Almanac::day_detail` takes a `DayOptions`, and
a field switched off is not computed rather than computed and hidden. That is
the same contract D-010 set for tithis in a solar grid.

Dignity, drishti, planetary war and the nakshatra lord have no toggle. They cost
one positions call between them, and the user named them as always-on.

---

## 6. Order of work

1. `DayOptions` through the facade; `DayPanchanga` made non-optional.
2. Yoga and karana, with tests against known dates.
3. Sunset, then the muhurtas that need only sunrise and sunset.
4. Dignity, nakshatra lord, drishti, planetary war — all from one positions call.
5. The two panes and the segmented control.
6. The three settings toggles, and the settings schema bump.

---

## 7. Open — needs a source before it is written

**The Durmuhurtam table.** Rahu Kaal, Yamaganda and Gulika have one settled
vara-to-part mapping and it is not in dispute. Durmuhurtam's is: which of the 15
day parts are inauspicious differs between published panchangas, and several
varas carry two windows rather than one.

Writing a table from memory would be hardcoding a guess, which is exactly what
this project does not do. Either it is taken from a named source and cited in
the code, or Durmuhurtam ships in a later pass and the other five go now.
