# Vargas — the sixteen divisional charts

Status: **the specification, and all sixteen are built against it**
(`crates/almanac/src/varga.rs`, whose tests are the worked examples in §7, and
`Varga::ALL`). D-032 records the three choices §8 left open.

The first pass was D1, D3, D7, D9 and D12; the other eleven followed. All
sixteen were documented here from the start, because the rules are a single
family and writing five of them without the rest would mean discovering the
shared structure twice.

---

## 0. The standard this document is held to

`TODO.md` records that the Durmuhurtam table was written from memory and two of
its values were wrong. The rule that came out of that is **domain tables are
cited, not remembered.** This document is written to it:

| Rule | What it means here |
|---|---|
| Every rule carries a source | Chapter and verse where one exists, and the translation named |
| Two independent sources | In practice five classical texts were read in full — BPHS, Brihat Jataka, Phaladeepika, Jataka Parijata, Saravali — plus two modern authors and four software implementations |
| Disagreements are printed, not resolved | The app already refuses to name a winner in a planetary war and gives the nodes no dignity (D-026). Printing one reading of a disputed rule would be the app asserting an interpretation |
| "Unverified" is a valid entry | §9 lists what could not be sourced. Gaps are not filled with plausible reconstruction |

Five errors and self-contradictions found while writing this are recorded rather
than quietly corrected — an arithmetic slip in a reference text's own worked
example (§7.12), a translation whose verse and whose note say different things
(§7.12), a footnote that attributes a rule to *Saravali* which *Saravali* does
not contain (§7.12), a software feature page that contradicts its own release
notes (§8), and an open-source implementation whose D-4 computes a D-2 (§7.4).
They are listed in §8, because a citation that is not checked is not much better
than a memory.

Two things this document deliberately does not do. It does not choose between
schemes where authorities differ — §8 lists ten such places. And it does not
treat a rule as sourced because software implements it: §10 grades each
implementation by how far its behaviour can be trusted as evidence.

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

`p` itself has a classical statement, and it is the same arithmetic:

> "To know any kind of Varga (Hora, Navansa, Drekkana etc.) adopt the following
> method. Convert the longitude into minutes of arc and multiply by the Varga
> figure concerned. Divide the product by 1800. The resultant figure will reveal
> the required Varga."
> — Saravali ch. 3 v. 18, Santhanam translation

1800 arcminutes is 30°, so that is `floor(d · n / 30)` in integer arcminutes —
exactly §6.6's recommendation to multiply before dividing, arrived at a thousand
years before floating point made it necessary.

---

## 2. The sixteen, in summary

| Varga | Name | `n` | Segment | Equal? | Rule | Signs it can occupy | Variants in the wild |
|---|---|---|---|---|---|---|---|
| D1 | Rashi | 1 | 30° | yes | `s` | 12 | — |
| D2 | Hora | 2 | 15° | yes | odd sign: Simha then Karka; even sign: reversed | **2** | **6 modern + the Yavana hora**; no classic names the signs |
| D3 | Drekkana | 3 | 10° | yes | `(s + 4p) mod 12` | 12 | **5**, four of them named and sourced |
| D4 | Chaturthamsa | 4 | 7°30′ | yes | `(s + 3p) mod 12` | 12 | 2–4, all generic axes |
| D7 | Saptamsa | 7 | 4°17′08.57″ | yes | `(7s + p) mod 12` | 12 | 3–6, all generic axes |
| D9 | Navamsa | 9 | 3°20′ | yes | `(9s + p) mod 12` | 12 | **3 or 4** — JHora's own docs disagree |
| D10 | Dasamsa | 10 | 3° | yes | `(s + 8·(s mod 2) + p) mod 12` | 12 | 4–6, all generic axes |
| D12 | Dwadasamsa | 12 | 2°30′ | yes | `(s + p) mod 12` | 12 | 2–5, all generic axes |
| D16 | Shodasamsa | 16 | 1°52′30″ | yes | `(4s + p) mod 12` | 12 | **a named classical split**, plus 2–4 generic |
| D20 | Vimsamsa | 20 | 1°30′ | yes | `(8s + p) mod 12` | 12 | 2–4, all generic axes |
| D24 | Chaturvimsamsa | 24 | 1°15′ | yes | `(4 − (s mod 2) + p) mod 12` | 12 | 3, all generic axes |
| D27 | Bhamsa | 27 | 1°06′40″ | yes | `(3s + p) mod 12` | 12 | one repudiated rule (§7.12), plus 2–3 generic |
| D30 | Trimsamsa | 5 | **unequal**, 5/5/8/7/5 | **no** | table, §7.13 | **10** | **equal-vs-unequal is a live argument**, plus 3–5 schemes |
| D40 | Khavedamsa | 40 | 45′ | yes | `(6·(s mod 2) + p) mod 12` | 12 | 4, all generic axes |
| D45 | Akshavedamsa | 45 | 40′ | yes | `(4·(s mod 3) + p) mod 12` | 12 | 4, all generic axes |
| D60 | Shashtiamsa | 60 | 30′ | yes | `(s + p) mod 12` | 12 | 4, all generic axes |

The "variants" column counts computation schemes shipped by Jagannatha Hora
and/or enumerated by its Python reimplementation, with classical disagreements
called out in bold. A range is given where JHora's feature page and its own
release notes disagree. **"All generic axes" means every alternative is one of
the three transformations in §4.4 rather than a different tradition** — a
distinction that matters, because the first are one flag on one function and the
second are separate rules needing separate citations. Every rule stated in §7 is
the Parashari one.

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

**Note where the enumeration comes from.** The *verse* defines the classes by
alternation from Mesha and names no signs; the four-sign lists are the
translator's note. The same is true of the two other classical statements:

> "The signs (from Mesa) are alternately malefic and benefic; male and female —
> moveable, fixed and common."
> — Brihat Jataka I.11, Vijnananda translation (the enumeration is again in his note)

> "From Aries onwards alternatively the Rasis are known as malefic and benefic on
> the one hand and male and female on the other hand. These are also classified
> as Chara (Movable), Sthira (Fixed, Immovable) and DvisvaBhava (Ubhaya, Dual,
> Common) Rasis."
> — Saravali ch. 3 v. 20–21, Santhanam translation

Arithmetically the two forms are identical — `s mod 3` *is* the alternation. Two
modern sources enumerate in the body text rather than in a note: Narasimha Rao
(§2.2.4, "Ar, Cn, Li and Cp are known as chara rasis […] Ta, Le, Sc and Aq […]
Ge, Vi, Sg and Pi"), and deFouw & Svoboda, *Light on Life* ch. 5, with the same
three groups. **Four independent sources, no disagreement.**

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
Vijnananda's gloss on Brihat Jataka I.11 gives the Sanskrit directly — *ayuji*,
"in the odd signs"; *samabhe*, "in the even signs" — with the same two lists.
Narasimha Rao gives every synonym: "Ar, Ge, Le, Li, Sg and Aq are called odd
rasis or vishama rasis or oja rasis. They are also known as male rasis."

**A trap.** Rao (his §2.2.3) defines a second, different pairing one paragraph later:
"Ar, Ta, Ge, Li, Sc and Sg are called **odd-footed** rasis or vishamapada rasis
or **ojapada** rasis." *Oja-pada* is not *oja*. It is used by some dasas and by
**no varga rule in this document**. The two share a Sanskrit stem and differ in
six of twelve signs.

### 3.3 The four elements

Used by D27, and by the element statement of D9 (§7.6).

| Element | Rashis | Indices |
|---|---|---|
| Fiery (agni) | Mesha, Simha, Dhanu | 0, 4, 8 |
| Earthy (prithvi) | Vrishabha, Kanya, Makara | 1, 5, 9 |
| Airy (vayu) | Mithuna, Tula, Kumbha | 2, 6, 10 |
| Watery (jala) | Karka, Vrishchika, Meena | 3, 7, 11 |

`element = s mod 4`, with `0` fiery, `1` earthy, `2` airy, `3` watery.

**BPHS ch. 4 gives no element table.** It labels individual signs, and it groups
the same four sets under a different vocabulary. The element *names* used by the
D9 and D27 rules come from elsewhere in the tradition — so cite Rao or *Light on
Life* for the grouping, not BPHS.

Per sign, in the descriptions:

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
four groups. Brihat Jataka I.11 groups the same four trines again, under a third
vocabulary — direction: "Mesa, Simha, and Dhanu represent east; Vrisa, Kanya and
Makara represent south; Mithuna, Tula, and Kumbha represent west and Karka,
Vrischika and Mina represent north."

The element names themselves are stated by two modern sources:

> "(1) Ar, Le and Sg are called agni rasis or fiery rasis. (2) Ta, Vi and Cp are
> called bhoo rasis or earthy rasis. (3) Ge, Li and Aq are called vaayu rasis or
> airy rasis. (4) Cn, Sc and Pi are called jala rasis or watery rasis."
> — Narasimha Rao, his §2.2.5

> "ARIES, LEO and SAGITTARIUS are Fiery / TAURUS, VIRGO and CAPRICORN are Earthy
> / GEMINI, LIBRA and AQUARIUS are Airy / CANCER, SCORPIO and PISCES are Watery"
> — deFouw & Svoboda, *Light on Life*, ch. 5

Four vocabularies — temperament, direction, element, and the bare trine — one
partition. No disagreement anywhere.

**The asymmetry that will be mistyped.** D9 starts from **Mesha, Makara,
Tula, Karka** for fiery / earthy / airy / watery. D27 starts from **Mesha,
Karka, Tula, Makara**. The earthy and watery starts are swapped between the two.
Both are independently attested (§7.6, §7.12); neither is a typo.

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

## 4. Four structural facts

The first three are derived from the rules in §7, not asserted, and each is a
property the implementation can be tested against. The fourth is a way of
organising the disagreements so they stop looking like sixteen separate
arguments.

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

Corroborated negatively, which is the only way a silence can be: three
independent implementations were read for a node branch inside a varga
function, and none has one — Jagannatha Hora's Python reimplementation,
Maitreya, and VedAstro, whose varga tables are keyed by sign and degree alone so
that the identity of the body cannot reach the result. The reverse-reckoning
options that do exist in JHora are about *even signs* (all bodies) or about
*dasas*, not about vargas and not about nodes.

One exception, and it is only cosmetic: Maitreya reverses the *displayed degree*
within the varga sign for Rahu and Ketu — `30 − degree` — hardwired, with no
setting and no documentation. The varga **sign** is unaffected. Since the D1
chart does not print degrees in a compartment (`kundali.md` §1.3), this does not
arise for us; it is recorded so that a future comparison against Maitreya is not
misread as a disagreement about placement.

Whether any *tradition* places the nodes by a different rule remains
**unverified** (§9). The silence across four independent software
implementations and every text consulted is weak evidence that it does not.

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

**The D30 restriction is a property of the drawing convention, not of the
classical scheme.** What the texts say is that the Sun and Moon own no
trimsamsa — Santhanam on Saravali 3.15, "The Sun and the Moon have no lordship
over Trimsamsa division"; Rath, "the Sun and Moon are not the Lords of any
trimsamsa and the Nodes (Rahu & Ketu) also do not own any trimsamsa." Karka and
Simha then become unreachable only once the lord is turned into a *sign*, which
is what drawing a D30 chart requires and what Varahamihira never did (§7.13).
Once that step is taken the restriction follows by construction, and it was
verified here by enumeration; but only one consulted source states it in words,
and that source is a calculator rather than an authority: "every D30 chart ever
drawn uses only ten of the twelve signs."

BPHS comes closest, in the course of explaining why the Sun and Moon are a
special case:

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
- The fourth is not. Each member needs its own rule and its own citation. Of the
  five named, only the **Jagannatha drekkana** is stated here precisely enough
  to implement without further research; the Kashinatha and Raman horas each
  have an undefined case (§7.2), and the two Krishna Mishra / Kalachakra
  navamsas have no stated rule at all (§9).

**First pass takes none of them.** Parashari only, named on the surface.

---

## 5. The varga lagna

**The same rule, applied to the ascendant's longitude.** This is cited, not
assumed, because it was the one point where the obvious answer could have been
wrong.

Five statements, from four sources, plus four classical texts that assume it:

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
   Rasi chart are used to determine the Trimsamsa occupied by them. […] In this
   manner, the trimsamsa of all the planets and Lagna is determined and the
   resultant chart is called the Trimsamsa or D-30 Chart." Same rule, a
   different varga, so it is not a property of the drekkana alone.
5. deFouw & Svoboda give the whole procedure, and put the lagna first: "To draw
   up the navamsha you begin by fixing its ascendant. If, for example, a
   horoscope's ascendant is 15° of Sagittarius, that ascendant point will fall in
   the fifth navamsha of Sagittarius, which happens to be Leo. […] The navamsha
   chart for this horoscope will therefore have Leo as its first house. Then,
   each graha's navamsha is determined, and they are placed in the navamsha chart
   accordingly." Checked: Dhanu is `s = 8`, 15° is `p = 4`,
   `(9·8 + 4) mod 12 = 4` = Simha. Reproduces.

The classics assume it rather than state it, which is the next best thing —
Jataka Parijata I.31, "the owners of the Sapthamamsas or the 7th portions **of
Lagna and other houses**"; Jataka Parijata I.36, "**when the Lagna is an even
sign**, the lords of the Shodasamsas are to be counted in the inverse order";
Saravali 3.13, "should the natal **Lagna** be in such Vargothama Navansa";
Phaladeepika 3:11, "if the lord of the **Navamsa of the Ascendant** be strong".
A lagna that has a navamsa, a saptamsa and a shodasamsa, and whose odd/even
parity is read off its rashi, is being divided by the ordinary rule.

No source found states a different rule for the ascendant, in any varga. There
is no separate "varga lagna formula".

Corroborated in code: JHora's reimplementation puts the ascendant at index 0 of
the same position list every graha goes through, with no special case, and
Maitreya routes its ascendant object through the same `calcVarga` call as a
planet.

**One implementation disagrees, and it is worth knowing about.** VedAstro does
not derive a varga lagna at all. It maps each of the twelve *houses* through the
varga table using the **house's midpoint longitude** — its own doc-comments say
"Get Navamsa D9 sign of house mid point". Two consequences: its divisional
"lagna" is the varga of the first house cusp's midpoint rather than of the exact
ascendant, and the twelve resulting divisional house-signs are not guaranteed to
be twelve consecutive distinct signs. That is a different model, not a different
formula, and this document does not adopt it.

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

The same figures, from a jyotisha source rather than from our own code —
Narasimha Rao, *Impact of Birthtime Error*: "**Lagna moves by 1° in 4 min. Lagna
moves by 10′ in 2/3 min (or 40 seconds). Lagna moves by 1′ in 4 sec. Lagna moves
by 10″ in 2/3 sec.**" And, on what that costs: "We can see that lagna changes
rasi in **D-10 in 12 min**. It changes rasi in **D-24 in 5 min**" — both of
which the table below reproduces exactly. His conclusion is the one that matters
for us: "**Lagna changes rasi in divisional charts much faster than planets.**"

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

### 6.5 Swiss Ephemeris does none of this

Checked, because it would be worth knowing if it did: **Swiss Ephemeris computes
no vargas.** The programmer's documentation, the user documentation and
`swetest -h` between them contain no occurrence of *varga*, *navamsa*,
*drekkana*, *amsa* (outside the word *ayanamsa*) or *divisional*; every `hora`
hit is `SEFLG_JPLHORA`, meaning JPL Horizons. What it supplies is the sidereal
frame — `swe_set_sid_mode`, `swe_get_ayanamsa_ex_ut`, `SEFLG_SIDEREAL`,
`swe_houses_ex` — which is exactly what the app already uses.

So every rule in §7 is ours to implement, and every one of them is arithmetic on
a longitude the engine already returns. There is no library call to defer to and
no library behaviour to match. That is the argument for the test vectors in §7
being numerous rather than representative.

### 6.6 Two implementation notes on `p`

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

**The verse names lords, not signs — and so does every other classical text.**
Four independent statements, all of them assigning the two halves to the Sun and
the Moon as *lords*, none of them naming Simha or Karka:

| Source | Words |
|---|---|
| Phaladeepika 3:4 | "Hora is half of a Rasi. In an odd sign, the halves belong to the Sun and the Moon and to the Moon and the Sun when the sign is an even one." |
| Jataka Parijata I.30 | "Hora means the half of a Rasi; in an odd sign, the halves belong respectively to the Sun and the Moon, and in an even one to the Moon and the Sun." |
| Saravali 3.14 | "The first Hora of an odd Rasi belongs to the Sun, while the second one is ruled by the Moon. In the case of an even Rasi the first Hora is ruled by the Moon and the second by the Sun." |
| Brihat Jataka I.11 | "The Sun and Moon are respectively the rulers of the two halves of any odd sign; and the Moon and the Sun become the rulers of the horas of any even sign." |

The step from "the Sun's hora" to "Simha" is **a drawing convention, not a
classical rule**. It is what every modern implementation does, and it is stated
plainly by Narasimha Rao — "Sun's hora means Leo in hora chart and Moon's hora
means Cancer in hora chart", producing "a hora chart that has all the planets in
two signs — Cancer and Leo" — but no text consulted says it. Rao then adds, of
his own statement of the rule: "**Though absolutely correct, the above is not
quite complete.** Proper use of hora chart is beyond the scope of this book. So
we will ignore and not use hora chart in this book."

That is the deepest reason D2 has six variants: the classics under-specify it,
and everything past lordship is somebody's completion of the text.

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

**A classical dissent, too.** The Yavana school gave the horas to all seven
grahas rather than to the Sun and Moon alone. Jataka Parijata I.30's notes name
the disagreement and its resolution:

> "According to the Yavanas, the lords of the Horas and Drekkanas are as
> described in the following sloka […] so that, according to this view, the
> ownership of the Horas is not restricted to the Sun and the Moon, but is
> shared by all the planets. But this view of the Yavanas is not recognised by
> Varahamihira, nor even by authorities like Satya […] But it may be mentioned
> here that this convention of the Yavanas has been accepted for Prasna (Horary
> astrology) while the other is recognised for purposes of horoscopy."

So the split is old, and the classical resolution is by *use*: the Parashari
hora for a natal chart, the Yavana hora for prasna. Chandra draws neither a
natal chart nor a prasna, so neither use applies to it directly — which is one
more argument for leaving D2 out of the first pass.

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

Corroborated by four classical texts and two moderns. The classics give
*lords*, as with D2 — but here that is harmless, because the 5th and 9th from a
sign are always in its own trine, so lord and sign agree:

| Source | Words |
|---|---|
| Phaladeepika 3:4 | "The Drekkana or third portions of a sign belong to the lords of the sign itself, of the 5th house and of the 9th house." |
| Jataka Parijata I.30 | "…owned by the lords of the sign itself, of the son's or 5th house, and of the 9th or the house of Dharma" |
| Saravali 3.14 | "first Lord of the same Rasi, second Lord of the 5th Rasi therefrom and third Lord of the 9th from the Rasi in question" |
| Brihat Jataka I.11 | "The Decanates are ruled by the lords of its own, fifth and ninth houses." |
| Narasimha Rao | "Bodies in the first 10° of a rasi are placed in drekkana chart in the same rasi. Bodies in the middle 10° […] in the 5th from the rasi. Bodies in the last 10° […] in the 9th from the rasi." |
| Sanjay Rath | stated as signs, not lords: "The first Drekkana of a sign is mapped to itself, the second is mapped to the sign in the fifth from it and the third is mapped to the sign in the ninth from it." |

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

**Variants.** JHora ships "**two different variations of D-4 charts**". Its
reimplementation enumerates four: Parashari, parivritti cyclic, parivritti
even-reverse, Somanatha alternate — the three generic axes of §4.4. Which of
those is JHora's second is **unverified** (§9).

Maitreya offers a "Continuous" D-4 alongside "Parasara" — and, read from its
source, **the continuous branch computes a D-2, not a D-4** (`ret = 2 * len`).
The defect is present identically in Maitreya 8 and in the Maitreya 9 fork.
Recorded because it is a caution about corroborating against software: an
implementation is evidence of what its author intended only where the code has
been read.

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

Corroborated four times over:

| Source | Words |
|---|---|
| Phaladeepika 3:6 | "The Saptamsas or the 1/7th portion are in the case of an odd sign, counted regularly from the sign itself. In the case of an even sign, they are counted from 7th sign onwards." |
| Jataka Parijata I.31 | "…in the case of an odd sign to be counted regularly from the lord thereof, while in the case of an even sign, they are to be reckoned from the lord of the 7th house onwards" |
| Saravali 3.16 | states it as a bare enumeration instead of a rule: "The Sapthamsas for the 12 Rasis from Aries onwards are, respectively, counted from Aries, Scorpio, Gemini, Capricorn, Leo, Pisces, Libra, Taurus, Sagittarius, Cancer, Aquarius and Virgo." |
| Narasimha Rao | "starting from the rasi itself, if it is an odd rasi, or starting from the 7th sign from it, if it is an even rasi" |

Saravali's twelve-sign list is worth checking against the formula rather than
trusting, since it is the one source that could disagree without looking like
it. `(7s) mod 12` for `s = 0…11` gives Mesha, Vrishchika, Mithuna, Makara,
Simha, Meena, Tula, Vrishabha, Dhanu, Karka, Kumbha, Kanya — Saravali's list
exactly. Santhanam's note on the verse says the same in words.

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

The last two are the reason §6.6 exists: the first boundary is at
4.285714285…°. It is one of four whose boundaries are not exactly representable
in decimal — D7, D9, D27 and D45, listed in §6.6.

Narasimha Rao's own examples reproduce: Mithuna 10° → Simha; Kanya 19° → Karka.

**Variants.** None in the classical literature. JHora's feature page lists none
either — but its release notes for 7.63 record "**2 new definitions of D-7**",
"mostly related to reversal of divisions in an even sign". Its reimplementation
enumerates six: Parashari with the even sign counted forward from the 7th
(default), counted backward from the 7th, run in reverse from the end of the
7th, parivritti cyclic, parivritti even-reverse, and Somanatha alternate. See §4.4: the alternatives are the generic axes, not separate traditions.

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

**The classics state the rule in two different vocabularies, and the split runs
across texts rather than between them.**

By starting sign, which is the element form in disguise:

| Source | Words |
|---|---|
| Phaladeepika 3:4 | "The first Navamsa in the signs from Aries onwards begins respectively with Aries, Capricorn, Libra and Cancer." |
| Saravali 3.11 | "The Navansas for these Rasis are calculated from Aries, Capricorn, Libra and Cancer in their order." |
| Jataka Parijata I.32–33 | spelled out sign by sign: "The Navamsas of Dhanus, Mesha and Simha respectively belong to the nine signs from Mesha onwards; those of Vrishabha, Kanya and Makara, to Makara […] The nine signs beginning with Thula are the owners of the Navamsas of Mithuna, Thula and Kumbha. The Navamsas of Kataka, Vrischika and Meena respectively appertain to the nine signs from Kataka onwards." |
| Narasimha Rao | with the element names attached: "starting from Ar, Cp, Li or Cn, based on whether the rasi is a fiery, earthy, airy or watery sign" |

By class, which is BPHS's form — and which the others state through the
vargottama, the navamsa of a sign that bears the sign's own name:

| Source | Words |
|---|---|
| Brihat Jataka I.14 | "In the cardinal signs, the first Navamsa […] is termed Vargottama […] In the fixed signs the fifth Navamsa […] In the mutable signs the ninth Navamsa" |
| Saravali 3.13 | "The first Navansa of a Movable Rasi, the 5th one in a Fixed Rasi and the 9th one in a Common (Dual) Rasi are called Vargothamamsa." |

**These are the same rule, and that is worth showing rather than asserting.**
*Light on Life* is the source that states the equivalence outright, quoting
Phaladeepika and then adding: "Note that the first navamsha for the cardinal
constellation (chara rashi) is always that constellation; the fifth navamsha for
a fixed constellation (sthira rashi) is always that constellation; and the ninth
navamsha for a mutable constellation (dvisvabhava rashi) is always that
constellation."

| Rashi | BPHS (movable/fixed/dual) | Rao (element) | Start |
|---|---|---|---|
| Mesha | movable → itself | fiery → Mesha | Mesha |
| Vrishabha | fixed → 9th = Makara | earthy → Makara | Makara |
| Mithuna | dual → 5th = Tula | airy → Tula | Tula |
| Karka | movable → itself | watery → Karka | Karka |
| Simha | fixed → 9th = Mesha | fiery → Mesha | Mesha |
| Kanya | dual → 5th = Makara | earthy → Makara | Makara |

and so on for all twelve. Both reduce to `9s mod 12`. Checked at every sign:
same / 9th / 5th for movable / fixed / dual gives Mesha, Makara, Tula, Karka,
Mesha, Makara, Tula, Karka, Mesha, Makara, Tula, Karka — which is
fiery / earthy / airy / watery cycling Mesha, Makara, Tula, Karka. **Six
independent statements, one rule.**

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
| 3.33333…° | Mesha 3°20′00″ | | 2 of 9 | **Vrishabha** |

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

Corroborated three times over: Phaladeepika 3:6, "In the case of an odd sign,
the Dasamamsas or 1/10th portions are counted from the sign itself. In the case
of an even sign, they are counted from the 9th onwards"; Jataka Parijata I.35,
the same in the same words but as lords; and Narasimha Rao, "starting from the
rasi itself or the 9th from it, based on whether the rasi is an odd or even
sign." Aliases: Dasamaamsa, Karmamsa, Swargamsa.

Note that BPHS's ch. 6 verse also assigns ten *deities* — Indra, Agni, Yama,
Rakshasa, Varuna, Vayu, Kubera, Isana, Brahma, Anantha, reversed for an even
sign. Those are a parallel scheme on the same segments, not a competing sign
mapping, and this document does not carry them.

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

**Variants.** None in the classical literature. JHora 7.63 records "**3 new
definitions of D-10**"; its reimplementation enumerates six — Parashari with the
even sign counted forward from the 9th (default), backward from the 9th, in
reverse from the end of the 9th, parivritti cyclic, parivritti even-reverse, and
Somanatha alternate. See §4.4: these are the generic axes applied to the Parashari rule, not separate traditions.

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

Corroborated four times: Phaladeepika 3:4, "The owners of the Dwadasamas or
1/12th portion of a sign are counted from that sign"; Jataka Parijata I.35, the
same; Saravali 3.13, "The rulers of Dwadasamasas start from that Rasi itself";
and Narasimha Rao, "Bodies in the 12 parts of a rasi go into the 12 rasis
starting from the rasi itself."

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

**Variants.** None in the classical literature, and D12 is the only varga for
which no source consulted offered an alternative rule. JHora 7.63 still records
"**one new definition of D-12**", and its reimplementation enumerates five —
Parashari, Parashari with even-sign reversal, parivritti cyclic, parivritti
even-reverse, Somanatha alternate. See §4.4: these are the generic axes applied to the Parashari rule, not separate traditions.

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
rasis" — i.e. the `mod 12` is in the source, not an implementation liberty. Also
by Santhanam's own supplementary note to Saravali, item (10): "The counting
commences from Aries for Chara Rasis, from Leo for Sthira Rasis and from
Sagittarius for Dwiswabhava Rasis."

Jataka Parijata I.36's notes corroborate the same mapping arithmetically, by
describing it as a division of the whole ecliptic: "the ecliptic is cut up into
192 equal segments of 1°52′30″ each in length. The segments are named in regular
consecution, in the order of the zodiacal signs, so that the initial segment of
the sign Aries takes on the name of that sign, that of the sign Taurus assumes
the name of Leo, and so forth." That is §4.1's continuous form, `(4s + p) mod
12`, stated as such.

> **A genuine classical disagreement — the only one in the sixteen where a
> commentator names the split in print.**
>
> Jataka Parijata I.36 states the D16 rule as *deities* rather than signs: "Their
> lords in the case of an odd sign are Brahma, Vishnu, Hara and Ravi recurring in
> regular order. When the Lagna is an even sign, the lords of the Shodasamsas are
> to be counted in the inverse order from Bhaskara or Ravi." Phaladeepika 3:6
> gives a related but not identical allocation. And Jataka Parijata's own
> commentator then says so:
>
> > "But **Phaladeepika and Sarvartha Chintamani interpret the allocation of the
> > Shodasamsa rulerships differently.** On that account, the language of
> > Jatakaparijata cannot be forced to bear a meaning which its author did not
> > evidently intend. Jatakaparijata and Parasara go a good way together as
> > regards Shodasamsa rulerships."
>
> Note what is and is not in dispute. Jataka Parijata carries **both** the deity
> scheme and the Mesha/Simha/Dhanu sign scheme, without reconciling them; the
> named disagreement is about the *deity* allocation. What Sarvartha Chintamani's
> D16 *sign* rule actually is remains **unverified** (§9) — it is known here only
> through Jataka Parijata's report of it.
>
> BPHS, Jataka Parijata, Santhanam's Saravali supplement and Narasimha Rao all
> agree on Mesha / Simha / Dhanu. That is what §7.9's rule states.

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

**Variants.** **One classical disagreement, quoted above** — the only varga where
a commentator names one in print. Plus the modern generic axes: JHora 7.63
records "one new definition of D-16"; its reimplementation enumerates four —
Parashari, parivritti even-reverse, parivritti cyclic, Somanatha alternate
(§4.4).

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

**Variants.** None in the classical literature. JHora 7.63 records "**one new
definition of D-20**"; its reimplementation enumerates four, the same set as
D16. See §4.4: these are the generic axes applied to the Parashari rule, not separate traditions.

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

**Variants.** None in the classical literature. JHora 7.63 records "**2 new
definitions of D-24**"; its reimplementation enumerates exactly three —
Parashari, Parashari with even-sign reversal, and Parashari with an even-sign
"double reverse". The counts agree for once. See §4.4: these are the generic axes applied to the Parashari rule, not separate traditions.

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

**Traced, and it is not what the footnote implies.** Kalyana Varma's *Saravali*
contains **no nakshatramsa rule at all** — its varga verses (ch. 3, vv. 11–18)
cover only the dasavarga, and ch. 61 says so: "The effects of ten divisions
(Dasa Vargas) will help to arrive at the natal Ascendant." The "different
calculation" is Santhanam's **own** supplementary note, appended to his Saravali
to complete the shodasavarga. Item (13):

> "Nakshatramsa or Sapthavimsamsa: This is 1/27th part of a Rasi. **The counting
> starts from the Rasi itself, whether it is an odd one or an even one.** For
> example, for Aries, start from Aries and count upto Pisces, you get 12
> Sapthavimsamsas. Then the 13th one is again Aries […] the 27th one is Gemini."

So the rejected rule is `(s + p) mod 12` — the D12 shape applied at 27 parts —
against BPHS's `(3s + p) mod 12`. They agree only for Mesha, Karka, Tula and
Makara, and differ for the other eight. **Not a disagreement between two
classics: one translator publishing two rules and later repudiating the first.**

Recorded in full because the alternative — writing "Saravali differs" and moving
on — would have left a false claim about a classical text standing in a
specification.

*(Santhanam's BPHS footnote cites "P. 31"; the passage sits on p. 24 of the
Vol. I scan read here. Same content, different printing.)*

**Worked examples**

| Longitude | Rashi, degree | Element | Part | D27 |
|---|---|---|---|---|
| 5.0000° | Mesha 5°00′00″ | fiery → Mesha | 5 of 27 | **Simha** |
| 35.0000° | Vrishabha 5°00′00″ | earthy → Karka | 5 of 27 | **Vrishchika** |
| 71.0000° | Mithuna 11°00′00″ | airy → Tula | 10 of 27 | **Karka** |
| 95.0000° | Karka 5°00′00″ | watery → Makara | 5 of 27 | **Vrishabha** |
| 229.0000° | Vrishchika 19°00′00″ | watery → Makara | 18 of 27 | **Mithuna** |
| 1.11083° | Mesha 1°06′39″ | | 1 of 27 | **Mesha** |
| 10/9° | Mesha 1°06′40″ | | 2 of 27 | **Vrishabha** |
| 1.11139° | Mesha 1°06′41″ | | 2 of 27 | **Vrishabha** |

The middle row is the boundary itself, `30/27 = 10/9`, which the pair either
side of it skipped. It is not exactly representable in decimal, so it is written
as the fraction — the same care D7, D9 and D45 need, per §6.6.

The Mithuna 11° row is the one Rao gets wrong. It is included deliberately: a
test asserting **Karka** there is a test that would have caught the erratum.

**Variants.** Besides Santhanam's repudiated one above, JHora 7.63 records "**one
new definition of D-27**"; its reimplementation enumerates three — Parashari, Parashari with
even-sign reversal, Somanatha alternate (§4.4). An independent statement of the
Parashari rule, in the same words as BPHS's verse, is given by Barbara Pijan
Lama: "The Saptavimshamsha distribution commences from Mesha and other Movable
Rashi for all the 12 Rashi", with "Size of 1 Bhamsa = 1 degree + 6 minutes + 40
seconds of arc."

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

**Corroborated exactly, boundary for boundary, by the sources below.** The
unequal split is the single most error-prone table in the sixteen, so it was
checked more than the rest:

| Source | Form given | Agrees |
|---|---|---|
| Brihat Jataka I.7 | "Five, five, eight, seven and five parts (degrees) are respectively those of Mars, Saturn, Jupiter, Mercury and Venus in the odd signs. In the even signs their order is reversed." Aiyar's note resolves "reversed": "the first five are those of Venus; the next seven are those of Mercury; the next eight are those of Jupiter; the next five are those of Saturn; and the last five are those of Mars." | yes |
| Phaladeepika 3:4 | states both parities in full: "In an even sign it is reversed. Then Venus, Mercury, Jupiter, Saturn and Mars have 5, 7, 8, 5 and 5 degrees respectively." | yes |
| Jataka Parijata I.37 | "In an even sign, Sukra, Budha, Guru, Sani and Kuja have 5, 7, 8, 5 and 5 degrees respectively." | yes |
| B.V. Raman, *Hindu Predictive Astrology* ch. XI | tabulated: "In even signs — Venus Mercury Jupiter Saturn Mars / 5 7 8 5 5 = 30" | yes |
| Saravali 3.15 + Santhanam's table | verse says only "reverse"; his table resolves it as 5/7/8/5/5 | yes |
| Narasimha Rao | ten degree intervals, odd and even | yes |
| Sanjay Rath, *Trimsamsa D-30 Chart* | cumulative: odd 5/10/18/25/30 → Ar, Aq, Sg, Ge, Li; even 5/12/20/25/30 → Ta, Vi, Pi, Cp, Sc | yes |
| DesiUtils D30 calculator | degrees, lord, target sign and deity, both parities | yes |

The Sanskrit at BPHS 6.27–28 is *vyatyayāt* / *viparyayāt*, "by reversal", and
the question is whether the reversal applies to the degree quanta as well as to
the lords. **Eight sources say it does.** One dissent was found: Sarajit
Poddar keeps the odd boundaries and reverses only the lord order — but his own
page contradicts itself (the prose puts Venus first, the table puts Venus at
25–30°) and names Sagittarius, an odd sign, as an even-sign target. Reported as
a transcription defect rather than a rival scheme; his own teacher Sanjay Rath
publishes 5/7/8/5/5.

**The classical citation for the *sign* mapping**, which BPHS's verse does not
give, is Jataka Parijata I.37's note — and it states the parity rule as a
principle rather than a table:

> "Each of the planets other than the Sun and the Moon own two signs, one odd and
> the other even. When a planet is in an odd sign, then take the odd Thrimsamsa
> Rasi of the planet in whose Thrimsamsa the first planet lies. Thus, a planet in
> an odd sign in a Guru Trimsamsa must be placed in the Thrimsamsa kundali in
> Dhanus (an odd sign) and not in Meena, while a planet in an even sign in Guru
> Trimsamsa must be placed in Meena."

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

**Variants — and one of them is a live argument, not a footnote.**

Sanjay Rath: "There are two methods to draw a Trimsamsa (D30) Chart. We discuss
the method of Parashara." JHora ships three; its reimplementation names five —
traditional Parashari, a cyclic "parivritti" trimsamsa, a "shashtyamsa-like"
trimsamsa, an even-sign reversal, and the Somanatha alternate. Methods 2–5 are
all **equal 1° divisions occupying all twelve signs**.

The equal division has a named advocate on the record. Asked directly which of
the two to use, Ernst Wilhelm answered:

> "Lords are just lords, not for forming a chart. Form a chart by dividing the
> sign into 30."

That is a coherent position, not a mistake: the classical texts assign
trimsamsa *lords*, and Varahamihira and Kalyana Varma use trimsamsas as lords
without ever drawing a D30 chart at all — Brihat Jataka XXI, "A person born when
the Trimsamsa of Mars is occupied by the Sun"; Saravali ch. 46, "In Leo the
Trimsamsa of Mars will make the female garrulous." Note that the Sun *occupies*
a trimsamsa there quite happily. The sign mapping, and with it the ten-sign
restriction, is a property of the modern drawing convention.

**A trap in the wording.** Several translations define the *word* trimsamsa as
a 1/30 equal part and then immediately give the unequal scheme — Aiyar on Brihat
Jataka I.7, Raman ch. XI. Those are definitions of the arc, not endorsements of
equal division. Popular sites collapse the two.

The Parashari unequal scheme above is the default in JHora, in Maitreya and in
VedAstro, and is what every classical text states.

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

**Variants.** None in the classical literature. JHora's reimplementation
enumerates four — Parashari, parivritti even-reverse, parivritti cyclic,
Somanatha alternate. See §4.4: these are the generic axes applied to the Parashari rule, not separate traditions.

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

**Variants.** None in the classical literature. JHora's reimplementation
enumerates four, the same set as D40. See §4.4: these are the generic axes applied to the Parashari rule, not separate traditions.

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

The third and fourth rows are Rao's own example and BPHS's own example
respectively, both reproducing. The last two are the boundary pair.

**Variants.** None in the classical literature, and none for the sign mapping in
any source that states a rule. JHora's feature page lists no D-60 variations —
but its release notes for 7.63 record "**3 new definitions of D-60**", and its
reimplementation enumerates four: from the sign itself (default), cyclic from
Mesha, even-sign reversal from Mesha, even-sign reversal from the sign. See
§4.4.

**Nodes:** always the 7th from each other.

**Precision:** the finest of the sixteen. 30′ of longitude, 2 minutes of clock
time, and — per §6.3 — the varga where the place list's coverage gap bites
hardest.

---

## 8. Disagreements, collected

Everything this document declines to resolve, in one place. Precedent: **D-026**
(no dignity for the nodes) and the planetary-war rule — where authorities
differ, the app prints the difference or prints nothing, never a winner.

**The classical disagreements** — different authorities, different rules:

| # | Varga | The disagreement |
|---|---|---|
| 1 | **D16** | **The only one a commentator names in print.** Jataka Parijata I.36's notes: "Phaladeepika and Sarvartha Chintamani interpret the allocation of the Shodasamsa rulerships differently." What Sarvartha Chintamani's rule is remains unknown; the sign mapping Mesha/Simha/Dhanu is agreed by BPHS, Jataka Parijata, Saravali's supplement and Rao |
| 2 | D2 | **Every classical text gives lords, not signs.** Sun's hora and Moon's hora are stated by Phaladeepika, Jataka Parijata, Saravali, Brihat Jataka and BPHS alike; the step to Simha and Karka is in none of them. Six modern completions follow, of which the two best argued by JHora's author are not the standard one |
| 3 | D2 | The **Yavana hora** gives the horas to all seven grahas. Rejected for natal work by Varahamihira and Satya, accepted for prasna — Jataka Parijata I.30's notes |
| 4 | D2 | Whether Sanjay Rath's six-sign Surya/Chandra Hora is a D-2 mapping at all, or a different object |
| 5 | D3 | Parashari (1st/5th/9th) is universal in the classical literature, but the **Jagannatha** and **Somanatha** drekkanas are separate traditions with published, incompatible answers: Meena 20°–30° is Vrishchika, Meena and Tula respectively |
| 6 | D9 | Kalachakra navamsa, Krishna Mishra navamsa and a "Rath nadi navamsa" that counts backwards in watery and earthy signs are separate schemes, not transformations. Textual sources not found |
| 7 | **D30** | **Equal or unequal.** Every classical text gives the unequal 5/5/8/7/5, but every classical text uses trimsamsas as *lords* and never draws a D30 chart. Ernst Wilhelm, asked directly: "Lords are just lords, not for forming a chart. Form a chart by dividing the sign into 30." JHora ships three schemes, four of its reimplementation's five are equal 1° divisions occupying all twelve signs |
| 8 | D30 | The even-sign split. Eight sources give 5/7/8/5/5; one (Sarajit Poddar) keeps 5/5/8/7/5 and reverses only lordship, on a page that contradicts itself |
| 9 | D27 | BPHS's *verse* says only "from Aries and other movable signs"; the element assignment comes from the translator's note. The verse alone is not computable |
| 10 | Varga lagna | JHora, its reimplementation and Maitreya all divide the exact ascendant. VedAstro instead divides each **house midpoint** independently, so its divisional houses need not be twelve consecutive signs. A different model, not a different formula |

**Resolved, and recorded because the resolution is the useful part:**

| # | Was | Is |
|---|---|---|
| 11 | "Saravali gives a different D27 rule and Santhanam calls it defective" | *Saravali has no D27 rule.* The rejected rule is Santhanam's own supplementary note — `(s + p) mod 12` instead of `(3s + p) mod 12` — which he later repudiated in his BPHS. One translator, two rules, not two classics (§7.12) |

**The mechanical disagreements** — the same rule with a switch flipped, per §4.4.
Numbered `M` because the classical list above owns the bare numbers, and both are
cited from elsewhere in this document:

| # | The axis | Where it shows up |
|---|---|---|
| M1 | Parivritti / cyclic | offered for nearly every varga by JHora and by Maitreya ("Continuous") |
| M2 | Even-sign reversal | JHora 7.63: "one new definition of D-2, D-3, D-12, D-16, D-20 and D-27, 2 new definitions of D-7 and D-24 and 3 new definitions of D-10 and D-60. The alternative definitions are mostly related to reversal of divisions in an even sign" |
| M3 | Somanatha alternate | odd forward from Mesha, even backward from Meena; offered for nearly every varga |
| M4 | D30 degree-within-sign | JHora 7.51 offers two ways to stretch an unequal segment across a sign. Maitreya silently runs even signs backwards. Only matters if a varga chart prints degrees |

**Disagreements inside a single source**, recorded because they are a caution
about the method. Numbered `S`, for the same reason:

| # | Where |
|---|---|
| S1 | JHora's feature page says three D-9 variants; its 7.4 release note adds a fourth. Its feature page lists no D-7, D-10, D-12, D-16, D-20, D-24, D-27 or D-60 variants; its 7.63 release note adds several of each |
| S2 | BPHS ch. 6's D-27 verse and Santhanam's note to it differ in what they specify (§7.12) |
| S3 | Santhanam's BPHS footnote attributes a D-27 rule to *Saravali*; *Saravali* has no D-27 rule, and the rule is his own supplementary note (§7.12) |
| S4 | Narasimha Rao's D-27 worked example contradicts his own rule (§7.12) |
| S5 | Maitreya's "Continuous" D-4 computes a D-2. Read from its source; present in both the original and a fork |

Position taken: **the first pass implements the Parashari rule only, and names
it.** A varga chart should say which scheme produced it, for the same reason
`chakra.rs` carries `place` — a D-2 computed one way looks exactly like a D-2
computed another, and there is no mark on the chart to tell them apart.

---

## 9. Unverified

Entries here are gaps, not omissions. Nothing below was filled by
reconstruction.

| # | What is unknown |
|---|---|
| 1 | **Sarvartha Chintamani's D16 rule.** Known only through Jataka Parijata's report that it differs. Phaladeepika's D16 allocation is stated but as deities, not as signs |
| 2 | The Kashinatha D-2 rule where a rashi's lord owns only one sign — Simha under the Moon's hora, Karka under the Sun's. Raman's scheme states its equivalent exception ("for Moon, both day and night signs are Cancer and for Sun, both day and night signs are Leo"); Kashinatha's does not |
| 3 | B.V. Raman's hora rule **from Raman's own text**. It is described and implemented in software but no page in Raman was located |
| 4 | The Yavana hora's actual sloka — Jataka Parijata's note refers to it without reproducing it here |
| 5 | Whether Sanjay Rath's six-sign Surya/Chandra Hora is a D-2 mapping rule or a different object |
| 6 | The **textual** sources for the Somanatha and Jagannatha drekkanas. Their rules are known here only from software and from JHora's author; the underlying works (Rath's *Upadesa Sutras*, Rangacharya's *Jaimini Sutramritam*, Somanatha's own text) were not consulted |
| 7 | The rules of the Kalachakra navamsa and the non-uniform Krishna Mishra navamsa |
| 8 | Which of the generic axes is JHora's second D-4 variant |
| 9 | The rules of JHora's second and third D-30 variants beyond the bare names in its reimplementation |
| 10 | A second authority, in words, for "only ten signs can be occupied in D30". It follows by construction from tables four sources agree on, and was verified here by enumeration, but only a calculator site states it outright |
| 11 | The numeric spread between the ayanamsas the app offers (§6.4). Measurable in-app with `Engine::ayanamsa`; not quoted here because no source table was found |
| 12 | Whether any tradition places Rahu or Ketu in a varga by a rule other than the one used for a graha. Searched for specifically. No classical text addresses the nodes in vargas at all; four independent implementations have no node branch in any varga function; the only "reverse" doctrines found attach to nodal *aspects* and to certain *dasas*. **Do not put a node reversal in the implementation on this evidence** |
| 13 | What AstroSage, Prokerala or Drik Panchang actually compute. None publishes a varga rule or offers a variant selector. Determining their behaviour would mean running known data through each and comparing — not done |
| 14 | Whether Narasimha Rao's D-40, D-45 and D-60 timing figures follow from his own stated ones. He gives the rate ("lagna moves 1′ in 4 sec") and works D-10 and D-24 ("D-10 in 12 min […] D-24 in 5 min") but never writes the finer three. §6.1's numbers for those are this document's arithmetic, not his |
| 15 | K.S. Charak, *Elements of Vedic Astrology* — the source VedAstro's tables claim to follow. The archive.org scan's OCR is unusable |

---

## 10. Sources

Ordered by weight. A rule was accepted only when a primary text and at least one
independent statement agreed; where only one source exists, §9 says so.

### Primary text

| Source | Supports |
|---|---|
| [Brihat Parasara Hora Sastra Vol. I, tr. R. Santhanam, Ranjan Publications, Delhi 1992 — archive.org scan](https://archive.org/details/heag_brihat-parasara-hora-sastra-vol-1-by-maharshi-parasara-commentary-editor-tr) | The primary text. Ch. 4 v. 5–11 (sign classifications, elements, day/night strength), ch. 6 v. 2–41 (all sixteen varga rules, speculums and worked examples), ch. 7 v. 1–8 (what each varga is read for) and v. 42–53 (the varga groupings), plus Santhanam's notes on the Sun and Moon in D30 and on Saravali's D27 |
| [Brihat Parashara Hora Shastra — a second archive.org scan of the same Santhanam translation](https://archive.org/details/BPHSEnglish) | Cross-check. Used where the first scan's OCR is unreadable; the two agree wherever both are legible |

### The other classical texts

Each was read in full, not quoted from a secondary source. Two of them —
Phaladeepika and Jataka Parijata — present the **dasavarga**, ten divisions, not
the sixteen; Saravali's verses cover the dasavarga too, and its remaining six
come from Santhanam's supplement. Jataka Parijata's notes do list all sixteen by
name, which is an independent attestation of the list itself.

| Source | Supports |
|---|---|
| [Brihat Jataka, tr. Swami Vijnananda](https://archive.org/stream/BrihatJatakaOfVarahamihiraBySwamiVijnananda/Brihat%20Jataka%20of%20Varahamihira%20By%20Swami%20Vijnananda_djvu.txt) · [tr. N. Chidambaram Aiyar](https://archive.org/stream/brihatjataka00varaiala/brihatjataka00varaiala_djvu.txt) | I.11 — the sign classifications by alternation, the odd/even Sanskrit (*ayuji* / *samabhe*), the direction trines, the hora lords, the drekkana lords. I.14 — navamsa by vargottama. I.7 with Aiyar's note — the D30 degree split for both parities. Ch. XXI and XXIV — trimsamsas used as lords with the Sun occupying one |
| [Jataka Parijata Vol. I, tr. V. Subrahmanya Sastri](https://archive.org/stream/JatakaParijataVolIOfIIByVSubrahmanyaSastri/Jataka%20Parijata%20Vol%20I%20of%20II%20by%20V%20Subrahmanya%20Sastri_djvu.txt) | I.30 — hora and drekkana lords, **and the Yavana hora dissent**. I.31 — saptamsa, "of Lagna and other houses". I.32–34 — navamsa in both formulations. I.35 — dasamsa, dwadasamsa. **I.36 — the named D16 disagreement with Phaladeepika and Sarvartha Chintamani**, and the 192-segment continuous statement. **I.37 with its note — the D30 degree split and the classical statement of the sign mapping by parity** |
| [Phaladeepika, tr. V. Subrahmanya Sastri](https://jyotishvidya.com/HTMLobj-9415/Mantreswara_s__Phaladeeplka_.pdf) | 3:4 — hora, drekkana, dwadasamsa, navamsa start signs, D30 split with both parities spelled out. 3:6 — saptamsa, dasamsa, and the D16 allocation that Jataka Parijata reports as differing. 3:11 — the navamsa and drekkana **of the ascendant** |
| [Saravali Vol. I, tr. R. Santhanam](https://static1.squarespace.com/static/5e5b12e392faf542399f9528/t/6444ef10e62dac013a8d316a/1682239298930/Saravali+-+R+Santhanam+-+Vol-1.pdf) | 3.11–3.18 — navamsa, dwadasamsa, vargottama, hora, D30, saptamsa as a twelve-sign enumeration, and **v. 18's generic formula for computing any varga**. Santhanam's supplementary notes — D16, D20, D24, D27, D40, D45, D60, including **the repudiated D27 rule of §7.12** |
| [B.V. Raman, *Hindu Predictive Astrology*, ch. XI](https://archive.org/stream/hindupredictiveastrologyofbvraman/Hindu%20Predictive%20Astrology%20of%20B%20V%20Raman_djvu.txt) | The D30 table for both parities, 5/5/8/7/5 and 5/7/8/5/5 |
| [Hart deFouw & Robert Svoboda, *Light on Life*](https://storage.yandexcloud.net/j108/library/1h8hzjau/Robert_Svoboda_-_Light_on_life_An_Introduction_to_the_Astrology_of_India.pdf) | Ch. 5 — the movable/fixed/dual and the four element groups, enumerated in the text. Ch. on navamsa — **the explicit statement that the class formulation and the element formulation are the same rule**, and a full worked procedure that fixes the varga ascendant first |

### Independent statements of the rules

| Source | Supports |
|---|---|
| [P.V.R. Narasimha Rao, *Vedic Astrology: An Integrated Approach*, ch. 6 "Divisional Charts"](https://www.vedicastrologer.org/articles/vedic_astro_textbook.pdf) | All sixteen rules restated in modern notation with worked examples; D9 stated by element rather than by class; the D30 degree ranges; the D60 procedure; the statement that lagna and special lagnas divide like planets. Contains the D27 erratum recorded in §7.12. Ch. 32 supplies the birth-time-error figures in §6.1 |
| [Sanjay Rath, "Principles of Divisional Charts"](https://srath.com/jyoti%E1%B9%A3a/principles-of-divisional-charts/) | The drekkana rule stated independently; a worked example applying it **to the lagna**; the six-sign Surya/Chandra Hora conception |
| [Sanjay Rath, "Trimsamsa D-30 Chart"](https://srath.com/jyoti%E1%B9%A3a/varga/trimsamsa-d-30-chart/) | Third independent statement of the D30 table, in cumulative-degree form; that the Sun, Moon, Rahu and Ketu own no trimsamsa; that the lagna is divided like a planet; and that a second D30 method exists which he does not give |
| [Ernst Wilhelm, forum answer on D30](https://astrology-videos.com/forum/general-predictive-astrology/trimsamsa-d30-calculation-rashi-30-or-using-planetary-lords) | The equal-1° trimsamsa position, on the record and unhedged: "Lords are just lords, not for forming a chart. Form a chart by dividing the sign into 30." |
| [Vijayalur, "Drekkana / Dreshkana / Decanate"](https://vijayalur.com/2011/05/19/drekkana-dreshkana-decante/) | Independent statement of the Parashari drekkana, and that there are four kinds — while declining to state the other three |
| [Barbara Pijan Lama, "D-27 Bhamsha"](https://barbarapijan.com/bpa/Varga/D27_Bhamsha.htm) | Independent statement of the D27 segment size and starting rule, in BPHS's own phrasing |
| [DesiUtils — Trimsamsa D30 calculator](https://desiutils.in/astrology/trimsamsa-d30) | Fourth statement of the D30 table, as published calculator behaviour, including the ten-sign restriction stated outright |

### The variants

| Source | Supports |
|---|---|
| [Features of Jagannatha Hora](https://www.vedicastrologer.org/jh/features.htm) | The variant counts, verbatim: six D-2, four D-3, two D-4, three D-9, three D-30; 23 divisional charts; mean or true nodes; seven named ayanamsas; and the birth-time tool that "the time to be subtracted or added in order to change the lagna in the divisional chart is displayed at one mouse click" |
| JHora release notes — [7.32](https://www.vedicastrologer.org/jh/update_7.32.htm), [7.4](https://www.vedicastrologer.org/jh/update_7.4.htm), [7.5](https://www.vedicastrologer.org/jh/update_7.5.htm), [7.51](https://www.vedicastrologer.org/jh/update_7.51.htm), [7.63](https://www.vedicastrologer.org/jh/update_7.63.htm) | The navamsa variant history; the uniform Krishna Mishra rule; the generic custom D-N structures; the D-30 divisional-longitude option; and 7.63's "reversal of divisions in an even sign" additions that the feature page never absorbed |
| [P.V.R. Narasimha Rao, "Using Kashinatha Hora Chart"](https://jyotish-blog.blogspot.com/2008/06/using-kashinatha-hora-chart.html) · ["Parasara's Hora Chart Decoded"](https://blog.indianastrologysoftware.com/parasaras-hora-chart-decoded/) | The Cancer–Leo rule stated as a rule; Parivritti Dwaya; Kashinatha with a worked example; Uma-Shambhu; and the four-way enumeration of the D-2 disagreement with the author's objection to each |
| [P.V.R. Narasimha Rao, "What is Dreshkona?"](https://jyotish-blog.blogspot.com/2005/03/what-is-dreshkona.html) · ["Jagannatha Drekkana"](https://jyotish-blog.blogspot.com/2005/03/jagannatha-drekkana.html) · ["Somnatha Drekkana"](https://jyotish-blog.blogspot.com/2005/02/somnatha-drekkana.html) | The four D-3 schemes by JHora's own UI labels; the Jagannatha mapping in full; the Somanatha description; and the three-way worked disagreement on Meena 20°–30° |

### Implementations read as evidence

Software is corroboration of what practitioners compute, not of what a text
says. Each is flagged for how far it can be trusted.

| Source | Weight | Supports |
|---|---|---|
| [Maitreya 8 — source](https://github.com/martin-pe/maitreya8), [docs](https://saravali.github.io/) | **Strong.** Open source, read directly; its rules are in `src/jyotish/Varga.cpp` | Independent implementation of all sixteen Parashari rules; only three variant switches, all undocumented; the D-30 even-sign degree reversal; the node display-degree reversal; and the "Continuous" D-4 defect |
| [PyJHora](https://github.com/naturalstupid/PyJHora) | **Medium.** A reimplementation of JHora, not JHora. Its own README says features "outside of his book but in his JHora software were collected from various internet sources" | The most complete machine-readable enumeration of variant names in existence; the generic parivritti / even-reverse / Somanatha algorithms with docstrings; and confirmation that no varga function anywhere branches on Rahu or Ketu |
| [VedAstro](https://github.com/VedAstro/VedAstro) | **Weak — use only as a third opinion.** Its varga tables were generated by a language model, per the authors' own commit messages ("thanks to GPT4-32K & GPT-4"), and the functions that consume them are absent from the published source | Agrees with the Parashari rule for every varga including the unequal D30; but its varga-lagna-by-house-midpoint model is a real design divergence (§5) |
| [Swiss Ephemeris — programmer's docs](https://www.astro.com/swisseph/swephprg.htm) | — | Computes no vargas at all. Establishes that §7 is entirely ours to implement (§6.5) |

### Not used

`jagannathhora.com` and `jagannathahora.com` carry JHora-branded varga content
and are **not** authored by P.V.R. Narasimha Rao. They appear high in search
results for every query in this document and were excluded on that ground.
AstroSage, Prokerala and Drik Panchang list the sixteen with significations but
publish no computation rule; AstroSage says so outright — "I will not explore
the mathematics behind these divisional charts."

---

In-repo references: `docs/DECISIONS.md` D-003 (ayanamsa, `Ketu = Rahu + 180°`),
D-006 (provenance), D-026 (no dignity for the nodes); `docs/TODO.md` E1
(the D1 chart as built), E5 (the place list, 418 places at arcminute
precision), and the "cited, not remembered" note under Panchanga;
`docs/design/kundali.md` §1.2 (the lagna's rate of motion), §3 (the three chart
formats); `crates/almanac/src/chakra.rs` (the `Chakra` payload the varga charts
would extend); `crates/almanac/src/zodiac.rs` (`Rashi::index`,
`degrees_in_rashi`, `pada`).
