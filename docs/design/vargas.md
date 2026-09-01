# Vargas — the sixteen divisional charts

Status: **research.** Nothing here is built. This is the specification the
implementation will be written against.

Scope: **D1, D3, D7, D9 and D12 are approved for the first pass.** All sixteen
of the Shodasavarga are documented here, because the rules are a single family
and writing five of them without the other eleven would mean discovering the
shared structure twice.

---

## 0. The standard this document is held to

`TODO.md` records that the Durmuhurtam table was written from memory and two of
its values were wrong. The rule that came out of that is **domain tables are
cited, not remembered.** This document is written to it:

| Rule | What it means here |
|---|---|
| Every rule carries a source | Chapter and verse of BPHS where one exists, and the translation named |
| Two independent sources | A BPHS translation, plus published software behaviour or a second author |
| Disagreements are printed, not resolved | The app already refuses to name a winner in a planetary war and gives the nodes no dignity (D-026). Printing one reading of a disputed rule would be the app asserting an interpretation |
| "Unverified" is a valid entry | §9 lists what could not be sourced. Gaps are not filled with plausible reconstruction |

Two errors found while writing this are recorded rather than quietly corrected:
one in a reference text (§7.12), one a self-contradiction inside a single
translation (§7.12). Both are in D-27.

---

## 1. Notation

Everything below is stated in terms the code already has.

| Symbol | Meaning | Existing code |
|---|---|---|
| `s` | Rashi index, `0` = Mesha … `11` = Meena | `Rashi::index()` |
| `d` | Degrees within the rashi, `0 ≤ d < 30` | `degrees_in_rashi()` |
| `n` | Number of parts the rashi is divided into | — |
| `w` | Segment width, `30 / n` degrees | — |
| `p` | Part index, `floor(d / w)`, range `0 … n−1` | — |

Three conventions, each of which is an off-by-one waiting to happen:

- **BPHS counts parts from 1.** Its "the 5th Navamsa" is `p = 4`.
- **BPHS counts signs inclusively.** "The 9th from Vrishabha" is Makara, not
  Kumbha: Vrishabha itself is the 1st. So "the kth from `s`" is
  `(s + k − 1) mod 12`.
- **An odd sign has an even index.** Mesha is the *first* sign, so it is odd,
  and its index is `0`. Throughout: **odd sign ⟺ `s % 2 == 0`.**

Formulas below are written as `(a·s + b + p) mod 12`, which is the form the
implementation should take: no counting loops, no tables.

---

## 2. The sixteen, in summary

| Varga | Name | `n` | Segment | Equal? | Rule | Signs it can occupy | Disputed |
|---|---|---|---|---|---|---|---|
| D1 | Rashi | 1 | 30° | yes | `s` | 12 | no |
| D2 | Hora | 2 | 15° | yes | odd sign: Simha then Karka; even sign: reversed | **2** | **yes** — six schemes in JHora |
| D3 | Drekkana | 3 | 10° | yes | `(s + 4p) mod 12` | 12 | **yes** — four schemes in JHora |
| D4 | Chaturthamsa | 4 | 7°30′ | yes | `(s + 3p) mod 12` | 12 | **yes** — two schemes in JHora |
| D7 | Saptamsa | 7 | 4°17′08.57″ | yes | `(7s + p) mod 12` | 12 | no |
| D9 | Navamsa | 9 | 3°20′ | yes | `(9s + p) mod 12` | 12 | **yes** — three schemes in JHora |
| D10 | Dasamsa | 10 | 3° | yes | `(s + 8·(s mod 2) + p) mod 12` | 12 | no |
| D12 | Dwadasamsa | 12 | 2°30′ | yes | `(s + p) mod 12` | 12 | no |
| D16 | Shodasamsa | 16 | 1°52′30″ | yes | `(4s + p) mod 12` | 12 | no |
| D20 | Vimsamsa | 20 | 1°30′ | yes | `(8s + p) mod 12` | 12 | no |
| D24 | Chaturvimsamsa | 24 | 1°15′ | yes | `(4 − (s mod 2) + p) mod 12` | 12 | no |
| D27 | Bhamsa | 27 | 1°06′40″ | yes | `(3s + p) mod 12` | 12 | **yes** — Saravali differs |
| D30 | Trimsamsa | 5 | **unequal**, 5/5/8/7/5 | **no** | table, §7.13 | **10** | **yes** — three schemes in JHora |
| D40 | Khavedamsa | 40 | 45′ | yes | `(6·(s mod 2) + p) mod 12` | 12 | no |
| D45 | Akshavedamsa | 45 | 40′ | yes | `(4·(s mod 3) + p) mod 12` | 12 | no |
| D60 | Shashtiamsa | 60 | 30′ | yes | `(s + p) mod 12` | 12 | no |

D30 is the only unequal division in the Parashari scheme, and the only one whose
name does not give the number of parts. A trimsamsa is a thirtieth of a rashi,
which is 1°; the Parashari scheme then groups those thirty degrees into **five**
unequal blocks of 5, 5, 8, 7 and 5. So a rashi has five D30 parts, not thirty.

Read for what, per BPHS ch. 7 v. 1–8 (Santhanam):

| Varga | Read for | Varga | Read for |
|---|---|---|---|
| D1 | the physique | D16 | conveyances |
| D2 | wealth | D20 | worship, spiritual progress |
| D3 | happiness through coborn | D24 | learning |
| D4 | fortunes | D27 | strength and weakness |
| D7 | sons and grandsons | D30 | evils |
| D9 | spouse | D40 | auspicious and inauspicious effects |
| D10 | power and position | D45 | all general indications |
| D12 | parents | D60 | all general indications |

---

## 3. Sign classifications

Four classifications are needed. Each is used by at least one varga rule, and
each is cited.

### 3.1 Movable, fixed, dual (chara, sthira, dwisvabhava)

Used by D9, D16, D20, D45.

| Class | Rashis | Indices |
|---|---|---|
| Movable (chara) | Mesha, Karka, Tula, Makara | 0, 3, 6, 9 |
| Fixed (sthira) | Vrishabha, Simha, Vrishchika, Kumbha | 1, 4, 7, 10 |
| Dual (dwisvabhava) | Mithuna, Kanya, Dhanu, Meena | 2, 5, 8, 11 |

`class = s mod 3`, with `0` movable, `1` fixed, `2` dual.

> "**CLASSIFICATION OF SIGNS:** Movable, Fixed and Dual are the names given to
> the 12 signs in order. […] *Notes:* The 12 signs are divided into movable,
> fixed and dual. Movable are Aries, Cancer, Libra and Capricorn. […] Taurus,
> Leo, Scorpio and Aquarius are fixed or immovable. Gemini, Virgo, Sagittarius
> and Pisces are dual or common."
> — BPHS ch. 4 v. 5–5½, Santhanam translation

Corroborated by P.V.R. Narasimha Rao, who uses "movable, fixed or dual" as the
selector for D-8, D-16, D-20 and D-45 without redefining it, and by the fact
that both sources' D-16 and D-20 tables agree row for row.

### 3.2 Odd and even (oja, yugma)

Used by D2, D7, D10, D24, D40, and by D30's degree split.

Odd signs are the 1st, 3rd, 5th, 7th, 9th, 11th — Mesha, Mithuna, Simha, Tula,
Dhanu, Kumbha — indices 0, 2, 4, 6, 8, 10. **Odd sign ⟺ `s % 2 == 0`.**

BPHS does not use the words "odd" and "even" in ch. 4; it uses male and female,
which are the same partition:

> "Aries, Gemini, Leo, Libra, Sagittarius and Aquarius are male signs. These are
> also known as malefic or cruel signs. Taurus, Cancer, Virgo, Scorpio,
> Capricorn and Pisces are female signs."
> — BPHS ch. 4 v. 5–5½ notes, Santhanam translation

The ch. 6 varga verses then say "odd sign" and "even sign" throughout, so the
identification male = odd is BPHS's own and not an inference across texts.

### 3.3 The four elements

Used by D27, and by the element statement of D9 (§7.6).

| Element | Rashis | Indices |
|---|---|---|
| Fiery (agni) | Mesha, Simha, Dhanu | 0, 4, 8 |
| Earthy (prithvi) | Vrishabha, Kanya, Makara | 1, 5, 9 |
| Airy (vayu) | Mithuna, Tula, Kumbha | 2, 6, 10 |
| Watery (jala) | Karka, Vrishchika, Meena | 3, 7, 11 |

`element = s mod 4`, with `0` fiery, `1` earthy, `2` airy, `3` watery.

BPHS states this twice, in two vocabularies. Per sign, in the descriptions:

> "**ARIES DESCRIBED:** […] and is fiery, its ruler is Mars."
> "**TAURUS DESCRIBED:** […] An earthy sign, Taurus rises with its back."
> "**GEMINI DESCRIBED:** […] It lives in the west and is an airy sign."
> "**CANCER DESCRIBED:** […] It is Satwik in disposition […] and is a watery sign."
> — BPHS ch. 4 v. 6–11, Santhanam translation

And by trines, in the temperament verse:

> "Aries and its trines are bilious. Taurus and its trines are windy. Gemini and
> its trines have a mix of all the three temperaments […] Cancer and its trines
> are phlegmatic."
> — BPHS ch. 4 v. 5–5½ notes, Santhanam translation

"Trines" here means the 1st, 5th and 9th, so the two statements give the same
four groups. Corroborated independently by Narasimha Rao, who states the D-27
rule directly as "fiery, earthy, airy or watery".

### 3.4 Day-strong and night-strong

Not used by any Parashari varga. Needed only to describe the Kashinatha D-2
variant (§7.2), and recorded here because the classification is easy to confuse
with §3.3.

| Class | Rashis |
|---|---|
| Nocturnal / night-strong | Mesha, Vrishabha, Mithuna, Karka, Dhanu, Makara |
| Diurnal / day-strong | Simha, Kanya, Tula, Vrishchika, Kumbha, Meena |

> "The signs Aries, Taurus, Gemini, Cancer, Sagittarius and Capricorn are
> nocturnal signs as these are strong during night time. The other six, viz.
> Leo, Virgo, Libra, Scorpio, Aquarius and Pisces are called diurnal signs being
> strong during day time."
> — Santhanam's notes to BPHS ch. 4, in the Adhana worked example

Stated as commentary there, but each sign's own verse in ch. 4 v. 6–11 carries
it too — Aries "strong during night", Leo "resides in the east and is
day-strong", and so on. Narasimha Rao gives the identical two lists when
defining the Kashinatha hora, which is the only place this classification is
needed here.

---

## 4. Three structural facts

These are derived from the rules in §7, not asserted. Each is a property the
implementation can be tested against.

### 4.1 Six of the sixteen are pure cyclic divisions

For a rashi at index `s` and a part index `p`, a longitude is `30s + d`, so

```
floor(longitude / w)  =  n·s + p        where w = 30/n
```

and the **continuous** form of a varga — divide the whole zodiac into equal
parts of `w` and take the part number mod 12 —

```
varga_sign = floor(longitude / (30 / n)) mod 12
```

equals the per-rashi rule exactly when the rule's starting sign is `(n·s) mod
12`. That is true for six of the sixteen, and only six:

| Varga | Continuous form valid | Why |
|---|---|---|
| D1 | **yes** | trivially |
| D7 | **yes** | `7s mod 12` is the stated start for every `s` |
| D9 | **yes** | `9s mod 12` is the stated start for every `s` |
| D16 | **yes** | `16s ≡ 4s (mod 12)`, the stated start |
| D20 | **yes** | `20s ≡ 8s (mod 12)`, the stated start |
| D27 | **yes** | `27s ≡ 3s (mod 12)`, the stated start |
| D2, D3, D4, D10, D12, D24, D40, D45, D60 | no | the stated start is not `n·s mod 12` |
| D30 | n/a | unequal |

Verified by exhaustive enumeration over all twelve rashis and every part.

Two of the near-misses are worth naming, because they look like they should
work and do not:

- **D12 and D60** both start from the rashi itself, so their rule is
  `(s + p) mod 12` — but `12s ≡ 0` and `60s ≡ 0 (mod 12)`, so the continuous
  form would start every rashi at Mesha. Wrong for both.
- **D2's** continuous form, `(2s + p) mod 12`, is not an error but a *different
  chart*: it is exactly the Parivritti Dwaya hora of §7.2.

This matters most for D9: **navamsa is the 108-fold division of the zodiac**,
which is why a navamsa boundary is also a nakshatra-pada boundary — 27
nakshatras × 4 padas = 108 = 12 rashis × 9 navamsas, all on the same 3°20′
grid. `zodiac.rs::pada` already computes that grid. The two must agree, and a
test asserting so is free.

### 4.2 Rahu and Ketu

The app computes Ketu as Rahu + 180° exactly (**D-003**). Nothing in BPHS ch. 6
gives the nodes a special division rule, and Narasimha Rao's chapter says the
rules apply to "a physical or a mathematical point in the zodiac that has a
longitude associated with it". So: **the nodes divide like any other body.**

The consequence is arithmetic, not interpretation, and it is a drawing
constraint. With Ketu exactly opposite Rahu, the separation between them *in the
varga* is fixed per varga:

| Separation in the varga | Vargas |
|---|---|
| Always the 7th from each other, as in D1 | D1, D3, D4, D7, D9, D10, D12, D27, D60 |
| **Always in the same rashi** | **D2, D16, D20, D24, D30, D40, D45** |

Proof for the second row: the rule for each of those seven has the form
`(a·s + b + p) mod 12` where `a·6 ≡ 0 (mod 12)` — `a ∈ {0, 4, 8}` — or depends on
`s` only through `s mod 2` or `s mod 3`, both of which are unchanged by adding
6. The part index `p` is identical because the degree within the sign is
identical. So the two nodes land on the same target in all seven.

So a D-16, D-20, D-24, D-30, D-40 or D-45 chart **always** shows `Ra` and `Ke`
sharing a compartment, and a D-2 chart always shows them sharing one of its two.
The renderer must not treat that as an anomaly. It is also a strong test: any
implementation that puts them anywhere else in those seven is wrong.

This holds only because the app uses `Ketu = Rahu + 180°`. Software that
computes a separate true Ketu would break it by a few arcseconds and could
occasionally straddle a boundary in D60.

### 4.3 How many signs a varga can occupy

Verified by exhaustive enumeration over all twelve rashis and all parts:

| Varga | Occupiable rashis |
|---|---|
| **D2** | **2** — Karka and Simha only |
| **D30** | **10** — Mesha, Vrishabha, Mithuna, Kanya, Tula, Vrishchika, Dhanu, Makara, Kumbha, Meena. **Karka and Simha never** |
| all others | 12 |

D2 and D30 are therefore the two charts where a twelve-compartment kundali is
guaranteed to be mostly empty: ten empty compartments in D2, two in D30. Both
are structural, not a data problem, and both need saying on the surface if those
charts are ever drawn.

BPHS states the D30 restriction itself, in the course of explaining why the Sun
and Moon are a special case there:

> "Nextly, a brief clarification is required about the Sun and Moon not having
> own Trimsamsas. The Sun can occupy Aries in Trimsamsas and the Moon can be in
> Taurus in Venusian Trimsamsa. **They cannot be in Leo or Cancer in Trimsamsa
> charts.** Hence only exaltation will apply to them in Trimsamsa."
> — Santhanam's notes to BPHS ch. 7

Karka and Simha are the Moon's and Sun's own signs, and the D30 lords are the
five planets other than the luminaries — hence the gap.

### 4.4 The variants are not sixteen unrelated arguments — they are four axes

Surveying what the software actually implements makes the disagreement
tractable. Almost every named variant of almost every varga is one of four
moves applied to the Parashari rule.

| Axis | What it does | Named instances |
|---|---|---|
| **Parivritti / cyclic** | ignore the per-rashi start sign; run the parts continuously from Mesha, `(n·s + p) mod 12` | Parivritti Dwaya (D2), Parivritti Traya (D3), "Continuous" in Maitreya, JHora's generic custom D-N |
| **Even-sign reversal** | keep the start sign, but run the parts of an even sign backwards | Uma-Shambhu (D2), "Parasara parivritti even reverse" (D3), and — per JHora 7.63 — new definitions of D2, D3, D12, D16, D20, D27 and more, "mostly related to reversal of divisions in an even sign" |
| **Somanatha parivritti alternate** | odd signs forward from Mesha, even signs backward from Meena (§7.3) | offered for nearly every varga in JHora's reimplementation |
| **Different start signs entirely** | a genuinely different mapping, not a transformation of the Parashari one | Jagannatha drekkana, Kashinatha hora, Raman hora, Kalachakra navamsa, Krishna Mishra navamsa |

Two consequences for the implementation:

- The first three axes are **flags on one function**, not separate functions. If
  variants are ever offered, that is the shape to build.
- The fourth is not, and each member of it needs its own rule and its own
  citation. Only the Jagannatha drekkana and the Kashinatha hora are sourced
  well enough here to implement.

**First pass takes none of them.** Parashari only, named on the surface.

---

---

## 5. The varga lagna

**The same rule, applied to the ascendant's longitude.** This is cited, not
assumed, because it was the one point where the obvious answer could have been
wrong.

Four statements, from three sources:

1. Narasimha Rao defines the input to every division as "planets, upagrahas,
   lagna or special lagnas — basically a physical or a mathematical point in the
   zodiac that has a longitude associated with it", and again: "we need to know
   the rasis occupied by planets, upagrahas, lagna and special lagnas to draw
   any chart."
2. Sanjay Rath works one: "Lagna at 14° Pisces is in second Drekkana and is
   mapped into Cancer the fifth house from Pisces." Meena is index 11, second
   drekkana is `p = 1`, `(11 + 4·1) mod 12 = 3` = Karka. The lagna took the
   graha rule unchanged.
3. Santhanam, on marking a chart: "These positions can also be marked in a
   zodiacal diagram for the planets **and the ascendant** for an easy grasp" —
   and, on the varga-strength names, "the planet **or the ascendant, as the case
   may be**, has obtained 12 good Vargas in the Shodasa Varga".
4. Rath again, on D-30: "The positions of the planets **and the Lagna** in the
   Rasi chart are used to determine the Trimsamsa occupied by them." Same rule,
   a different varga, so it is not a property of the drekkana alone.

No source found states a different rule for the ascendant, in any varga. There
is no separate "varga lagna formula".

The consequence for us: `Chakra::lagna.longitude` is already carried on the
payload, so a varga chart needs no new engine call — only the same division
applied to one more longitude. The house numbering of a varga chart then counts
from the varga lagna's rashi exactly as `chakra.rs` counts from the D1 lagna.

---

## 6. Precision

### 6.1 Segment sizes, and how much time each is worth

The lagna moves about **0.25° per minute of clock time** — a degree every four
minutes (`chakra.rs`, `kundali.md` §1.2). That is **15′ of longitude per minute
of time**, or **1′ per 4 seconds**.

| Varga | Segment (°) | Segment (arcmin) | Lagna crosses it in | 1′ of error, as % of a segment |
|---|---|---|---|---|
| D1 | 30°00′00″ | 1800 | 120 min | 0.06% |
| D2 | 15°00′00″ | 900 | 60 min | 0.11% |
| D3 | 10°00′00″ | 600 | 40 min | 0.17% |
| D4 | 7°30′00″ | 450 | 30 min | 0.22% |
| D30 | 5°00′00″ (smallest) | 300 | 20 min | 0.33% |
| D7 | 4°17′08.57″ | 257.14 | 17.1 min | 0.39% |
| D9 | 3°20′00″ | 200 | 13.3 min | 0.50% |
| D10 | 3°00′00″ | 180 | 12 min | 0.56% |
| D12 | 2°30′00″ | 150 | 10 min | 0.67% |
| D16 | 1°52′30″ | 112.5 | 7.5 min | 0.89% |
| D20 | 1°30′00″ | 90 | 6 min | 1.11% |
| D24 | 1°15′00″ | 75 | 5 min | 1.33% |
| D27 | 1°06′40″ | 66.67 | 4.4 min | 1.50% |
| D40 | 0°45′00″ | 45 | 3 min | 2.22% |
| D45 | 0°40′00″ | 40 | 2.7 min | 2.50% |
| D60 | 0°30′00″ | 30 | 2 min | 3.33% |

D30's entry is its *smallest* segment; the 8° segment is 480′ and takes 32
minutes.

### 6.2 The clock is not the limiting factor

The chart is cast for the present instant. At 1′ per 4 seconds of clock time, a
system clock a second out moves the lagna 15″ — one hundred and twentieth of
the D60 segment. macOS keeps the clock inside that by orders of magnitude. Not a
concern for any varga.

The refresh cadence is a different matter: a D60 lagna is stale after **2
minutes**, a D45 after 2.7, a D40 after 3. `kundali.md` §5.5 sets the panel's
refresh; whatever it is, a drawn D60 is a claim about a two-minute window.

### 6.3 The location *is* the limiting factor

`TODO.md` E5: the bundled place list is `zone.tab`, **418 places at degrees and
arcminutes, about 1.9 km**.

The arithmetic that matters is how geographic longitude maps to the lagna. One
degree of geographic longitude shifts local sidereal time by 4 minutes, and the
lagna advances 360° per 360° of sidereal time. So **averaged over a day, 1′ of
geographic longitude moves the lagna by 1′.** (Instantaneously the ratio swings
either side of 1 — the ascendant does not advance uniformly, and the swing grows
with latitude — so treat 1:1 as the order of magnitude, not as a constant.)

Two questions follow, and they have different answers.

**How far wrong must the place be to move the varga lagna a whole segment?**
Segment in arcminutes ≈ arcminutes of geographic longitude ≈
`arcmin × 1.855 × cos φ` km.

| Varga | Segment (arcmin) | Longitude error for a full segment, at the equator | at 28.6° N |
|---|---|---|---|
| D1 | 1800 | 3339 km | 2932 km |
| D2 | 900 | 1670 km | 1466 km |
| D3 | 600 | 1113 km | 977 km |
| D4 | 450 | 835 km | 733 km |
| D30 | 300 | 557 km | 489 km |
| D7 | 257.14 | 477 km | 419 km |
| D9 | 200 | 371 km | 326 km |
| D10 | 180 | 334 km | 293 km |
| D12 | 150 | 278 km | 244 km |
| D16 | 112.5 | 209 km | 183 km |
| D20 | 90 | 167 km | 147 km |
| D24 | 75 | 139 km | 122 km |
| D27 | 66.67 | 124 km | 109 km |
| D40 | 45 | 83 km | 73 km |
| D45 | 40 | 74 km | 65 km |
| D60 | 30 | 56 km | 49 km |

By that measure the 1.9 km grid is harmless everywhere: even D60 needs a ~50 km
error before the answer is *typically* wrong.

**How often does the quantisation flip the answer?** The `zone.tab` grid rounds
to the arcminute, so the stored longitude is within ±0.5′ of the place it names.
For a segment of width `w` arcminutes and an error `e`, the chance the true and
stored longitudes fall in different segments is `≈ e / w`:

| Varga | `w` (arcmin) | `e = 0.5′` — grid rounding, 0.9 km | `e = 5′` — wrong suburb, ~9 km |
|---|---|---|---|
| D1 | 1800 | 0.03% | 0.3% |
| D2 | 900 | 0.06% | 0.6% |
| D3 | 600 | 0.08% | 0.8% |
| D4 | 450 | 0.11% | 1.1% |
| D30 | 300 | 0.17% | 1.7% |
| D7 | 257.14 | 0.19% | 1.9% |
| D9 | 200 | 0.25% | 2.5% |
| D10 | 180 | 0.28% | 2.8% |
| D12 | 150 | 0.33% | 3.3% |
| D16 | 112.5 | 0.44% | 4.4% |
| D20 | 90 | 0.56% | 5.6% |
| D24 | 75 | 0.67% | 6.7% |
| D27 | 66.67 | 0.75% | 7.5% |
| D40 | 45 | 1.1% | 11.1% |
| D45 | 40 | 1.25% | 12.5% |
| D60 | 30 | 1.7% | 16.7% |

Conclusion, stated plainly:

- The **grid** — arcminute rounding, ±0.9 km — is not the limiting factor for
  any varga. Worst case is D60 at 1.7%, and even that is below the D1 chart's
  own exposure to a stale minute of clock time.
- The **coverage** is the limiting factor. E5's own example — a user in Howrah
  who can only choose Kolkata — is a ~10 km error. At that error the failure
  rate passes **5% for every varga finer than D16**, and **10% for D40, D45 and
  D60**.
- So: E5 does not block the first-pass five. D9 at a 10 km error is wrong 2.5%
  of the time, which is the same order as the D1 lagna's existing exposure. It
  does block **D40, D45 and D60**, where one chart in eight or worse would be
  wrong for a reason the user cannot see and cannot fix.

### 6.4 The ayanamsa moves every boundary at once

Changing the ayanamsa shifts every sidereal longitude by the same amount, and so
shifts every varga boundary. A change of `x` arcminutes is `x/w` of a segment.
Since the ayanamsas Swiss Ephemeris offers differ from each other by amounts on
the order of a degree — 60′ — a switch between two of them **relocates every
body in every varga finer than D3**, and moves D40, D45 and D60 by more than a
whole segment.

The exact spread between Lahiri and the other ayanamsas the app offers is
**unverified here** (§9). It is directly measurable with `Engine::ayanamsa` and
should be measured rather than quoted.

This is not a defect. It is the reason **D-003** makes the ayanamsa an explicit,
named default rather than an implementation detail, and the reason a varga chart
must be as clear about its ayanamsa as `chakra.rs` is about its place.

### 6.5 Two implementation notes on `p`

- Compute `p = floor(d * n / 30)`, multiplying before dividing. **Four vargas
  have boundaries that are not representable in binary floating point** — D7
  (30/7), D9 (10/3), D27 (10/9) and D45 (2/3). Checked: at the first boundary of
  each of those four, `d * n / 30` evaluates to exactly `1.0` in f64 when `d` is
  built as `deg + min/60 + sec/3600`, so `floor` lands on the intended side.
- Clamp: `p = min(p, n − 1)`. A `d` that rounds to exactly 30.0 must not produce
  `p = n`.
- Boundary test vectors are given per varga in §7 — the value one arcsecond
  below the first boundary and the value on it. They exist precisely because a
  body sitting exactly on a boundary is the case floating point gets wrong.

---

## 7. The sixteen

Each section gives the BPHS verse, the formula, worked examples a test can be
written from, known variants, and anything specific to the nodes.

The BPHS text used throughout is **Brihat Parasara Hora Sastra, Vol. I, English
translation, commentary, annotation and editing by R. Santhanam, Ranjan
Publications, New Delhi** — chapter 6, "The Sixteen Divisions of a Sign", verses
2–41. Two independently scanned copies were consulted (§10); where the OCR of
one is unreadable the other supplies the text, and where both are readable they
agree.

The names, per BPHS ch. 6 v. 2–4:

> "**NAMES OF THE 16 VARGAS:** Lord Brahma has described 16 kinds of Vargas
> (Divisions) for each sign. Listen to those. The same are: Rasi, Hora,
> Drekkana, Chathurthamsa, Sapthamamsa, Navamsa, Dasamamsa, Dvadasamsa,
> Shodasamsa, Vimsamsa, Chaturvimsamsa, Sapthavimsamsa, Trimsamsa, Khavedamsa,
> Akshavedamsa and Shashtiamsa."

---

### 7.1 D1 — Rashi

| | |
|---|---|
| Parts | 1 |
| Segment | 30° |
| Equal | yes |
| Formula | `s` |
| Occupiable | 12 |

The chart the app already draws. Included for completeness: BPHS treats the
rashi as the first of the sixteen vargas, and Narasimha Rao notes "Rasi chart is
simply a special case of divisional charts. If we divide each rasi into just one
part (i.e. in effect, no division), we get rasi chart."

Nothing to implement. It is the identity case of the same function.

---

### 7.2 D2 — Hora

| | |
|---|---|
| Parts | 2 |
| Segment | 15° = 900′ |
| Equal | yes |
| Occupiable | **2** |

> "**RASI AND HORA:** The Rasi owned by a planet is called its Kshetra (one
> sign). The first half of an odd sign is the Hora ruled by the Sun while the
> second half is the Hora of the Moon. The reverse is true in the case of an
> even sign. Half of Rasi is called Hora. These are totally 24 counted from
> Aries and repeat twice (at the rate of 12) in the whole of the zodiac."
> — BPHS ch. 6 v. 5–6

**The verse names lords, not signs.** The step from "the Sun's hora" to "Simha"
is not in the verse; it is universal in practice and stated explicitly by
Narasimha Rao: "Sun's hora means Leo in hora chart and Moon's hora means Cancer
in hora chart," producing "a hora chart that has all the planets in two signs —
Cancer and Leo."

**Rule (Parashari, standard):**

```
odd sign  (s % 2 == 0):  p = 0 → Simha (4);  p = 1 → Karka (3)
even sign (s % 2 == 1):  p = 0 → Karka (3);  p = 1 → Simha (4)
```

**Worked examples**

| Longitude | Rashi, degree | Half | D2 |
|---|---|---|---|
| 7.0000° | Mesha 7°00′00″ (odd) | 1st | **Simha** |
| 22.0000° | Mesha 22°00′00″ (odd) | 2nd | **Karka** |
| 37.0000° | Vrishabha 7°00′00″ (even) | 1st | **Karka** |
| 52.0000° | Vrishabha 22°00′00″ (even) | 2nd | **Simha** |
| 14.99972° | Mesha 14°59′59″ | 1st | **Simha** |
| 15.00000° | Mesha 15°00′00″ | 2nd | **Karka** |

**Variants — this is the most disputed of the sixteen.** Jagannatha Hora ships
"**six different variations of hora (D-2) charts (including Kashinatha Hora)**",
and unusually all six are named, four of them by JHora's own author, who has
written two articles arguing about which reading of BPHS ch. 6 v. 5–6 is right.

| Variant | Rule | Occupies | Held by |
|---|---|---|---|
| **Parashari / Cancer–Leo** | as above | 2 signs | BPHS as universally read. The only one implemented by Maitreya's "Parasara" mode and by VedAstro |
| **Parivritti Dwaya** (bicyclical) | cyclic: "the two parts of Aries go into Aries and Taurus; the two parts of Taurus go into Gemini and Cancer; the two parts of Gemini go into Leo and Virgo […] we cyclically go around the zodiac twice", i.e. `(2s + p) mod 12` | 12 | Narasimha Rao: it "does not satisfy the basic criterion of Parasara" and shows "family support to one's activities. It does not show wealth" |
| **Kashinatha** | take the hora by the Parashari half-rule, then place the body in whichever sign **its own rashi's lord** owns that matches: day-strong (§3.4) for the Sun's hora, night-strong for the Moon's. Worked: "Jupiter is at 25°16′ in Taurus […] second half […] goes into Sun's hora. Thus it should go into the day-strong sign owned by Venus (lord of Taurus). Thus, Jupiter is placed in Libra" | up to 12 | Named for Pt. Kashinath Rath; attributed to the tradition of Sri Achyuta Dasa. Rao's own preferred reading for wealth |
| **Uma-Shambhu** | the 24 horas are taken in the order "the first half of Aries, the second half of Aries, the second half of Taurus, the first half of Taurus (reversed for Taurus as it is an even sign), the first half of Gemini, the second half of Gemini […] then mapped to 2 cycles of the Zodiac" — a cyclic hora with the two halves of an even sign swapped | 12 | Narasimha Rao, derived from Krishna Mishra's navamsa reversal. **The default in the PyJHora reimplementation** |
| **Raman (1st/11th, day/night)** | "The first half of an odd sign is ruled by the lord of that sign and the planets go to the Day sign of this planet. The second half of an odd sign is ruled by the lord of the 11th from that sign and the planet belongs to the Night sign of this planet." Even signs reversed. Explicit exception: "For Moon, both day and night signs are Cancer and for Sun, both day and night signs are Leo" | up to 12 | B.V. Raman's *Suprajarama* tradition |
| **Somanatha parivritti alternate** | odd signs run forward from Mesha, even signs run backward from Meena: Mesha → (Mesha, Vrishabha); Vrishabha → (Meena, Kumbha); Mithuna → (Mithuna, Karka); Karka → (Makara, Dhanu) … | 12 | The same "alternate" family as the Somanatha drekkana (§7.3) |

Rao's *Parasara's Hora Chart Decoded* enumerates the competing readings as View
1 (Cancer/Leo), View 2 (day/night-strong signs — the Kashinatha family), View 3
(1st/11th — the Raman family) and View 4 (parivritti dwaya), and objects to each
in turn. **That article is the best single source for this disagreement, and it
does not settle it.**

Sanjay Rath describes the hora on a different basis again — a division of the
whole zodiac into a solar half and a lunar half rather than of each rashi: "The
solar half or Surya Hora included the six signs in the zodiacal order from Leo
to Capricorn and the lunar half or Chandra Hora included the six signs from
Cancer to Aquarius in the reverse order." Whether that is a seventh D-2 mapping
or a different object is **unverified** (§9).

**Note on Kashinatha and Raman:** both are undefined, or need a special case,
where a rashi's lord owns only one sign. Raman's source states the exception
("for Moon, both day and night signs are Cancer and for Sun, both day and night
signs are Leo"); no equivalent statement was found for Kashinatha, so the Simha
and Karka cases there are **unverified**.

**Nodes:** Rahu and Ketu always share a hora under the Parashari rule (§4.2).

**If D2 is ever drawn:** ten of the twelve compartments are permanently empty
under the Parashari rule. A chart format designed for a spread of bodies will
look broken. That, plus six live variants of which the two best-argued are not
the standard one, is a reason to decide *whether* to offer D2 before deciding
how.
---

### 7.3 D3 — Drekkana

| | |
|---|---|
| Parts | 3 |
| Segment | 10° = 600′ |
| Equal | yes |
| Occupiable | 12 |

> "**DECANATE:** One third of a Rasi is called Drekkana (decanate). These are
> totally 36, counted from Aries (to Pisces), repeating thrice at the rate of 12
> per round. The 1st, 5th and the 9th Rasis from a sign are its three decanates
> […] *Notes:* Each Rasi has three decanates or Drekkanas. The first one is
> ruled by the lord of the very sign. The second one belongs to the planet that
> rules the 5th from the sign in question. The lord of the 9th from the sign in
> question is the lord of the 3rd decanate. Each decanate is 10 degrees in
> length."
> — BPHS ch. 6 v. 7–8

Corroborated by Narasimha Rao: "Bodies in the first 10° of a rasi are placed in
drekkana chart in the same rasi. Bodies in the middle 10° of a rasi are placed
in drekkana chart in the 5th from the rasi. Bodies in the last 10° of a rasi are
placed in drekkana chart in the 9th from the rasi."

**Why 1st/5th/9th:** they are the trines, which are the same-element signs
(§3.3). A drekkana never changes a body's element.

**Rule:** `(s + 4p) mod 12`

**Worked examples**

| Longitude | Rashi, degree | Part | D3 |
|---|---|---|---|
| 5.0000° | Mesha 5°00′00″ | 1 of 3 | **Mesha** |
| 15.0000° | Mesha 15°00′00″ | 2 of 3 | **Simha** |
| 25.0000° | Mesha 25°00′00″ | 3 of 3 | **Dhanu** |
| 55.0000° | Vrishabha 25°00′00″ | 3 of 3 | **Makara** |
| 9.99972° | Mesha 9°59′59″ | 1 of 3 | **Mesha** |
| 10.00000° | Mesha 10°00′00″ | 2 of 3 | **Simha** |

Narasimha Rao's own worked example, as a third check: Mithuna 3° → Mithuna;
Mithuna 19° → Tula; Mithuna 21° → Kumbha. All three reproduce.

**Variants.** JHora ships "**four different variations of D-3 charts**", and
four are named consistently across sources: Parashari, **Jagannatha**,
**Somanatha**, **Parivritti-traya**.

**Jagannatha drekkana — sourced, and it is not the Parashari.**

> "The 3 parts of all fiery signs are mapped to Ar, Le and Sg. The 3 parts of
> all earthy signs are mapped to Cp, Ta, Vi. The 3 parts of all airy signs are
> mapped to Li, Aq and Ge. The 3 parts of all watery signs are mapped to Cn, Sc
> and Pi."
> — Narasimha Rao, Sri Jagannatha Jyotish discussion articles

As a formula: `(9·(s mod 4) + 4p) mod 12`. Both charts keep a body inside its
own element; they order the three signs of that element differently.

Two checks against the same source's own comparisons, both reproducing:

| Case | Parashari | Jagannatha |
|---|---|---|
| Simha 10°–20° (`s=4, p=1`) | `4 + 4 = 8` → **Dhanu** | `9·0 + 4 = 4` → **Simha** |
| Meena 20°–30° (`s=11, p=2`) | `11 + 8 = 19 ≡ 7` → **Vrishchika** | `9·3 + 8 = 35 ≡ 11` → **Meena** |

The source's own words: "10-20 deg of Leo is mapped to Sg in Parasara drekkana
and to Le in Jagannatha drekkana"; "If you use Parasara drekkana, 20-30 deg in
Pisces falls in Scorpio […] If you use Jagannatha drekkana, 20-30 deg in Pisces
falls in Pisces." Both agree with the formula. The same source notes that
movable signs give the same answer under both — which is true, because for a
movable sign `9·(s mod 4) ≡ s (mod 12)`.

**Somanatha drekkana — sourced, and the formula checks out.** "Drekkanas run
from Aries directly for the odd signs, while they run from Pisces in the reverse
direction for the even signs." Spelled out: Mesha → (Mesha, Vrishabha,
Mithuna); Vrishabha → (Meena, Kumbha, Makara); Mithuna → (Karka, Simha, Kanya);
Karka → (Dhanu, Vrishchika, Tula); and so on, the odd signs taking successive
blocks of three forward from Mesha and the even signs successive blocks of three
backward from Meena.

As a formula, for any division of `n` parts — this is the generic "Somanatha
parivritti alternate" family, which JHora and its reimplementation offer for
almost every varga:

```
odd sign  (s % 2 == 0):  (n·(s / 2) + p) mod 12
even sign (s % 2 == 1):  (11 − n·((s − 1) / 2) − p) mod 12
```

Checked against the author's own comparison, "20-30 deg in Pisces falls in
Libra": Meena is `s = 11`, even, `(s−1)/2 = 5`, `p = 2`, so
`(11 − 3·5 − 2) mod 12 = (−6) mod 12 = 6` = **Tula**. Reproduces. Checked again
at n = 2 against a hora enumeration from the same family — Mesha → (Mesha,
Vrishabha), Vrishabha → (Meena, Kumbha), Mithuna → (Mithuna, Karka), Karka →
(Makara, Dhanu) — all four reproduce.

**Parivritti-traya (tri-cyclical) — sourced.** Three continuous cycles of the
zodiac: Mesha → (Mesha, Vrishabha, Mithuna); Vrishabha → (Karka, Simha, Kanya);
Mithuna → (Tula, Vrishchika, Dhanu) … i.e. `(3s + p) mod 12`, the continuous
form of §4.1.

**A fifth exists.** JHora's release notes for 7.63 record "one new definition of
D-2, D-3, D-12, D-16, D-20 and D-27 […] The alternative definitions are mostly
related to reversal of divisions in an even sign" — so the four named above are
JHora 7.0's four, and there is now a Parashari-with-even-sign-reversal as well.
See §4.4.

The Parashari drekkana is the default in JHora, in Maitreya and in VedAstro, and
is what every general reference describes.

**Nodes:** always the 7th from each other, as in D1.

---

### 7.4 D4 — Chaturthamsa

| | |
|---|---|
| Parts | 4 |
| Segment | 7°30′ = 450′ |
| Equal | yes |
| Occupiable | 12 |

> "**CHATURTHAMSA:** The lords of the 4 angles from a sign are the rulers of
> respective Chaturthamsa of a Rasi commencing from Aries. Each Chathurthamsa is
> one fourth of a Rasi. […] *Notes:* Each Chaturthamsa is one fourth of a sign
> or 7°30′. The 1st, 2nd, 3rd and 4th Chaturthamsas are ruled respectively by
> the same sign, the 4th, 7th and 10th signs therefrom. […] *Example:* The four
> Chaturthamsas of Aries are respectively ruled by Aries, Cancer, Libra and
> Capricorn."
> — BPHS ch. 6 v. 9

Corroborated by Narasimha Rao, who states the arcs explicitly: "planets in
0°–7.5° in a rasi go into 1st from that rasi; planets in 7.5°–15° go into 4th
from that rasi; planets in 15°–22.5° go into the 7th from that rasi; and,
planets in the 22.5°–30° go into the 10th from that rasi." He adds the aliases
**Chaturamsa** and **Turyamsa**.

**Why 1/4/7/10:** the kendras. A chaturthamsa keeps a body in the same
quadruplicity — movable stays movable, fixed stays fixed, dual stays dual.

**Rule:** `(s + 3p) mod 12`

**Worked examples**

| Longitude | Rashi, degree | Part | D4 |
|---|---|---|---|
| 3.0000° | Mesha 3°00′00″ | 1 of 4 | **Mesha** |
| 20.0000° | Mesha 20°00′00″ | 3 of 4 | **Tula** |
| 44.0000° | Vrishabha 14°00′00″ | 2 of 4 | **Simha** |
| 53.0000° | Vrishabha 23°00′00″ | 4 of 4 | **Kumbha** |
| 7.49972° | Mesha 7°29′59″ | 1 of 4 | **Mesha** |
| 7.50000° | Mesha 7°30′00″ | 2 of 4 | **Karka** |

**Variants.** JHora ships "**two different variations of D-4 charts**". The
second is **unverified** (§9).

**Nodes:** always the 7th from each other.

---

### 7.5 D7 — Saptamsa

| | |
|---|---|
| Parts | 7 |
| Segment | 30/7 = 4°17′08.571″ ≈ 257.14′ |
| Equal | yes |
| Occupiable | 12 |

> "**SAPTHAMAMSA:** The Sapthamamsa (one seventh of a Rasi) counting commences
> from the same sign in the case of an odd sign. It is from the seventh sign
> thereof while an even sign is considered. […] *Notes:* Each sign is made in 7
> equal parts of 4°17′8.57″ which is called Saptamamsa. […] *Example:* For
> Aries, these divisions are Aries, Taurus, Gemini etc., while for Taurus these
> are Scorpio, Sagittarius, Capricorn etc."
> — BPHS ch. 6 v. 10–11

Corroborated verbatim in effect by Narasimha Rao: "starting from the rasi
itself, if it is an odd rasi, or starting from the 7th sign from it, if it is an
even rasi", with the same segment size to the same precision.

**Rule:** `(7s + p) mod 12` — equivalently `(s + 6·(s mod 2) + p) mod 12`. Both
forms are given because the first is the continuous-division form (§4.1) and the
second is the verse's form; they agree for every `s`.

**Worked examples**

| Longitude | Rashi, degree | Part | D7 |
|---|---|---|---|
| 5.0000° | Mesha 5°00′00″ (odd) | 2 of 7 | **Vrishabha** |
| 35.0000° | Vrishabha 5°00′00″ (even) | 2 of 7 | **Dhanu** |
| 70.0000° | Mithuna 10°00′00″ (odd) | 3 of 7 | **Simha** |
| 169.0000° | Kanya 19°00′00″ (even) | 5 of 7 | **Karka** |
| 4.28556° | Mesha 4°17′08″ | 1 of 7 | **Mesha** |
| 4.28583° | Mesha 4°17′09″ | 2 of 7 | **Vrishabha** |

The last two are the reason §6.5 exists: the first boundary is at
4.285714285…°, and it is the only one of the sixteen vargas whose boundaries are
not exactly representable.

Narasimha Rao's own examples reproduce: Mithuna 10° → Simha; Kanya 19° → Karka.

**Variants.** None found. JHora lists no D-7 variations on its feature page.

**Nodes:** always the 7th from each other.

---

### 7.6 D9 — Navamsa

| | |
|---|---|
| Parts | 9 |
| Segment | 3°20′ = 200′ |
| Equal | yes |
| Occupiable | 12 |

> "**NAVAMSA:** The Navamsa calculations are for a movable sign from there
> itself, for a fixed sign from the 9th thereof and for a dual sign from the 5th
> thereof. […] *Notes:* Navamsa is 1/9th part of a sign or 3°20′. The 9 Navamsas
> in order commence from the same sign for a movable sign, from the 9th for a
> fixed sign and from 5th for a dual sign. For example: the Navamsas of Aries
> are counted from Aries itself; from Capricorn for Taurus and from Libra for
> Gemini."
> — BPHS ch. 6 v. 12

Narasimha Rao states the **same rule in different words**: "Bodies in the 9
parts of a rasi go into the 9 rasis starting from Ar, Cp, Li or Cn, based on
whether the rasi is a fiery, earthy, airy or watery sign."

**These are the same rule, and that is worth showing rather than asserting:**

| Rashi | BPHS (movable/fixed/dual) | Rao (element) | Start |
|---|---|---|---|
| Mesha | movable → itself | fiery → Mesha | Mesha |
| Vrishabha | fixed → 9th = Makara | earthy → Makara | Makara |
| Mithuna | dual → 5th = Tula | airy → Tula | Tula |
| Karka | movable → itself | watery → Karka | Karka |
| Simha | fixed → 9th = Mesha | fiery → Mesha | Mesha |
| Kanya | dual → 5th = Makara | earthy → Makara | Makara |

and so on for all twelve. Both reduce to `9s mod 12`. Two independent
statements, one rule.

**Rule:** `(9s + p) mod 12`

**Rao's alias:** Dharmamsa. "It is the most popular chart after rasi chart and
some astrologers simply refer to it as 'Amsa' (division)."

**Worked examples** — one per class, because the class is the branch:

| Longitude | Rashi, degree | Class | Part | D9 |
|---|---|---|---|---|
| 5.0000° | Mesha 5°00′00″ | movable, fiery | 2 of 9 | **Vrishabha** |
| 35.0000° | Vrishabha 5°00′00″ | fixed, earthy | 2 of 9 | **Kumbha** |
| 65.0000° | Mithuna 5°00′00″ | dual, airy | 2 of 9 | **Vrishchika** |
| 95.0000° | Karka 5°00′00″ | movable, watery | 2 of 9 | **Simha** |
| 229.0000° | Vrishchika 19°00′00″ | fixed, watery | 6 of 9 | **Dhanu** |
| 3.33306° | Mesha 3°19′59″ | | 1 of 9 | **Mesha** |
| 3.33333° | Mesha 3°20′00″ | | 2 of 9 | **Vrishabha** |

Rao's examples reproduce: Mithuna 11° → Makara; Vrishchika 19° → Dhanu.

**Cross-check available for free:** a navamsa boundary is a nakshatra-pada
boundary. `zodiac.rs::pada` already computes padas; the 108 navamsas and the
108 padas are the same 3°20′ grid. Any disagreement between them is a bug in one
of the two.

**Variants.** JHora ships "**three different variations of D-9 charts**" on its
feature page, which is surprising for the varga usually treated as settled — and
its own release notes then contradict that count. Named:

| Variant | Status |
|---|---|
| **Parashari** — the rule above | default everywhere. Maitreya and VedAstro implement only this one |
| **Kalachakra navamsa** | named in JHora 7.32's release note; rule **unverified** here |
| **Krishna Mishra navamsa** | added in JHora 7.32. Rule **unverified**; JHora's author says it "is biased towards some signs, i.e. some signs occur 12 times, some signs occur 9 times and some signs occur only 6 times" |
| **Uniform Krishna Mishra navamsa** | added in JHora 7.4 as the author's "independent interpretation of Krishna Mishra's verses", so that "all signs occur 9 times". The rule *is* stated, in JHora 7.5: "the 9 divisions of Ta are mapped to Vi, Le, …, Aq, Cp, instead of the regular zodiacal order Cp, Aq, …, Le, Vi" — i.e. the Parashari start sign with the parts of an even sign run backwards (§4.4, axis 2) |

Note the count: 7.32 says "thus, three navamsa options are available"; 7.4 adds a
fourth; the feature page still says three. **JHora's own documentation
disagrees with itself**, and this document does not resolve it.

Every general reference consulted gives the Parashari rule and no other.

**Nodes:** always the 7th from each other.

---

### 7.7 D10 — Dasamsa

| | |
|---|---|
| Parts | 10 |
| Segment | 3° = 180′ |
| Equal | yes |
| Occupiable | 12 |

> "**DASAMSA:** Starting from the same sign for an odd sign and from the 9th
> with reference to an even sign, the 10 Dasamsas each of 3° are reckoned. […]
> *Example:* For odd signs, the Dasamsas are the 10 signs counted successively
> therefrom. For even signs, these fall in 10 successive signs counted from the
> 9th thereof."
> — BPHS ch. 6 v. 13–14

Corroborated by Narasimha Rao: "starting from the rasi itself or the 9th from
it, based on whether the rasi is an odd or even sign." Aliases: Dasamaamsa,
Karmamsa, Swargamsa.

**Rule:** `(s + 8·(s mod 2) + p) mod 12`

Note that this is **not** the continuous form: `10s mod 12` would give Kumbha as
Vrishabha's start, and the verse says Makara.

**Worked examples**

| Longitude | Rashi, degree | Part | D10 |
|---|---|---|---|
| 5.0000° | Mesha 5°00′00″ (odd) | 2 of 10 | **Vrishabha** |
| 35.0000° | Vrishabha 5°00′00″ (even) | 2 of 10 | **Kumbha** |
| 70.0000° | Mithuna 10°00′00″ (odd) | 4 of 10 | **Kanya** |
| 229.0000° | Vrishchika 19°00′00″ (even) | 7 of 10 | **Makara** |
| 2.99972° | Mesha 2°59′59″ | 1 of 10 | **Mesha** |
| 3.00000° | Mesha 3°00′00″ | 2 of 10 | **Vrishabha** |

Rao's examples reproduce: Mithuna 10° → Kanya; Vrishchika 19° → Makara.

**Variants.** None found.

**Nodes:** always the 7th from each other.

---

### 7.8 D12 — Dwadasamsa

| | |
|---|---|
| Parts | 12 |
| Segment | 2°30′ = 150′ |
| Equal | yes |
| Occupiable | 12 |

> "**DVADASAMSA:** The reckoning of the Dvadasamsa (one twelfth of a sign or 2½
> degrees each) commences from the same sign. […] *Notes:* Each Dvadasamsa is
> 2°30′ and the 12 divisions fall successively in the successive 12 signs from
> the sign in question. […] *Example:* The Dvadasamsas in Aries in order are:
> Aries, Taurus, Gemini, Cancer, Leo, Virgo, Libra, Scorpio, Sagittarius,
> Capricorn, Aquarius and Pisces."
> — BPHS ch. 6 v. 15

Corroborated by Narasimha Rao: "Bodies in the 12 parts of a rasi go into the 12
rasis starting from the rasi itself."

The simplest of the sixteen: no odd/even branch, no class branch, no element
branch. Every rashi's twelve parts run through all twelve signs starting from
itself.

**Rule:** `(s + p) mod 12`

**Worked examples**

| Longitude | Rashi, degree | Part | D12 |
|---|---|---|---|
| 5.0000° | Mesha 5°00′00″ | 3 of 12 | **Mithuna** |
| 56.0000° | Vrishabha 26°00′00″ | 11 of 12 | **Meena** |
| 229.0000° | Vrishchika 19°00′00″ | 8 of 12 | **Mithuna** |
| 2.49972° | Mesha 2°29′59″ | 1 of 12 | **Mesha** |
| 2.50000° | Mesha 2°30′00″ | 2 of 12 | **Vrishabha** |

Rao's examples reproduce: Mithuna 11° → Tula; Vrishchika 19° → Mithuna.

**Variants.** None found. This is the one varga where no source consulted
offered an alternative.

**Nodes:** always the 7th from each other.

---

### 7.9 D16 — Shodasamsa (Kalamsa)

| | |
|---|---|
| Parts | 16 |
| Segment | 1°52′30″ = 112.5′ |
| Equal | yes |
| Occupiable | 12 |

> "**SHODASAMSA:** Starting from Aries for a movable sign, from Leo for a fixed
> sign and from Sagittarius for a dual sign, the 16 Shodasamsas (16th part of a
> sign i.e. of 1°52′30″) are regularly distributed. […] *Example:* The 16
> Shodasamsas for Aries or Cancer, or Libra or Capricorn (movable signs) are
> distributed to the 16 signs (12+4) commencing from Aries."
> — BPHS ch. 6 v. 16

Corroborated by Narasimha Rao: "starting from Ar, Le and Sg, based on whether
the rasi is movable, fixed or dual", with the useful note that "After going over
the 12 rasis from a rasi, we get the same rasi as the 13th rasi. So the 13th,
14th, 15th and 16th rasis from a rasi are simply the 1st, 2nd, 3rd and 4th
rasis" — i.e. the `mod 12` is in the source, not an implementation liberty.

**Trap:** D16 and D20 both branch on movable/fixed/dual, and their start signs
are **not** the same. D16 is Mesha / Simha / Dhanu. D20 is Mesha / Dhanu /
Simha. The fixed and dual starts are swapped between the two. Both BPHS and Rao
agree on both, independently, so this is a real distinction and not a
transcription error in one of them.

**Rule:** `(4s + p) mod 12` — the continuous form, equal to
`(4·(s mod 3) + p) mod 12`.

**Worked examples**

| Longitude | Rashi, degree | Class | Part | D16 |
|---|---|---|---|---|
| 5.0000° | Mesha 5°00′00″ | movable | 3 of 16 | **Mithuna** |
| 35.0000° | Vrishabha 5°00′00″ | fixed | 3 of 16 | **Tula** |
| 71.0000° | Mithuna 11°00′00″ | dual | 6 of 16 | **Vrishabha** |
| 229.0000° | Vrishchika 19°00′00″ | fixed | 11 of 16 | **Mithuna** |
| 1.87472° | Mesha 1°52′29″ | | 1 of 16 | **Mesha** |
| 1.87500° | Mesha 1°52′30″ | | 2 of 16 | **Vrishabha** |

Rao's examples reproduce: Mithuna 11° → Vrishabha; Vrishchika 19° → Mithuna.

**Variants.** None found.

**Nodes:** **always in the same rashi** (§4.2).

---

### 7.10 D20 — Vimsamsa

| | |
|---|---|
| Parts | 20 |
| Segment | 1°30′ = 90′ |
| Equal | yes |
| Occupiable | 12 |

> "**VIMSAMSA:** From Aries for a movable sign, from Sagittarius for a fixed
> sign and from Leo for a common sign — this is how the calculations of Vimsamsa
> (1/20th of a sign or 1°30′ each) are to commence."
> — BPHS ch. 6 v. 17–21

Corroborated by Narasimha Rao: "starting from Ar, Sg and Le, based on whether
the rasi is movable, fixed or dual."

See the trap noted in §7.9: the fixed and dual starts are the reverse of D16's.

**Rule:** `(8s + p) mod 12` — the continuous form, equal to
`(8·(s mod 3) + p) mod 12`.

**Worked examples**

| Longitude | Rashi, degree | Class | Part | D20 |
|---|---|---|---|---|
| 5.0000° | Mesha 5°00′00″ | movable | 4 of 20 | **Karka** |
| 35.0000° | Vrishabha 5°00′00″ | fixed | 4 of 20 | **Meena** |
| 71.0000° | Mithuna 11°00′00″ | dual | 8 of 20 | **Meena** |
| 229.0000° | Vrishchika 19°00′00″ | fixed | 13 of 20 | **Dhanu** |
| 1.49972° | Mesha 1°29′59″ | | 1 of 20 | **Mesha** |
| 1.50000° | Mesha 1°30′00″ | | 2 of 20 | **Vrishabha** |

Rao's examples reproduce: Mithuna 11° → Meena; Vrishchika 19° → Dhanu (his own
note that "the 13th from Sg is Sg itself" is the `mod 12`).

**Variants.** None found.

**Nodes:** **always in the same rashi.**

---

### 7.11 D24 — Chaturvimsamsa (Siddhamsa)

| | |
|---|---|
| Parts | 24 |
| Segment | 1°15′ = 75′ |
| Equal | yes |
| Occupiable | 12 |

> "**SIDDHAMSA:** The Siddhamsa (1/24th part of a sign or 1°15′ each)
> distribution commences from Leo and Cancer respectively for an odd sign and an
> even sign. […] *Notes:* Siddhamsa is also called Chaturvimsamsa, each being of
> a length of 1°15′, (24 in number in the whole of a sign). The successively
> distributed Siddhamsas commence from Leo for any odd sign and from Cancer for
> any even sign."
> — BPHS ch. 6 v. 22–23

Corroborated by Narasimha Rao: "starting from Le or Cn, based on whether the
rasi is odd or even."

This is the only varga whose start sign does not depend on the rashi at all
beyond its parity — every odd rashi starts at Simha, every even rashi at Karka.

**Rule:** `(4 − (s mod 2) + p) mod 12`

**Worked examples**

| Longitude | Rashi, degree | Parity | Part | D24 |
|---|---|---|---|---|
| 5.0000° | Mesha 5°00′00″ | odd | 5 of 24 | **Dhanu** |
| 35.0000° | Vrishabha 5°00′00″ | even | 5 of 24 | **Vrishchika** |
| 71.0000° | Mithuna 11°00′00″ | odd | 9 of 24 | **Mesha** |
| 229.0000° | Vrishchika 19°00′00″ | even | 16 of 24 | **Tula** |
| 1.24972° | Mesha 1°14′59″ | | 1 of 24 | **Simha** |
| 1.25000° | Mesha 1°15′00″ | | 2 of 24 | **Kanya** |

Rao's examples reproduce: Mithuna 11° → Mesha; Vrishchika 19° → Tula.

**Variants.** None found.

**Nodes:** **always in the same rashi.**

---

### 7.12 D27 — Bhamsa (Nakshatramsa, Saptavimsamsa)

| | |
|---|---|
| Parts | 27 |
| Segment | 30/27 = 1°06′40″ = 66.667′ |
| Equal | yes |
| Occupiable | 12 |

This varga carries **two errors found in the sources**, both recorded rather
than quietly corrected.

**The verse:**

> "**BHAMSA (NAKSHATRAMSA OR SAPTAVIMSAMSA):** The Bhamsa lords are respectively
> the presiding deities of the 27 Nakshatras […] These are for an odd sign.
> Count these deities in a reverse order for an even sign. **The Bhamsa
> distribution commences from Aries and other movable signs for all the 12
> signs.** *Notes:* One Bhamsa is of 1°6′40″ of arc and there are 27 such
> divisions in a sign."
> — BPHS ch. 6 v. 24–26

**Santhanam's own note, three paragraphs later:**

> "The Bhamsas (or Nakshatramsas or Sapthavimsamsas) are distributed from Aries
> for fiery signs, from Cancer for earthy signs, from Libra for airy signs and
> Capricorn for watery signs."

The verse says only that the start is always a movable sign; the note says which
one, by element. The two are consistent — Mesha, Karka, Tula and Makara *are*
the four movable signs — but the verse alone is not sufficient to compute the
chart. The element note is what every other source states, and it is what the
speculum in the same chapter tabulates.

Corroborated by Narasimha Rao: "starting from Ar, Cn, Li and Cp based on whether
the rasi is a fiery, earthy, airy or watery rasi."

**Rule:** `(3s + p) mod 12` — the continuous form, equal to
`(3·(s mod 4) + p) mod 12`.

**Error 1 — a worked example in a reference text is wrong.** Narasimha Rao's
Example 23 reads: "11° is in the 10th part […] Because Ge is an airy rasi,
counting starts from Li. **The 10th from Li is Le.**" Counting inclusively from
Tula: Tula, Vrishchika, Dhanu, Makara, Kumbha, Meena, Mesha, Vrishabha, Mithuna,
**Karka**. The 10th from Tula is Karka, not Simha. His rule is right; the count
in that one example is off by one. His second example in the same paragraph
(Vrishchika 19° → Mithuna) is correct and reproduces exactly.

This is recorded, not hidden, because it is precisely the failure mode the
"cited, not remembered" rule exists for — and it shows that a citation still has
to be *checked*, not merely quoted.

**Error 2 — the translator flags a disagreement with another classic.**
Santhanam's note continues:

> "I have on P. 31 of my English translation of SARAVALI given different
> calculation for Nakshatramsa. That source obviously is defective and I would
> prefer Parasara's version as given in our present text."

So **Saravali (Kalyana Varma) gives a different nakshatramsa rule.** What that
rule is is **unverified** here (§9) — only that the same translator judged it
defective and preferred BPHS. That judgement is his, not this document's, and if
D27 is ever shipped the disagreement should be named on the surface the way
D-026's node dispute is.

**Worked examples**

| Longitude | Rashi, degree | Element | Part | D27 |
|---|---|---|---|---|
| 5.0000° | Mesha 5°00′00″ | fiery → Mesha | 5 of 27 | **Simha** |
| 35.0000° | Vrishabha 5°00′00″ | earthy → Karka | 5 of 27 | **Vrishchika** |
| 71.0000° | Mithuna 11°00′00″ | airy → Tula | 10 of 27 | **Karka** |
| 95.0000° | Karka 5°00′00″ | watery → Makara | 5 of 27 | **Vrishabha** |
| 229.0000° | Vrishchika 19°00′00″ | watery → Makara | 18 of 27 | **Mithuna** |
| 1.11083° | Mesha 1°06′39″ | | 1 of 27 | **Mesha** |
| 1.11139° | Mesha 1°06′41″ | | 2 of 27 | **Vrishabha** |

The Mithuna 11° row is the one Rao gets wrong. It is included deliberately: a
test asserting **Karka** there is a test that would have caught the erratum.

**Nodes:** always the 7th from each other.

---

### 7.13 D30 — Trimsamsa

| | |
|---|---|
| Parts | **5, unequal** |
| Segments | 5°, 5°, 8°, 7°, 5° (odd signs); 5°, 7°, 8°, 5°, 5° (even signs) |
| Equal | **no** |
| Occupiable | **10** |

The one Parashari varga that is not an equal division, and the only one whose
targets are chosen as the planetary lords' *own signs* rather than by counting.

> "**TRIMSAMSA:** The Trimsamsa lords for an odd sign are: Mars, Saturn,
> Jupiter, Mercury and Venus. Each of them in order rules 5, 5, 8, 7 and 5
> degrees. The deities ruling over the Trimsamsas are respectively, Agni, Vayu,
> Indra, Kubera, and Varuna. In the case of an even sign, the quantum of
> Trimsamsa, planetary lordship and deities get reversed."
> — BPHS ch. 6 v. 27–28

Santhanam's speculum then names the signs:

| Odd signs | | Even signs | |
|---|---|---|---|
| First 5° | Aries | First 5° | Taurus |
| Next 5° | Aquarius | Next 7° | Virgo |
| Next 8° | Sagittarius | Next 8° | Pisces |
| Next 7° | Gemini | Next 5° | Capricorn |
| Next 5° | Libra | Next 5° | Scorpio |

**Why those signs.** Each of the five planets owns two rashis, one odd and one
even. For an odd rashi the target is that planet's **odd** sign; for an even
rashi, its **even** sign. Mars: Mesha (odd) / Vrishchika (even). Saturn: Kumbha
/ Makara. Jupiter: Dhanu / Meena. Mercury: Mithuna / Kanya. Venus: Tula /
Vrishabha. Both columns are exactly that, which is why the even-sign order is
the reverse of the odd-sign order: reversing the lord sequence Mars → Venus into
Venus → Mars, and reversing 5/5/8/7/5 into 5/7/8/5/5.

**The Sun and Moon own no trimsamsa,** so Karka and Simha never appear —
§4.3.

**Rule (Parashari):**

```
odd sign  (s % 2 == 0):   0 ≤ d <  5 → Mesha (0)
                          5 ≤ d < 10 → Kumbha (10)
                         10 ≤ d < 18 → Dhanu (8)
                         18 ≤ d < 25 → Mithuna (2)
                         25 ≤ d < 30 → Tula (6)

even sign (s % 2 == 1):   0 ≤ d <  5 → Vrishabha (1)
                          5 ≤ d < 12 → Kanya (5)
                         12 ≤ d < 20 → Meena (11)
                         20 ≤ d < 25 → Makara (9)
                         25 ≤ d < 30 → Vrishchika (7)
```

Note the target does **not** depend on `s` at all beyond its parity. Every odd
rashi's first 5° goes to Mesha; every even rashi's first 5° goes to Vrishabha.

**Corroborated exactly, boundary for boundary, by three further sources.** The
unequal split is the single most error-prone table in the sixteen, so it was
checked more than the rest:

| Source | Form given | Agrees |
|---|---|---|
| Narasimha Rao | ten degree intervals, odd and even | yes |
| Sanjay Rath, *Trimsamsa D-30 Chart* | cumulative table: odd 5/10/18/25/30 → Ar, Aq, Sg, Ge, Li; even 5/12/20/25/30 → Ta, Vi, Pi, Cp, Sc | yes |
| DesiUtils D30 calculator | degrees, lord, target sign and deity, both parities | yes |

Rath adds the reason the luminaries are absent: "The Sun and Moon are not the
Lords of any trimsamsa and the Nodes (Rahu & Ketu) also do not own any
trimsamsa." (That is about *lordship*. The nodes are still *placed* in D30 by
the ordinary rule — §4.2.)

**Worked examples**

| Longitude | Rashi, degree | D30 |
|---|---|---|
| 3.0000° | Mesha 3°00′00″ (odd) | **Mesha** |
| 7.0000° | Mesha 7°00′00″ (odd) | **Kumbha** |
| 14.0000° | Mesha 14°00′00″ (odd) | **Dhanu** |
| 20.0000° | Mesha 20°00′00″ (odd) | **Mithuna** |
| 27.0000° | Mesha 27°00′00″ (odd) | **Tula** |
| 33.0000° | Vrishabha 3°00′00″ (even) | **Vrishabha** |
| 37.0000° | Vrishabha 7°00′00″ (even) | **Kanya** |
| 44.0000° | Vrishabha 14°00′00″ (even) | **Meena** |
| 52.0000° | Vrishabha 22°00′00″ (even) | **Makara** |
| 57.0000° | Vrishabha 27°00′00″ (even) | **Vrishchika** |

Boundary vectors: Mesha 9°59′59″ → Kumbha, Mesha 10°00′00″ → Dhanu;
Vrishabha 11°59′59″ → Kanya, Vrishabha 12°00′00″ → Meena. The odd and even
boundary sets differ (10/18/25 against 12/20/25), so both need testing.

**Variants.** JHora ships "**three different variations of D-30 charts**", and
Sanjay Rath states plainly that "There are two methods to draw a Trimsamsa (D30)
Chart. We discuss the method of Parashara" — then does not give the other. So
the existence of alternatives is attested twice and no source states one.

JHora's reimplementation names five: traditional Parashari, a cyclic
"parivritti" trimsamsa, a "shashtyamsa-like" trimsamsa, an even-sign reversal,
and the Somanatha alternate. Those are names in an enumeration, not rules, and
none of them is recorded here as a rule. §9.

The Parashari unequal scheme above is the default in JHora, in Maitreya and in
VedAstro, and is the only one any consulted source states.

**A second, independent axis exists for D30 and only D30.** JHora 7.51 added an
option that concerns not which sign a body lands in but *where in that sign*:
"find divisional longitudes in D-30 by mapping each one degree in rasi to 30
degrees in D-30, or to map arcs (e.g. 5 deg or 8 deg arc) that go to the same
sign in D-30 to 30 degrees **together**". Because the segments are unequal,
there is no single obvious way to stretch one to a full sign. This only matters
if a varga chart ever prints a degree rather than a sign; the D1 chart does not
(`kundali.md` §1.3), so the first pass does not have to choose.

Maitreya makes the choice the other way and does not document it: its D30 runs
the degree *backwards* inside the target sign for an even rashi. Read from its
source, not from its manual.

**Nodes:** **always in the same rashi.**

---

### 7.14 D40 — Khavedamsa (Chatvarimsamsa)

| | |
|---|---|
| Parts | 40 |
| Segment | 45′ |
| Equal | yes |
| Occupiable | 12 |

> "**CHATVARIMSAMSA (1/40th part of a sign):** For odd signs count from Aries
> and for an even sign from Libra in respect of Chatvarimsamsas (each of 45′ of
> arc). […] *Notes:* Chatvarimsamsa or Khavedamsa is a fortieth part of a sign
> or 45′ of arc. These are successively distributed in the various signs from
> Aries in case of any odd sign, and from Libra in case of any even sign."
> — BPHS ch. 6 v. 29–30

Corroborated by Narasimha Rao: "starting from Ar or Li, based on whether the
rasi is odd or even."

Like D24, the start does not depend on the rashi beyond its parity.

**Rule:** `(6·(s mod 2) + p) mod 12`

**Worked examples**

| Longitude | Rashi, degree | Parity | Part | D40 |
|---|---|---|---|---|
| 5.0000° | Mesha 5°00′00″ | odd | 7 of 40 | **Tula** |
| 35.0000° | Vrishabha 5°00′00″ | even | 7 of 40 | **Mesha** |
| 71.0000° | Mithuna 11°00′00″ | odd | 15 of 40 | **Mithuna** |
| 229.0000° | Vrishchika 19°00′00″ | even | 26 of 40 | **Vrishchika** |
| 0.74972° | Mesha 0°44′59″ | | 1 of 40 | **Mesha** |
| 0.75000° | Mesha 0°45′00″ | | 2 of 40 | **Vrishabha** |

Rao's examples reproduce: Mithuna 11° → Mithuna; Vrishchika 19° → Vrishchika.

**Variants.** None found.

**Nodes:** **always in the same rashi.**

---

### 7.15 D45 — Akshavedamsa

| | |
|---|---|
| Parts | 45 |
| Segment | 40′ |
| Equal | yes |
| Occupiable | 12 |

> "**AKSHAVEDAMSA (1/45th part of a sign):** Aries, Leo and Sagittarius are the
> signs from which the distributions respectively commence for movable,
> immovable and common signs. […] *Notes:* Each Akshavedamsa is of 40′ arc as a
> sign is divided into 45 equal parts. Aries is the starting point for all
> movable signs, Leo for all fixed signs and Sagittarius for all dual signs."
> — BPHS ch. 6 v. 31–32

Corroborated by Narasimha Rao: "starting from Ar, Le or Sg, based on whether the
rasi is a movable, fixed or dual rasi." Alias: Pancha-chatvarimsamsa.

The same three start signs as D16, on the same three classes.

**Rule:** `(4·(s mod 3) + p) mod 12`

**Worked examples**

| Longitude | Rashi, degree | Class | Part | D45 |
|---|---|---|---|---|
| 5.0000° | Mesha 5°00′00″ | movable | 8 of 45 | **Vrishchika** |
| 35.0000° | Vrishabha 5°00′00″ | fixed | 8 of 45 | **Meena** |
| 65.0000° | Mithuna 5°00′00″ | dual | 8 of 45 | **Karka** |
| 229.0000° | Vrishchika 19°00′00″ | fixed | 29 of 45 | **Dhanu** |
| 0.66639° | Mesha 0°39′59″ | | 1 of 45 | **Mesha** |
| 0.66667° | Mesha 0°40′00″ | | 2 of 45 | **Vrishabha** |

Rao's examples reproduce: Mithuna 11° → Mesha; Vrishchika 19° → Dhanu.

**Variants.** None found.

**Nodes:** **always in the same rashi.**

---

### 7.16 D60 — Shashtiamsa

| | |
|---|---|
| Parts | 60 |
| Segment | 30′ |
| Equal | yes |
| Occupiable | 12 |

> "**SHASHTIAMSA (1/60th part of a sign or half-a-degree each):** To calculate
> the Shashtiamsa lord, ignore the sign position of a planet and take the
> degrees etc. it traversed in that sign. Multiply that figure by 2 and divide
> the degrees by 12. Add 1 to the remainder which will indicate the sign in
> which the Shashtiamsa falls."
> — BPHS ch. 6 v. 33–41

BPHS's own worked example:

> "Assume that Venus is placed in Capricorn 13°25′. To find out the Shashtiamsa
> lord, ignore the sign position and multiply the degrees and minutes by 2.
> Hence 13°25′ × 2 = 26°50′. The degrees i.e. 26 (ignoring minutes) be divided
> by 12. The remainder is 2 which should be increased by 1. Thus we get 3. Count
> 3 signs from Capricorn. The resulting Shashtiamsa position is Pisces."

Corroborated by Narasimha Rao with the identical procedure — "take its longitude
from the beginning of the occupied rasi, multiply it by 2, take degrees and
ignore minutes, add 1 to it" — and a second worked example: Vrishchika 12°58′ →
Dhanu.

**The verse's arithmetic and `floor(d/0.5)` are the same operation.**
`floor(2d)` is `floor(d / 0.5)`, which is `p`. "Add 1" makes it 1-based; "count
that many from the sign" is inclusive. So the two forms agree exactly.

**Rule:** `(s + p) mod 12` — the same shape as D12.

**On the even-sign reversal.** BPHS says "The reverse is the order for even signs
insomuch as **these names** are concerned" — the sixty Shashtiamsa *names*
(Ghora, Rakshasa, Deva …) run 0°→30° in an odd sign and 30°→0° in an even one.
The **sign mapping does not reverse.** Santhanam's table of the sixty names
prints both columns of degree ranges to make this explicit. Getting this wrong
would silently mirror every even sign.

**Worked examples**

| Longitude | Rashi, degree | Part | D60 |
|---|---|---|---|
| 5.0000° | Mesha 5°00′00″ | 11 of 60 | **Kumbha** |
| 35.0000° | Vrishabha 5°00′00″ | 11 of 60 | **Meena** |
| 222.9667° | Vrishchika 12°58′00″ | 26 of 60 | **Dhanu** |
| 283.4167° | Makara 13°25′00″ | 27 of 60 | **Meena** |
| 0.49972° | Mesha 0°29′59″ | 1 of 60 | **Mesha** |
| 0.50000° | Mesha 0°30′00″ | 2 of 60 | **Vrishabha** |

The last two data rows are BPHS's own example and Rao's own example
respectively, both reproducing.

**Variants.** None found for the sign mapping. JHora lists no D-60 variations on
its feature page.

**Nodes:** always the 7th from each other.

**Precision:** the finest of the sixteen. 30′ of longitude, 2 minutes of clock
time, and — per §6.3 — the varga where the place list's coverage gap bites
hardest.

---

## 8. Disagreements, collected

Everything this document declines to resolve, in one place. Precedent: **D-026**
(no dignity for the nodes) and the planetary-war rule — where authorities
differ, the app prints the difference or prints nothing, never a winner.

| # | Varga | The disagreement |
|---|---|---|
| 1 | D2 | Six schemes in JHora. Standard Cancer–Leo occupies two signs; Parivritti Dwaya occupies twelve; Kashinatha uses day/night sign strength. They give different charts for the same body |
| 2 | D2 | Whether Sanjay Rath's six-sign Surya Hora / Chandra Hora is a D-2 mapping at all, or a different object |
| 3 | D3 | Four schemes in JHora. Parashari (1st/5th/9th) is the default and is universal in the general literature; the **Jagannatha** drekkana is sourced in §7.3 and gives a different answer for every fixed and dual sign; Somanatha and Parivritti-traya are named but their rules were not found |
| 4 | D4 | Two schemes in JHora; the second not found |
| 5 | D9 | Three schemes in JHora, for the varga usually treated as settled; the other two not found |
| 6 | D27 | Santhanam states that Saravali gives a different nakshatramsa calculation and calls it defective. Two classics, two rules |
| 7 | D27 | BPHS's *verse* says only "from Aries and other movable signs"; the element assignment comes from the translator's note. The verse alone is not computable |
| 8 | D30 | Three schemes in JHora, and Sanjay Rath says outright that there are two methods and gives only Parashara's. The Parashari unequal 5/5/8/7/5 is the default everywhere consulted and is corroborated by four sources; the alternatives were not found |

Position taken: **the first pass implements the Parashari rule only, and names
it.** A varga chart should say which scheme produced it, for the same reason
`chakra.rs` carries `place` — a D-2 computed one way looks exactly like a D-2
computed another.

---

## 9. Unverified

Entries here are gaps, not omissions. Nothing below was filled by
reconstruction.

| # | What is unknown |
|---|---|
| 1 | The rules of three of JHora's six D-2 variants. Only Cancer–Leo, Parivritti Dwaya and Kashinatha were sourced |
| 2 | The Kashinatha D-2 rule where a rashi's lord owns only one sign (Simha under the Moon's hora, Karka under the Sun's) |
| 3 | The mapping rule of the Somanatha drekkana. Only one data point found (Meena 20°–30° → Tula); the rule is referred out to two Jaimini texts not consulted |
| 3a | The mapping rule of the Parivritti-traya drekkana. Named in every list of four; no source states it |
| 4 | The second D-4 variant in JHora |
| 5 | The second and third D-9 variants in JHora |
| 6 | What Saravali's nakshatramsa rule actually is |
| 7 | The second and third D-30 variants in JHora — their existence is attested twice, their rules not once |
| 8 | The numeric spread between the ayanamsas the app offers (§6.4). Measurable in-app with `Engine::ayanamsa`; not quoted here because no source table was found |
| 9 | Whether any tradition places Rahu or Ketu in a varga by a rule other than the one used for a graha. Nothing found either way; the silence is itself weak evidence that the ordinary rule applies |

---

## 10. Sources

| Source | Supports |
|---|---|
| [Brihat Parasara Hora Sastra Vol. I, tr. R. Santhanam, Ranjan Publications, Delhi 1992 — archive.org scan](https://archive.org/details/heag_brihat-parasara-hora-sastra-vol-1-by-maharshi-parasara-commentary-editor-tr) | The primary text. Ch. 4 v. 5–11 (sign classifications), ch. 5 (day/night strength), ch. 6 v. 2–41 (all sixteen varga rules, speculums and worked examples), ch. 7 v. 1–8 (what each varga is read for) and v. 42–53 (varga groupings), plus Santhanam's notes on the Sun and Moon in D30 and on Saravali's D27 |
| [Brihat Parashara Hora Shastra — second archive.org scan of the same Santhanam translation](https://archive.org/details/BPHSEnglish) | Cross-check. Used where the first scan's OCR is unreadable; the two agree where both are legible |
| [P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*, ch. 6 "Divisional Charts"](https://www.vedicastrologer.org/articles/vedic_astro_textbook.pdf) | Independent statement of all sixteen rules with worked examples, in modern notation. The D9 rule stated by element rather than by class, the D30 degree ranges, the D60 procedure, and the statement that lagna and special lagnas divide like planets. Contains the D27 erratum recorded in §7.12 |
| [Features of Jagannatha Hora — vedicastrologer.org](https://www.vedicastrologer.org/jh/features.htm) | The variant counts, verbatim: six D-2, four D-3, two D-4, three D-9, three D-30; and that JHora offers 23 divisional charts, mean or true nodes, and seven named ayanamsas |
| [P.V.R. Narasimha Rao, "Using Kashinatha Hora Chart" — Sri Jagannatha Jyotish discussion articles](https://jyotish-blog.blogspot.com/2008/06/using-kashinatha-hora-chart.html) | The standard Cancer–Leo hora rule stated as a rule ("Sun's hora means Leo in hora chart"), the Parivritti Dwaya hora rule, and the Kashinatha hora rule with a worked example |
| [Sanjay Rath, "Principles of Divisional Charts"](https://srath.com/jyoti%E1%B9%A3a/principles-of-divisional-charts/) | The drekkana rule stated independently; a worked example that applies it **to the lagna** ("Lagna at 14° Pisces is in second Drekkana and is mapped into Cancer the fifth house from Pisces"); the six-sign Surya/Chandra Hora conception |
| [P.V.R. Narasimha Rao, "Jagannatha Drekkana"](https://jyotish-blog.blogspot.com/2005/03/jagannatha-drekkana.html) · [and "Somnatha Drekkana"](https://jyotish-blog.blogspot.com/2005/02/somnatha-drekkana.html) | The Jagannatha drekkana mapping in full, with two worked comparisons against the Parashari; and the one Somanatha data point, with its rule referred out to texts not consulted |
| [Vijayalur, "Drekkana / Dreshkana / Decanate"](https://vijayalur.com/2011/05/19/drekkana-dreshkana-decante/) | Independent statement of the Parashari drekkana ("The first Drekkana of any sign is ruled by the owning planet, second Drekkana by the 5th lord and the third Drekkana by the 9th lord"), and that there are four kinds — while declining to state the other three |
| [Sanjay Rath, "Trimsamsa D-30 Chart"](https://srath.com/jyoti%E1%B9%A3a/varga/trimsamsa-d-30-chart/) | Third independent statement of the D30 table, in cumulative-degree form; that the Sun, Moon, Rahu and Ketu own no trimsamsa; that the lagna is divided like a planet; and that a second D30 method exists which he does not give |
| [DesiUtils — Trimsamsa D30 calculator](https://desiutils.in/astrology/trimsamsa-d30) | Fourth statement of the D30 table, as published calculator behaviour, including the ten-sign restriction stated outright |

In-repo references: `docs/DECISIONS.md` D-003 (ayanamsa, `Ketu = Rahu + 180°`),
D-006 (provenance), D-026 (no dignity for the nodes); `docs/TODO.md` E1
(the D1 chart as built), E5 (the place list, 418 places at arcminute
precision), and the "cited, not remembered" note under Panchanga;
`docs/design/kundali.md` §1.2 (the lagna's rate of motion), §3 (the three chart
formats); `crates/almanac/src/chakra.rs` (the `Chakra` payload the varga charts
would extend); `crates/almanac/src/zodiac.rs` (`Rashi::index`,
`degrees_in_rashi`, `pada`).
