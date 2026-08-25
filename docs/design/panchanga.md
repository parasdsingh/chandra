# Panchanga and jyotisha fields

The day view gains yoga, karana, muhurtas, dignity, drishti, planetary war and
the nakshatra lord. That roughly doubles what a day holds, on a surface D-025
has just made readable, so this document settles the shape before any of it is
built.

Status: **built.** What shipped differs from this document in three places, each noted where it applies.

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
| Yoga | Nakshatra |
| Karana | Nakshatra lord |
| Daylight | Motion |
| Muhurtas | Dignity |
| | Combust |
| | Drishti |
| | Planetary war |
| | Rise and set |

**Changed from the table above.** Nakshatra moved to Position. It is the
panchanga's fifth limb, but the payload carries the *subject's* nakshatra - on a
Mangala day, Mangala's - so a Nakshatra row under Day would have been true only
when the subject happened to be the Moon.

**Vara is not a row.** It is already in the line under the header, beside the
weekday, and a day has one.

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
| Brahma Muhurta | the 14th of 15 equal parts of the night **before this day's sunrise** |
| Durmuhurtam | day in 15 parts, one or two chosen by vara — table in §7 |

**Changed from the line above.** A civil day touches two nights, and the rules do
not agree on which they mean. Brahma Muhurta ends shortly before dawn, so
today's is the one in this morning's small hours - which is what a published
panchanga prints, and the only reading of use to someone planning to be awake
for it. Taken from the night after sunset it printed at the foot of the day's
list at 03:45 the *following* morning, which reads as a sorting fault. Tuesday's
second Durmuhurtam stays on the night after, because its source states it as an
offset after sunset.

Where the Sun does not rise or set, the muhurtas are absent rather than computed
against a substitute instant. A window defined as a fraction of daylight has no
meaning on a day with no daylight, and `Reference::LocalNoon` already exists to
say why.

### 4.4 Dignity

A lookup on the rashi the graha occupies: exaltation, debilitation and own sign.

**Changed from the line above: moolatrikona is not included.** Its degree ranges
differ between authorities, so unlike exaltation and own sign it cannot be stated
without choosing one - which would be this app asserting an interpretation. It
was also not among the states asked for. Read at the day's reference instant, like
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

## 7. The Durmuhurtam table

Taken from a published source rather than written from memory, and the source is
cited in the code. Two of the values I would have written from memory were
wrong — Friday's second window and Saturday's — which is what the rule against
hardcoding guesses is for.

The source states the windows as an offset in hours and minutes after sunrise,
on the assumption of a twelve-hour day. One day muhurta is then 48 minutes, so
an offset divided by 48 minutes gives the muhurta index directly, and the index
is what is implemented: it stays correct at a latitude where the day is not
twelve hours, which the clock offsets do not.

| Vara | Source offset | Muhurta |
|---|---|---|
| Sunday | 10h24m after sunrise | day 14 |
| Monday | 6h24m, then 8h48m after sunrise | day 9, day 12 |
| Tuesday | 2h24m after sunrise; 5h36m after **sunset** | day 4, night 8 |
| Wednesday | 5h36m after sunrise | day 8 |
| Thursday | 4h00m, then 8h48m after sunrise | day 6, day 12 |
| Friday | 2h24m, then 8h48m after sunrise | day 4, day 12 |
| Saturday | from sunrise, lasting 1h36m | day 1, day 2 |

Tuesday's second window is the only one in the night, and is the reason the
muhurta type carries which half of the day it divides rather than assuming the
day half.

The table corroborates itself against a fact from a different source:
Abhijit is the 8th day muhurta and is held not to apply on a Wednesday — and
Wednesday's Durmuhurtam is the 8th day muhurta. The two rules are the same
observation.

Sources:
- <https://www.oursubhakaryam.com/what_is_durmuhurtam_in_telugu_panchangam.html>
  — the vara offsets.
- <https://www.sanatanveda.com/astrology/simple-way-of-calculating-muhurta/>
  — 15 day muhurtas and 15 night muhurtas, Abhijit as the 8th of the day.
- <https://en.wikipedia.org/wiki/Brahmamuhurta> — Brahma Muhurta as the 14th
  muhurta of the night.
