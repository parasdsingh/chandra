# The Lagna Kundali, read in five seconds

Status: **proposed.** Nothing here is built. Answers: "UX looks a bit off; compare with
drikpanchang, and jagganath hora. there are other aspects which make UX odd for the macOS top
bar context that we have", and the earlier report that "kundali title bar and subtitle are not
formatted properly. neither are they consistent with calendar windows, nor with the kundali
window's own UX".

Specification the feature was built to: [`kundali.md`](kundali.md). Where that document and the
shipped code disagree, this one records which won and whether it should have.

Works under [D-006](../DECISIONS.md#d-006), [D-011](../DECISIONS.md#d-011),
[D-014](../DECISIONS.md#d-014), [D-020](../DECISIONS.md#d-020),
[D-023](../DECISIONS.md#d-023), [D-024](../DECISIONS.md#d-024).
Expressed in the vocabulary of [DESIGN.md](../DESIGN.md) §2, §3, §4, §5, §9, §11.

Every claim about the current code cites a file and a line. Every claim about a reference
application cites a URL or a file I fetched and read. Anything I could not check says
**unverified**.

---

## Decision summary

- **The chart is not the problem. The frame around it is.** The three renderers in
  `Chakra.tsx` are sound and the geometry is derived rather than tabulated. Almost everything
  that reads as "off" is in the four elements that sit around the drawing: the header title,
  the caption, the panel's height, and four new type styles that are not in the type scale.
- **The header names the feature; every other header in the app names a value.** The calendar
  says `Chandra · August 2026`, the day view says `Shukla Ashtami`, settings says the section.
  The chart says `Lagna Kundali` — a label for the view, not a reading of it. That is the
  inconsistency the user reported, and it is the one thing on the surface that a second
  glance cannot recover from.
- **The caption translates the title instead of placing the chart.** `ASCENDANT CHART · 02:51
  PM · DHANU 12°38′` spends its first and widest token restating the header in English, and
  buries the lagna's degree — the most volatile and most valuable number in the view — at the
  end of a 10px uppercase grey line. The day view's date line, which this is modelled on,
  never carries a value; values live at `--type-value`, 15px.
- **The place is missing, and `kundali.md` §4.4 already argued that it must not be.** The lagna
  moves 1° per four minutes of clock time, so a longitude 300 km out moves it about 3°.
  `Bootstrap.location` is a `Resolved` (`src/ipc/types.ts:521`) carrying both `label`
  (`:488`) and `provenance` (`:499`), and neither reaches the chart. **Both references print
  the place beside every chart** (§1.4, §2.3). This is the single largest omission.
- **The panel growing to 414 should stay — but it should stop being a resize.** Fitting a
  square chart into the 264px region puts its type at 0.83× at the Default scale and 0.66× at
  Compact, below the floor DESIGN §2.5 sets for the app's smallest labels. The height is
  right; the mechanism is wrong. It is currently decided by the front end *after* the window
  has been shown (`Panel.tsx:669-671` → `panel.rs:419`), so the first frame of every chart open
  is drawn at the previous height. It should be decided from the tray subject in
  `panel::toggle` before `show()`, which makes 414 a property of that status item rather than a
  transition (§5.5, §6). Jagannatha Hora is the worked example of choosing the other way — it
  keeps the type and shrinks the chart, and its own screenshots show compartments overlapping
  and clipping (§2.7).
- **Four type styles were invented and none is in DESIGN §3.3**: 9px/400, 10px/500, 10px/800,
  10px/400. The 9px rashi label is the smallest type in the app and its colour,
  `--chart-label` `#7f9bb5`, measures about **4.0:1** on the worst-case composited ground —
  below the 4.5:1 floor that retired `--text-tertiary` (§3.7). Same test, same verdict.
- **`.chakra__graha.is-own-sign` sets an underline** (`panel.css:992-996`). D-024 is titled
  "No underlines" and its body says "**No underlines anywhere**". DESIGN §3.3 repeats it. This
  is a direct contravention, not a grey area.
- **The chart's outer frame renders at half the weight of its internal lines.** The outer
  polygon's points sit on the viewBox edge, so half of its centred 1px stroke is clipped
  (§3.5). That is most of why the drawing reads as bleeding off the window rather than sitting
  in it.
- **Recommended layout: "the header carries the reading."** Header `Dhanu Lagna`, caption
  `12°38′ · 02:51 PM · PATAN`, chart 318 × 318 with its frame at full weight, panel 414 set
  from the tray item. Mockup at true proportions in §5.4.
- **Both references write a *number* in the North Indian compartment, and the build writes a
  name.** Drik Panchang prints the rashi numeral by default with names as an opt-in at half
  opacity (§1.2); Jagannatha Hora prints green digits 1–12 at each house's inner vertex (§2.2).
  They agree exactly, and against the current implementation. This is a decision for the user,
  not a defect — §9 Q5.
- **Rejected outright: Drik Panchang's per-graha colour coding.** Nine hard-coded fills, one
  per planet, verified in their SVG (§1.7). It is the clearest example of colour encoding a
  value, which D-023 forbids, and it is the one reference decision that would look most
  "authentic" if copied.

---

## 1. Drik Panchang

Read from the server-rendered HTML and inline SVG of
`https://www.drikpanchang.com/jyotisha/kundali/kundali.html`, fetched 30 August 2026. The chart
is inline `<svg>` in the document, not an image and not JS-generated, so everything below is
read from markup rather than from pixels.

### 1.1 The chart is a 3:2 rectangle, not a square

```
<svg id="id_147c7e" viewBox="0 0 400 266.6667" …>
```

Wrapped in `<div class="dpEmbeddedContainer dpRatio3By2">` with
`.dpRatio3By2 { padding-bottom: 66.8% }` and `.dpKundaliWrapper { max-width: 400px }`. So the
rendered chart is at most **400 × 267 CSS px**, landscape.

This matters more than it looks. Chandra's whole 414px problem exists because the chart was
assumed to be square. The reference the user named draws a North Indian chart 1.5 times wider
than it is tall, and the format survives it: the diamond stretches, the compartments stay
readable, and nothing about the construction breaks. `Chakra.tsx`'s geometry is already derived
from the box (`northCompartments` builds every vertex from `s`, `half`, `quarter`), so it would
generalise to a rectangle by taking a width and a height instead of one `SIZE`.

Drik Panchang also draws the compartment boundaries as **quadratic Béziers**, not straight
lines — `d="M 200,0 Q 200,36.3636 241.665,38.095 Q 283.33,39.8264 283.33,76.19 …"` — giving
the outer houses a bowed edge. That is a stylistic choice, not a structural one, and Chandra
should not copy it: a curve at 318px costs legibility in the corner triangles for no gain.

### 1.2 What is inside the chart face

Every `<text>` element in the D1 chart SVG, exhaustively:

| Content | Class | Size | Family | Fill |
|---|---|---|---|---|
| `9 10 11 12 1 2 3 4 5 6 7 8` — one per compartment | `cls_b8543a` | 13px | monospace | `#540000` |
| `Sur Cha Man Bud Gur Shu Sha Rahu Ketu` | `cls_4abf85` | 15px | monospace | nine different fills — §1.7 |
| `Drikp.com` — a watermark near the centre | `cls_6cd244` | 12px | monospace | `#DAAB39` |

That is the entire text content of the chart. Nothing else.

- **Sign identity is a number, not a name.** In the North chart the numeral is the *rashi*
  number and house 1 (the fixed top diamond) carried `9` — Dhanu, confirmed by the page's own
  Bhava table. In the South chart the numeral is the *bhava* number. The numeral always
  supplies whichever of the two dimensions position does not.
- Sign **names** are available but **off by default**, behind a toolbar item "Show Rashi Name
  in Chart" (cookie `drik-chart-rashi-tags-status`), added as four-letter abbreviations at 12px
  and `opacity: .5` in the opposite corner of the cell from the numeral.
- **Graha labels are three-letter Latin transliterations of the Sanskrit names**, with Rahu and
  Ketu written out in full. Two other modes exist and are server-rendered: `?lang=hi` gives
  full Devanagari (`सूर्य चन्द्र मंगल …`), and a Western toggle gives `Sun Moon Mar Mer Jup Ven
  Sat Rahu Ketu`.
- **No degrees.** Not one numeric position inside any square chart.
- **No retrograde mark, no combustion mark, no dignity mark.** In the sample I fetched, Shani,
  Rahu and Ketu were all flagged retrograde in the table below and none of the three carried
  any mark in the North, South or East chart. Only the West (wheel) style draws retrograde, as
  `⤿` at 40px on the planet's spoke.
- **No ascendant marker at all.** No `As`, no `Lg`, no stroke, no fill. The lagna is carried
  entirely by convention plus the numeral: house 1 is the top diamond, and the number in it
  names the rising sign.

### 1.3 Proportions of the type

The graha label is 15px in a 400px-wide chart: **3.75% of the chart's width**, for a
three-letter string. Chandra's is 10px in 318 — **3.14%**, for a two-letter string. The
reference types its chart noticeably larger relative to the drawing than Chandra does.

### 1.4 What sits around the chart

Top to bottom, per chart block, and there are two blocks side by side (D1 and D9):

1. A chart-style row: `North` `South` `East` `West` as inline text buttons.
2. An anchor-graha `<select>`: `− Lagna` selected, then the nine grahas and three outers.
3. A division `<select>`: 21 options, `BC - Bhav Chalit`, `D1 - Rashi` … `D60`.
4. A two-line caption block, `class="dpImageCaption"` — in my fetch,
   `Aug 30, 2026 at 03:29 PM` and `Patan, Nepal`. **The instant and the place, and nothing
   else.**
5. The chart.
6. A one-line description under it: `Body, Physical Matters and all General Matters` for D1.

There is **no legend and no key** anywhere on the page. The `(Q)` and `(T)` markers beside
graha and bhava names in the tables are never explained.

### 1.5 The tables, which is where every qualifier lives

Three per chart, all default-visible.

**Graha Details** — nine columns:
`Graha` · `Longitude` · `Nakshatra` · `Nakshatra Lord/Sub Lord` · `Ruler of` · `Is In` ·
`B. Owner` · `Relationship` · `Dignities`

Row 1 is always the ascendant, labelled `Lagna`. Longitude is formatted
`12° **Simh** 48′ 29″` — degrees, a bolded sign abbreviation, arcminutes, arcseconds; always
sign-relative. Retrograde `↺`, combustion `🔥`, Sthana Parivartana `💱` and Pushkaramsha `🌸`
all appear **here and only here**, each with a `title` tooltip and a link.

**Bhava Details** — six columns:
`Bhava` · `Residents` · `Owner` · `Rashi` · `Qualities` · `Aspected By`

**Upagraha** — four columns: `Upagraha` · `Longitude` · `Nakshatra/Swami` · `Raw Longitude`.

### 1.6 The Lagna table

`https://www.drikpanchang.com/muhurat/lagna.html` — `Udaya Lagna with Pushkara Navamsha for
Delhi, NCT, India`. (The URL in the brief, `/lagna/lagna-timings.html`, is a 404.)

**Two columns**: `Lagna Rashi` and `Lagna Time`. Twelve rows, in **rising order starting from
the day's first lagna**, not from Mesha. Above the table sits a "Running Lagna" card: a rashi
ideogram, the sign name, the span `02:15 PM to 04:19 PM`, a live JS countdown, the place, and
the date. **There is no chart on this page at all.**

The card is the interesting part: it is Drik Panchang's own answer to "what does someone want
in one glance" for exactly the quantity Chandra's chart is about. The answer is *the sign, the
span it is running for, and how long is left* — and the chart is not in it.

### 1.7 Colour

**Colour encodes graha identity.** Nine hard-coded fills, read directly out of the SVG:

| Graha | fill | Graha | fill | Graha | fill |
|---|---|---|---|---|---|
| Sur | `red` | Cha | `white` | Man | `#BB0000` |
| Bud | `#7B0000` | Gur | `#A52583` | Shu | `green` |
| Sha | `#4B4B4B` | Rahu | `black` | Ketu | `brown` |

Chart chrome is one fixed palette and carries nothing: ground `#EFB861`, every stroke
`#B80000`, numerals `#540000`. There is no dark variant; the palette is baked into the markup.

On the lagna table, colour encodes again — each rashi's name cell takes one of six classes and
the mapping is exactly its dispositor: `dpLightRed` for Simha (Surya), `dpDarkGreen` for Kanya
and Mithuna (Budha), `dpMangoYellow` for Dhanu and Meena (Guru), and so on.

### 1.8 Unverified

Rendered appearance — I read markup, not pixels, so font fallback for `monospace` and how
`fill="white"` Chandra actually reads on `#EFB861` are unchecked. Collision handling for five
or more grahas in one house is unverified: the sample had at most three.

---

## 2. Jagannatha Hora

Primary evidence, all downloaded and read as pixels or fetched as text:

| Source | What it is |
|---|---|
| `vedicastrologer.org/images/jhshot.jpg` (900 × 546) | the official screenshot, linked from `/jh/jhshot.htm`; an older release |
| `softportal.com` → `jagannatha-hora-big-1.png` … `-4.png` (2560 × 1400) | four screenshots of **JH 8.0**, including one with the chart right-click menu open |
| `indianastrology08.blogspot.com` → `ss4.png`, `SS5.png`–`SS9.png` (1600 × 900) | JH 7.x opening page, the Preferences menu, the chart-style dialog, and North Indian mode |
| `vedicastrologer.org/jh/features.htm` | the official feature list |
| `vedicastrologer.org/jh/update_7.32.htm`, `_7.33.htm`, `_7.6.htm` | official release notes |

### 2.1 The window, and the one assumption a popover cannot honour

JH 8.0 opens on the **`Basics`** tab — verified in two independent screenshots, v7.x and v8.0.
That landing view is the designer's own answer to "what do you look at first", and it holds
four things at once:

- **Left third: a scrollable 2 × 3 column of six chart squares** — Rasi, Navamsa (D-9),
  Trimsamsa (D-30), Drekkana (D-3), Dasamsa (D-10), Shashtyamsa (D-60).
- **Right, upper: the planetary-positions table** (§2.4).
- **Right, lower left: a `Natal Chart:` key-info block** — Date, Time, Time Zone, Place, Lunar
  Yr-Mo, Tithi with % left, Vedic Weekday, Nakshatra with % left, Yoga, Karana, Hora Lord,
  Sunrise, Sunset, Ayanamsa, Sid Time, and more.
- **Right, lower right: a Bhinna ashtakavarga chart.**

Above all of it: a menu bar (`File Edit Modes View Preferences Websites MoreMenus Help`), an
icon toolbar, a nine-item tab strip (`Chakras · Basics · Strengths · Dasas · Transits · Tajaka ·
Tithi Pravesha · Mundane · Miscellany`), a second tab strip below (`Key Info · Houses · Amsa
rulers · KP`), and a status line.

The six-chart column is "**packed chart mode**", and the release note that introduced it states
both the rule and the gate — `update_7.32.htm`, verbatim:

> "If one has a display with a horizontal size more than 1280 (e.g. 1440x1080 or 1600x1200 or
> 1920x1080), than a "packed chart mode" is available. Next to dasa calculations etc, one can
> see 6 different divisional charts (instead of normal two). One can turn this on/off based on
> how big one's monitor is."

and `features.htm`:

> "If one has a display that is higher than 800x600, two charts are displayed with most
> calculations (rasi and navamsa by default)."

"Next to dasa calculations etc" is the load-bearing phrase: **the chart column persists on the
left while the right pane changes with the tab.** Confirmed visually — the `Strengths` tab keeps
the same six charts and replaces only the right pane, with eight bar-graph panels.

The older screenshot shows the same program as a nine-window MDI (`Rasi`, `Bhava/Chalit D-20`,
`Vimsamsa`, `Period Entry Chart`, `Transit Calendar`, `Dasa-Transit Correlation`, and four bar
graphs).

**A chart in Jagannatha Hora is never alone, at any release.** Its minimum unit of presentation
is two charts plus a calculation panel, and its default on a modern display is six. That is the
assumption a 320pt popover cannot honour and must not try to.

### 2.2 What is inside a chart square

Verified by zooming the 2560px v8.0 screenshot and cross-checking every token against the
position table in the same image. Default style is **South Indian Regular**.

- **Two-letter Latin abbreviations**, large, dark red: `Su Mo Ma Me Ju Ve Sa Ra Ke`. They own
  the middle of the cell.
- **The ascendant is `As`** — not `Lg` — drawn at planet size in magenta.
- **Retrograde is the abbreviation in parentheses.** The chart showed `(Me)`, `(Ju)`, `(Sa)`;
  the table in the same screenshot reads `Mercury (R) - PK`, `Jupiter (R) - BK`,
  `Saturn (R) - DK`, and no other planet was parenthesised. This is the same convention
  `Chakra.tsx:416-418` chose, corroborated by the reference rather than invented.
- **A subordinate tier pinned to the cell's corners**, at about half size: special lagnas and
  upagrahas in olive (`HL GL SL PP BB Md Gk`) in the top corners, arudha padas in blue
  (`A2`–`A12`, `UL`) in the bottom corners, `AL` in purple inline with the planets. Each was
  confirmed by matching its sign against the table.
- **No degrees inside any compartment.** Checked across six screenshots, two versions, and both
  chart styles.
- **South Indian cells carry no sign name and no sign number.** Position is the whole of the
  identity; the reader is assumed to know the fixed layout.
- **North Indian cells do carry the sign number** — small green digits 1–12 at each house's
  inner vertex. This is direct evidence on §9 Q5.
- **The centre block carries the chart's identity**, three lines: `Natal Chart` small, the chart
  name large (`Rasi`, `Navamsa`), then the varga code (`D-9`, `D-30`).
- **In North Indian style that block cannot exist** — the centre is occupied by house 1 — so the
  title moves to the frame's top-left and the varga code to the top-right. The reference hits
  the same constraint §5.8 R-5 records for Chandra, and answers it the same way.

Name form is a preference, not a convention JH imposes. `features.htm`: "For planet and sign
names, one can use English names such as Sun, Moon, Ar, Ta etc or Sanskrit names such as Surya,
Chandra, Mesh, Vrish etc", and "One can view planet names and chart names in charts in ten
Indian languages, including English."

### 2.3 What sits beside a chart

The `Period Entry Chart` window is the clearest small case: a chart on the left, and a
label-and-value ledger on the right reading, verbatim —

```
Date:           February 18, 1859
Time:           4:17:35
Time Zone:      5:51:00 (East of GMT)
Place:          87 E 44' 00", 22 N 53' 00"
Lunar Yr-Mo:    Kaala-yukta - Magha
Tithi:          Krishna Pratipat (Su) [Kaa… (43.82% left)
Vedic Weekday:  Thursday (Ju)
Nakshatra:      Poorva Phalguni (Ve) …
```

Labels left, values right — the same shape as Chandra's day view field stack. The `Basics`
tab's `Natal Chart:` block opens with the identical four rows. **The instant, the time zone and
the place are the first things beside every chart JH draws**, which is what the shipped Chandra
caption reduced to one token.

### 2.4 The planetary-positions table

Read off the 2560px screenshot including the right edge, to confirm nothing was cut off.
**Exactly six columns:**

`Body` · `Longitude` · `Nakshatra` · `Pada` · `Rasi` · `Nava…` *(header truncated by the column
width; the data is a two-letter navamsa sign)*

- `Longitude` is formatted `29 Sg 03' 12.50"` — degrees, a two-letter sign, arcminutes,
  arcseconds to two decimals. **Sign-relative**, exactly as Chandra's `degrees_in_rashi` is.
- **Retrograde is `(R)` inside the `Body` cell**, and the chara karaka is a suffix on the same
  cell: `Mercury (R) - PK`, `Sun - AmK`, `Rahu - AK`.
- `Nakshatra` is a four-letter abbreviation (`USha`, `PPha`, `Magh`, `Hast`).
- Rows run Lagna, the nine grahas, then Maandi, Gulika and eighteen further special points.

**There is no dignity column, no combustion column and no speed column in this default view.**

### 2.5 Dignity and combustion are opt-in, and framed as a beginner aid

This corrects an assumption worth stating plainly: JH *can* mark dignity on the chart face, but
does not by default, and says why. `features.htm`:

> "Own signs, exaltation signs, debilitation signs and moolatrikonas of planets can be
> highlighted **to aid beginners**"

Verified as off by default: in the v8.0 screenshot Saturn in Aquarius and Sun in Leo — both own
signs — render identically to every other planet.

Combustion is not on the chart face at all. `update_7.6.htm`, verbatim:

> "Planetary aspect table with relationships marked as color codes is added. … It also shows
> whether the aspecting planet is exlated or debilitated or combust or in moolatrikona or in
> own house or in a friendly house etc."

So the reference's position is: **position on the face, qualifiers in a table, dignity available
on the face only as scaffolding for people still learning.** That is a materially better
argument for §5.2's recommendation than "nobody draws it" would have been.

### 2.6 Colour

Colour encodes the **category of the token**, never its value:

| Colour | Encodes |
|---|---|
| dark red, large | the nine planets |
| magenta | `As` |
| purple | `AL` |
| blue, small | arudha padas, `UL` |
| olive, small | upagrahas and special lagnas |
| green, small | sign numbers, North Indian style only |

And it is fully user-configurable — `features.htm`: "One can customize all the colors used in
the software", with the entry point verified in the menu as Preferences → Related to Display →
`Colors to be used`. **No colour in JH is a code the software teaches**, which is the line D-023
draws. Font size is adjustable independently of chart size, by toolbar buttons and a Preferences
submenu.

### 2.7 What JH does when a chart gets small — it collides

In the six-chart packed column at 2560px, crowded compartments **overlap and clip**: `Mo Su (Me`
runs past the cell boundary, `Ve Ma` and `(Sa) Ra` overprint. JH does not reflow, shrink or
abbreviate to fit; it packs and lets the glyphs collide.

That is the failure mode Chandra's `cluster()` (`Chakra.tsx:119-154`) exists to prevent, and it
is a verified example of what the alternative looks like. It is also evidence for §6: the
reference's own answer to "the chart is too small for its contents" is to gate the packed mode
on a 1280px display and offer a `Turn OFF packed chart mode` item, rather than to make the type
smaller.

### 2.8 Unverified

| Item | Why |
|---|---|
| Whether degrees can be shown inside compartments | the right-click menu has an item named `Chart display mode ▸`, and no screenshot or document captures its submenu |
| The contents of `Highlight rasi(s) ▸`, `Relationships ▸`, `Choose a view ▸` | same |
| The `Dasas` tab's layout | no screenshot found |
| The `Houses` / `Amsa rulers` / `KP` table columns | no screenshot found |
| Official help text | JH's help is a legacy WinHelp file inside the installer and is not published on the web |

One caution for anyone extending this research: **PyJHora is not a faithful UI clone of JH** and
must not be used to infer its conventions. Its `chart_styles.py` uses `'L'` for the ascendant
where JH uses `'As'`.

---

## 3. What is off in the current implementation

Ordered by how much of the reported problem each accounts for.

### 3.1 The header names the view, not the reading

`src/components/Panel.tsx:635`

```tsx
if (view() === "chart") return "Lagna Kundali";
```

Every other title in the panel is a value:

| View | Title | Kind |
|---|---|---|
| Calendar | `Chandra · August 2026` | subject and the scope in view |
| Day, lunar | `Shukla Ashtami` | what the day is called (`Panel.tsx:648-657`) |
| Day, solar | the civil date (`Header.tsx:50-55`) | what the day is called |
| Settings | the section name (`SettingsView.tsx:52`) | where you are |
| **Chart** | **`Lagna Kundali`** | **what the feature is called** |

The chart is the only one that names its own genre. The identity of the panel is already
carried twice over without it: the leading slot holds `ChartMark` (`Header.tsx:255-275`), the
same two-square mark the status item uses, and the settings section is itself titled
`Lagna Kundali` (`SettingsView.tsx:52`). What is missing from the header is the reading.

### 3.2 The caption translates the title and buries the value

`src/components/Panel.tsx:816-829` renders

```
ASCENDANT CHART · 02:51 PM · DHANU 12°38′
```

Three problems, in order of size.

1. **`ASCENDANT CHART` is a gloss on the header, not a location for the chart.** The day view's
   date line — which `panel.css:1012-1015` says this is modelled on — reads
   `TODAY · THURSDAY · 20 Aug · Guruvara` (`DayDetail.tsx:166-179`). It places the day; it
   never restates the header. Fifteen of the line's characters are spent saying in English what
   the line above says in Sanskrit.
2. **`DHANU 12°38′` is the most valuable number in the view and it is set at the smallest type
   in it**, in `--text-secondary`, uppercased, at the end of the line. DESIGN §3.3 puts values
   at `--type-value`, 15px, primary. The day view has no number in its date line at all.
3. **There is no place.** `kundali.md` §4.4 states why that is not a matter of taste:

   > "Line 3 is not decoration. The lagna is the first value in this app whose correctness
   > depends materially on which step of the D-007 chain resolved the location."

   `Bootstrap.location` (`src/ipc/types.ts:521`) is a `Resolved` carrying both `label`
   (`:488`) and `provenance` (`:499`), and the chart reads neither. Both references print the
   place beside every chart (§1.4, §2.3).

Two smaller notes on the same line. The `°` and `′` are set in micro-caps with `+0.6px`
tracking (`panel.css:1019-1027`), which is a treatment designed for words. And
`panel.css:1017-1018` claims the 16px padding "puts it under the header's text" — the header's
text starts at x = 48 (`panel.css:88-96`, `24px 1fr 24px` with an 8px gap inside 16px padding),
so it does not; it aligns to the content column, which is the right thing to do but not what
the comment says.

### 3.3 The panel's height changes, after the window is already on screen

`src/components/Panel.tsx:669-671`

```tsx
createEffect(() => {
  void ipc.setPanelTall(view() === "chart");
});
```

reaching `src-tauri/src/panel.rs:419-443`. The sequencing is:

| Step | Where | Height |
|---|---|---|
| Tray item clicked; window placed and shown | `panel.rs:191-218` | whatever it was last |
| `chandra://open` reaches the page | `panel.rs:211` | unchanged |
| The page renders the chart at `.panel-frame.is-chart`, 414 | `panel.css:169-175` | window still unchanged |
| The effect fires; IPC; `set_tall` resizes | `Panel.tsx:669` → `panel.rs:439` | now 414 |

So the first painted frame of a chart opened after any graha panel is a 414px layout inside a
332px window, clipped 82px at the foot. `PanelTall` is app state and survives a hide
(`panel.rs:398-412`), so the reverse also happens: after a chart, the next graha open shows a
332px calendar inside a 414px window until the effect shrinks it. **The duration of the
mismatch is one render plus one IPC round trip and I have not measured it** — but the ordering
is structural, not a race.

The resize is anchored at the top (`panel.rs:309` fixes `y` to the work area's top edge plus
6), so growth and shrinkage both happen at the bottom edge, and DESIGN §2.2 forbids animating
a frame, so it is a step.

`set_tall` also carries a bug already paid for:

> "`apply_scale` also re-cuts the material, and re-applying the vibrancy view drops the
> window's key status - which the focus handler reads as the user clicking away, so it hides
> the panel. Opening settings from the chart made the whole panel disappear." — `panel.rs:433-437`

That is the cost of making one view own the window's size, and it will be paid again by
whatever changes next.

Three documents and one comment are now stale because of this:

- DESIGN §2.2: "Height **332px**, constant" and "**The height is constant, and that is the
  whole point.**"
- `Panel.tsx:5-7`: "Nothing opens a second window and **nothing changes the panel's height**,
  so the popover never resizes under the pointer."
- `panel.css:6-8`: "The window is exactly the panel's size, **because the panel's height is
  constant**" — contradicted eleven lines later by the comment at `panel.css:16-19`.

### 3.4 The chart is 314px, not 318

`panel.rs:31-34` states the budget as

> "region is 346: 14 caption + 2 + 318 chart + 12 padding"

The CSS produces something else. `.chakra-view` (`panel.css:917-925`) is a column flex box with
`gap: var(--s-1)` = 2px and `padding: 0 0 var(--s-4)` = 12px at the foot. `.chakra__gloss`
(`panel.css:1019-1027`) is `height: 14px` **and** `margin: 0 0 var(--s-2)` = 4px.

```
346  region
-12  padding-bottom
-14  gloss height
- 4  gloss margin-bottom   ← not in the budget
- 2  flex gap
────
314  left for the chart
```

The `<svg>` is `width: 100%` (318) with `height: auto`, so its intrinsic height is 318; as a
flex item with the default `flex-shrink: 1` it is compressed to 314, and `preserveAspectRatio`
defaults to `xMidYMid meet`, so the drawing is letterboxed to 314 × 314 with a 2px bar each
side. **The chart therefore does not reach either window edge**, which is the one thing
`panel.css:913-916` and `panel.css:927-929` both say it does. The gloss's 4px margin duplicates
the 2px gap; one of the two should go.

### 3.5 The chart's outer frame is drawn at half weight

`Chakra.tsx:31` sets `SIZE = 318` and the outer compartment vertices are at `x = 0`, `x = s`,
`y = 0`, `y = s` (`Chakra.tsx:190-194`). `.chakra__cell` strokes at `stroke-width: 1`
(`panel.css:936-940`), centred on the path, so half of the outer stroke falls outside the
viewBox and is clipped by the SVG's default `overflow: hidden`.

The result: every internal division is a full 1px and the chart's own border is 0.5px, sitting
directly on top of the panel's own 1px `--border`. A frame lighter than the lines inside it is
most of why the drawing reads as escaping the window rather than sitting in it.

### 3.6 Four type styles, none of them in the type scale

DESIGN §3.3 lists eight roles. The chart uses none of them.

| Rule | Declaration | Nearest role in §3.3 |
|---|---|---|
| `.chakra__label` (`panel.css:964-968`) | `400 9px/1`, `letter-spacing: 0.2px` | none — 9px does not exist |
| `.chakra__graha` (`panel.css:972-975`) | `500 10px/1` | Micro-caps is 10px but 600 and uppercase |
| `.is-exalted` (`panel.css:980-982`) | `font-weight: 800` | none — the scale tops out at 600 |
| `.is-debilitated` (`panel.css:984-987`) | `font-weight: 400` + `--text-secondary` | — |

`panel.css`'s own header comment (lines 2-4) states the invariant these break:

> "No literal colour, radius, duration or font size appears below; they all come from
> tokens.css."

Two consequences beyond the bookkeeping.

- **9px is below the scale's floor and the app's scale clamp compounds it.** DESIGN §2.5:
  "Below 0.8 the 10px labels stop being legible." At the Compact setting the rashi labels
  render at 7.2px and the grahas at 8px.
- **Three weights on a 10px two-letter label is more precision than the medium carries.** 400
  against 500 is a 100-unit step at 10px; whether it is perceptible at all is **unverified**
  and should be measured before it is relied on to distinguish "debilitated" from "neither".
  Both references decline to draw dignity by default. Drik Panchang has no dignity mark on the
  face at all (§1.2); Jagannatha Hora can highlight it and ships it off, describing the feature
  as one that exists "to aid beginners" (§2.5).

### 3.7 `--chart-label` sets text below the contrast floor

`src/styles/tokens.css:67` — `--chart-label: #7f9bb5`, used as `fill` on the 9px rashi labels.

Relative luminance, computed:

| Colour | sRGB | Relative luminance |
|---|---|---|
| `--text-secondary` `#a1a1a8` | 161, 161, 168 | 0.3589 |
| `--chart-label` `#7f9bb5` | 127, 155, 181 | **0.3130** |

`--chart-label` is **darker than `--text-secondary`**, which DESIGN §4.1 measures at exactly
4.76:1 on the worst-case composited ground — the app's floor. Solving that figure for the
ground gives L ≈ 0.0359, and

```
(0.3130 + 0.05) / (0.0359 + 0.05) = 4.23
```

Against `G_lite` `#393939` as modelled in `day-view-states.md` §2.2 (L = 0.04089) it is
**3.99**. Either way it is below the 4.5:1 small-text floor, on the smallest type in the
application. This is the same test, on the same ground, that retired `--text-tertiary` under
D-023.

It is also a **third hue**, and two documents say there are two:

- `tokens.css:47` — "Meaning. Exactly two hues in the entire app."
- DESIGN §4 — "/* meaning — exactly two hues */"

`--chart-label` appears in neither, and DESIGN §11.3's "never conveyed by colour or shape
alone" table has no row for it.

### 3.8 An underline, which D-024 forbids by name

`panel.css:992-996`

```css
.chakra__graha.is-own-sign {
  text-decoration: underline;
  …
}
```

D-024 is titled "One mark vocabulary, the same in both calendars. **No underlines.**" Its body:
"**No underlines anywhere.** The rule under the numeral is gone and nothing replaced it in that
position." DESIGN §3.3: "No italics anywhere. **No underlines anywhere**."

The reasoning that removed it was that a rule under type reads as underlined text. Under a
two-letter label at 10px that reading is stronger, not weaker: the mark is as wide as the word.

### 3.9 Two tokens are given a second meaning

| Token | What it means in the app | What the chart uses it for |
|---|---|---|
| `--surface-selected` | the selected day (DESIGN §11.3) | the lagna's compartment fill, `panel.css:944-947` |
| dimming | a cell belonging to the neighbouring month; D-020 and `day-view-states.md` §7.3 R-9 record that "a dimmed panel reads as disabled" | debilitation, `panel.css:984-987` |

The lagna fill is also redundant in the default format. `Chakra.tsx:337` applies `is-lagna` in
all three, but `panel.css:949-951` argues that "The North Indian chart needs none: its lagna is
house 1 and house 1 is always the top compartment" — which is true of the fill as much as of
the diagonal. So North gets a fill it does not need, and South and East get a fill *and* a
diagonal for one fact. Neither reference marks the lagna with a fill: Drik Panchang marks it
with nothing at all (§1.2) and Jagannatha Hora writes `As` in the cell (§2.2).

### 3.10 The degraded-ephemeris sentence never reaches the chart

`Chakra` carries `source: Source` (`src/ipc/types.ts:388-394`, `crates/almanac/src/chakra.rs:34`),
built with `Source::weakest` over the ascendant and all nine positions
(`crates/almanac/src/chakra.rs:184-187`). The front end never reads it. `grep source
src/components/Panel.tsx` finds exactly one use, `Panel.tsx:767`, and it is the calendar's.

The calendar prints the sentence (`Panel.tsx:767-772`), the day view prints it, and the chart —
which composes ten degraded readings into one figure — does not. That is D-006 and DESIGN §9.4
unimplemented for this view, and the payload to implement it is already on the wire.

### 3.11 The sixty-second refetch blanks the region

`Panel.tsx:98-109`

```tsx
const [chartAt, setChartAt] = createSignal(Date.now());
…setInterval(() => setChartAt(Date.now()), 60_000)…
const [chart] = createResource(… chartAt() …, (at) => ipc.chakra(at));
```

and `Panel.tsx:796-808` reads `chart()` directly. When the source signal changes, a Solid
resource drops to `undefined` for the duration of the new fetch, so the `<Show when={chart()}>`
fails and the caption and the chart both unmount and remount.

This is the identical defect the same file already documents and solved once, for months:

> "A Solid resource drops to `undefined` while it refetches, so reading the grid straight from
> three resources emptied all three the instant a scroll committed: the strip went blank and
> the title cleared until the round trips came back." — `Panel.tsx:126-131`

It fires only if the panel keeps focus for a full minute, which is uncommon but not rare.

Separately, the timer exists at all against `kundali.md` §5.5, which decided the opposite —
"**The chart is not refreshed while open.** … A timer would be background work (D-017) for a
change that is almost always invisible." The implementation's counter-argument
(`Panel.tsx:91-97`) is reasonable and the timer does not run while the panel is hidden, so this
is a spec that has gone stale rather than a defect. It should be recorded as such.

### 3.12 Everything the chart draws is spoken as one string

`Chakra.tsx:315-322` gives the `<svg>` `role="img"` with an `aria-label` built by `spoken()`
(`Chakra.tsx:447-472`), which concatenates twelve houses into a single sentence — roughly 700
characters. `role="img"` makes the subtree presentational, so the twelve `<title>` elements
inside (`Chakra.tsx:354-356`, `Chakra.tsx:423`) are not reachable as separate objects.

`kundali.md` §7.1 specified the opposite and gave the reason:

> "So: `role="list"` with twelve `listitem`s, in rashi order … CSS Grid `grid-area` places each
> item in its fixed cell, decoupling DOM order from visual order."

Two consequences:

- A VoiceOver user cannot walk the chart house by house; they get one utterance or none.
- **Every degree in the chart is mouse-only.** `describe()` (`Chakra.tsx:481-493`) puts each
  graha's position to the arcminute in a hover title; `spoken()` includes no degrees at all
  except the lagna's. A keyboard user has no route to them.

**Whether WebKit exposes the `aria-label` intact here is unverified.** DESIGN §11.4 records
that WebKit once pruned the entire calendar subtree and that it was found only by looking at
the live accessibility tree. The same check is owed here.

### 3.13 Smaller things

| # | What | Where |
|---|---|---|
| a | `.chakra__lagna-line` is dead CSS — no markup references it | `panel.css:1005-1010` |
| b | The degree-and-arcminute format is implemented twice, inline, with no shared helper — once for the caption and once for the hover | `Panel.tsx:820-828`, `Chakra.tsx:490` |
| c | `degrees_in_rashi` carries arcseconds that neither call site uses | `src/ipc/types.ts:403` |
| d | The chart writes three-letter sign *names* in the North Indian format, where both references write a *number* (§1.2) and where the compartments are houses. `Chakra.tsx:341-346` records the choice and its reason; it is a real departure from both references and belongs in §7 as a question, not in this list as a defect | `Chakra.tsx:347-357` |

**Checked and conforming, so not listed as defects:** the chart has no loading state, which is
what DESIGN §9.2 requires ("nothing renders until the day arrives. No labels, no placeholder");
`formatTime`'s `02:51 PM` is `Intl` with `hour: "2-digit"` (`src/lib/format.ts:47-53`) and is
the same everywhere in the app, so it is not a chart problem; the retrograde bracket matches
Jagannatha Hora (§2.2); the combustion wash correctly reuses `--glare` and D-024's vocabulary.

---

## 4. Which reference decisions to take, and which to reject

### 4.1 Take

| Decision | From | Why it survives the menu bar |
|---|---|---|
| **The chart face carries position only.** Every qualifier — degree, dignity, nakshatra — is deferred to a table or a hover | both (§1.2, §1.5, §2.2, §2.4, §2.5) | This is the strongest agreement between the two references and it is what makes a small chart possible at all |
| **The instant and the place go beside the chart, always** | both (§1.4, §2.3) | Two short lines. It is what the current caption has room for once the gloss is dropped |
| **Retrograde as parentheses** | JH (§2.2) | Already shipped. Costs no width the compartment has to find |
| **The subordinate tier of text is smaller, quieter, and pinned to a corner; the grahas own the middle** | JH (§2.2), DP's optional rashi tags at `opacity: .5` (§1.2) | Exactly the relationship the rashi label and the graha cluster need, and the current 9px/10px pair is too close to establish it |
| **Type the chart generously relative to the drawing** | DP at 3.75% of chart width (§1.3), JH larger still | Chandra is at 3.14% for a shorter string. There is room to grow |
| **The lagna is identified from outside the chart face** — a select reading `− Lagna`, a table row, a tab | DP (§1.2, §1.6) | The header is Chandra's version of that slot, and it is currently spending it on a genre label |
| **A non-square chart is legitimate** | DP's 3:2 (§1.1) | Not needed if the panel stays 414, but it is the escape hatch if the height is ever refused |
| **The North Indian compartment carries a sign *number*; the South Indian one carries nothing** | both, and they agree exactly (§1.2, §2.2) | The current build writes three-letter names in all three formats. A numeral is 1–2 characters against 3, in the compartments that have the least width. See §9 Q5 — this is a decision, not a defect |
| **Qualifiers live in a table whose `Longitude` is sign-relative DMS** | JH (§2.4) | `degrees_in_rashi` is already exactly that shape. Chandra's table is the hover and the spoken list |

### 4.2 Reject

| Decision | From | Why the menu bar breaks it |
|---|---|---|
| **Colour encoding graha identity** — nine fills, one per planet | DP (§1.7) | D-023, directly. It would also be the single most authentic-looking change available, which is what makes it worth naming |
| **Colour encoding rashi lord** on the lagna table | DP (§1.6) | Same |
| **A chart never shown alone; two charts plus a calculation panel** | JH (§2.1) | 320 × 414 holds one chart and two lines |
| **Tables beside the chart** — nine columns of Graha Details, six of Bhava | DP (§1.5) | The narrowest of those tables needs more than 288px. Chandra's equivalent is the hover and the spoken form |
| **Divisional charts, arudhas, special lagnas, bhava chalit** | both | `kundali.md` §10 already refuses these and the reasoning holds |
| **A watermark inside the chart** | DP (§1.2) | 12px of ink in the centre void for no reader |
| **Curved compartment boundaries** | DP (§1.1) | At 318px a Bézier costs legibility in the corner triangles |
| **No mark for the ascendant at all** | DP (§1.2) | DP can rely on it because the numeral in house 1 names the rising sign and the tables repeat it four ways. Chandra prints names rather than numbers and has one line of caption |
| **Nine simultaneous windows, a menu bar, two tab strips, a status line** | JH (§2.1) | The popover has no chrome by construction (DESIGN §2.2) |

### 4.3 The thing neither reference can tell us

Both are worked in. Drik Panchang's kundali page is a destination with two charts, three tables
and a 29-item toolbar; Jagannatha Hora is a desktop application with a dasha tree and bar
graphs. Neither has a five-second form.

The nearest thing either produces is Drik Panchang's **Running Lagna card** (§1.6): a sign, the
span it is running for, a countdown, and the place — with no chart. That is a genuinely
menu-bar-shaped answer to the same question, and it is worth noting that their own answer to
"what does someone glancing want" omits the drawing entirely. Chandra's chart earns its place
because it also shows where the nine grahas stand; but the card is the evidence that **the
lagna belongs in the header, not at the end of a caption.**

---

## 5. Proposals

### 5.1 The header carries the reading *(recommended)*

| | |
|---|---|
| Title | `Dhanu Lagna` — the rising sign, the way the day view's title is the tithi |
| Leading slot | unchanged: `ChartMark`, the same mark the status item carries |
| Caption | `12°38′ · 02:51 PM · PATAN` — the degree, the instant, the place |
| Reason | Every other header in the panel names a value (§3.1); the app already speaks the form `<sign> lagna` (`kundali.md` §5.4's tooltip). The caption becomes a locator, which is what the day view's date line is |
| Cost | The word "kundali" leaves the surface. It survives in the settings section title (`SettingsView.tsx:52`) and the tray tooltip |
| Truncation | `Dhanu Lagna` → `Dhanu`. One rung; at 15/600 the full form is ~90px in a 224px slot, so it never fires |

The place is a new caption token and it can be long. `Location.label` is a city name; the
ladder drops it first, then the instant, leaving `12°38′` — the degree is the last thing
dropped because it is the reason the chart is a chart.

### 5.2 The chart itself

| Change | From | To | Reason |
|---|---|---|---|
| Height | 314 (§3.4) | 318 | Delete the `margin-bottom` on `.chakra__gloss` and keep the flex `gap`. Then the CSS and `panel.rs:31-34` agree |
| Outer frame | 0.5px (§3.5) | 1px | Inset the outer vertices by 0.5, or give `.chakra__cell` `vector-effect: non-scaling-stroke` and a `stroke-alignment` that is supported — the inset is the portable fix |
| Rashi label | `400 9px`, `--chart-label` | `400 10px`, `--text-secondary` | 9px is off the scale and below the contrast floor (§3.6, §3.7). At 10px it is the size of Micro-caps without being uppercase |
| Graha | `500 10px` | `500 11px` | Both references type the chart larger relative to the drawing (§1.3). `(Sa)` at 11px is ≈24 units against a 30-unit `GRAHA_COLUMN`; `GRAHA_ROW` goes 12 → 13. **Whether five bodies still fit a corner triangle at 13 must be measured against real layout before the number is fixed** — the same discipline `kundali.md` §4.3 applies to the retrograde mark |
| Exalted | `font-weight: 800` | `font-weight: 600` | 600 is the scale's top (DESIGN §3.3) and 500 → 600 is the step the app already uses to mean emphasis |
| Debilitated | `400` + `--text-secondary` | `--text-secondary`, weight unchanged at 500 | One carrier, not two. Weight 400 vs 500 at 10px is below the medium's resolution (§3.6) |
| Own sign | `text-decoration: underline` | **removed from the face** | D-024 (§3.8). It is the smallest of the four dignity facts by the CSS's own argument (`panel.css:989-991`). Drik Panchang draws no dignity on the face; Jagannatha Hora can, ships it off, and calls it an aid "to aid beginners" (§2.5). It moves to the hover and the spoken form, where it already is |
| Lagna cell fill | `--surface-selected` at 0.55 | **removed** | §3.9. North identifies house 1 by position; South and East keep the diagonal, which is the published convention |
| `--chart-label` | `#7f9bb5` | **deleted**, or raised to `#a4b8cb` | §3.7 and §7 Q3 |

If the hue is kept rather than deleted, `#a4b8cb` is HSL(209, 27%, 72%) — the same hue and
saturation, lightness raised until it clears. Relative luminance 0.4662, so **5.68:1** on
`G_lite` and **6.01:1** on the ground DESIGN §4.1's 4.76 figure implies. It is lighter than
`--text-secondary`, so DESIGN §4.1's own shortcut applies: it clears whatever that clears.

### 5.3 The degraded case

When `chart().source === "moshier"`, the caption line is **replaced** by the sentence the
calendar already prints, at `--type-caption` over two lines, and the chart shrinks to 304
square:

```
28  provenance, 2 × 14
 2  gap
304 chart
12  padding
────
346 region — unchanged
```

The panel's height does not move. The normal case pays nothing. A chart composed from ten
degraded readings is exactly the case where D-006's sentence is owed, and §3.10 shows the
payload is already there.

### 5.4 The mockup, at true proportions

`5px = 1 character`, `12px = 1 line`. The panel is 320 × 414, so the frame is 64 characters
wide and 34½ lines tall. The character cell is 5 × 12, not square, so the chart's 45° diagonals
advance about 2.4 characters per line — they are drawn at the cell's aspect, not the chart's.

Lagna Dhanu 12°38′. Su Me Ve in Leo, Mo in Tau, Ma in Can, Ju in Gem, Sa retrograde in Pis,
Ra in Ari, Ke in Lib.

```
 x:0     16                                              304  320
   ┌──────────────────────────────────────────────────────────────┐ y:0
   │                                                              │  12  pad
   │  ◇   Dhanu Lagna                                       [ ⚙ ] │  40  header
   │                                                              │   4
   │  12°38′ · 02:51 PM · PATAN                                   │  14  micro-caps
   │                                                              │   2
   ├──────────────────────────────────────────────────────────────┤ ← chart, y:72
   │Cap             ╲                    ╱             Sco        │
   │                  ╲       Sag      ╱                          │
   │                    ╲            ╱                            │
   │Aqu                   ╲        ╱                       Lib    │
   │                        ╲    ╱                                │
   │        ╲                 ╲╱                 ╱          Ke    │
   │          ╲               ╱╲               ╱                  │
   │            ╲           ╱    ╲           ╱                    │
   │  Pis         ╲       ╱        ╲       ╱        Vir           │
   │              ╱ ╲   ╱            ╲   ╱ ╲                      │
   │   (Sa)     ╱     ╳                ╳     ╲                    │
   │          ╱     ╱   ╲            ╱   ╲     ╲                  │
   │        ╱     ╱       ╲        ╱       ╲     ╲       Su Me    │
   │            ╱           ╲    ╱           ╲              Ve    │
   │          ╱               ╲╱               ╲                  │
   │  Ari   ╱                 ╱╲                 ╲     Leo        │
   │      ╱                 ╱    ╲                 ╲              │
   │                      ╱        ╲                       Can    │
   │Tau                 ╱   Gem      ╲                       Ma   │
   │                  ╱                ╲                          │
   │  Mo            ╱        Ju          ╲                        │
   │              ╱                        ╲                      │
   ├──────────────────────────────────────────────────────────────┤ ← y:390
   │                                                              │  12  pad
   └──────────────────────────────────────────────────────────────┘ y:414
    ↑                                                            ↑
    the chart reaches both inner edges; nothing else in the panel does
```

Positions, from the code's own construction rather than from the drawing above:

| Element | Rule | `Chakra.tsx` |
|---|---|---|
| Compartment vertices | the square, both diagonals, the midpoint diamond | `169-237` |
| Rashi label | two fifths of the way from the compartment's outermost vertex to its centroid | `218-231` |
| Graha cluster | centred on the centroid, flowed into rows, columns limited by the polygon's width *at that row's height* | `119-154` |
| Lagna | house 1 is the top kite, by construction | `210-211` |

Header slots, unchanged from DESIGN §5.2:

```
 x:16      40    48                                    272   280   304
  ┌──────────┬───┬────────────────────────────────────────┬───┬───────┐
  │    ◇     │ 8 │  "Dhanu Lagna"                         │ 8 │   ⚙   │
  │  24 × 16 │   │  Title 15/600/−0.10, primary           │   │ 24×24 │
  └──────────┴───┴────────────────────────────────────────┴───┴───────┘
```

### 5.5 The panel's height is a property of the status item, not of a view

Recommended alongside §5.1–§5.4. Three changes, all in `panel.rs`:

1. `toggle` already knows the subject (`panel.rs:174-219`). Set the window size from
   `Subject::Chart` there, **before** `place()` and `show()`. That removes the clipped first
   frame described in §3.3, because the window is never shown at the wrong height.
2. `PanelTall` becomes a function of the current subject rather than of the front end's view,
   so `set_tall` and the IPC command it serves can go, and `Panel.tsx:661-671` with them.
3. Settings opened from the chart keeps the chart's height. Settings is a scrolling list
   (`SettingsView.tsx`), so 346px of region shows more of it and nothing is displaced. **The
   panel then never resizes during a session at all.**

DESIGN §2.2 needs a matching edit: the panel has two heights, 332 and 414, each fixed for the
life of a session and chosen by which status item was clicked. That is a weaker invariant than
"one height", and it is still an invariant.

### 5.6 The spoken form, and the degrees

`Chakra.tsx:315-322`'s single `aria-label` is replaced by what `kundali.md` §7.1 specified: a
visually-hidden ordered list of twelve items, in house order, each naming the sign, the house,
and every body in it with its states **and its degree**. The `<svg>` keeps `role="img"` with a
short label (`North Indian chart, Dhanu lagna`), and `aria-hidden` is not used, because the list
is the accessible name and the picture is the picture.

Zero pixels. Twelve DOM nodes. It also closes §3.12's second half: the arcminute positions stop
being reachable only by a pointer.

### 5.7 The two smaller fixes

- `chart.latest` in place of `chart()` at `Panel.tsx:798`, so the sixty-second refetch does not
  blank the region (§3.11). The month cache is the same idea at more scale; for a single value
  `latest` is the whole of it.
- Delete `.chakra__lagna-line` (§3.13a) and move the degree-and-arcminute format into
  `src/lib/format.ts` so `Panel.tsx:820-828` and `Chakra.tsx:490` share one implementation.

### 5.8 What was considered and is being rejected

| # | Alternative | Why not |
|---|---|---|
| R-1 | Fit the chart into the 264px region and keep one panel height | A 264 square renders the graha labels at 0.83 × 10px = 8.3px at the Default scale and 6.6px at Compact. DESIGN §2.5 sets the floor at "below 0.8 the 10px labels stop being legible". This is the argument that decides §6 |
| R-2 | Keep 264, hold the type at absolute size, and let the compartments shrink | Arithmetically it survives — the smallest compartment still admits two columns of four bodies. But five bodies in a corner triangle reduces the column pitch to about 26 units against a 24-unit `(Sa)`, and the app's own worst case (Su + Me + Ve + Mo, plus a fifth) is ordinary. It trades a measured legibility floor for an unmeasured collision |
| R-3 | Make the chart 3:2 like Drik Panchang's, 318 × 212, and fit the region | Genuinely available, and the geometry generalises. Rejected because `GRAHA_ROW` is the binding constraint: at 212 tall the row pitch falls to 8 units against 10px type and consecutive lines touch. A rectangle helps the frame, not the cluster |
| R-4 | Put the lagna's degree in the header beside the sign — `Dhanu 12°38′` | Tempting, and `tabular-nums` on `:root` (DESIGN §3.2) means it would not jitter. Rejected because the header would then carry a number that changes every four minutes while the caption below it carries none, which inverts the day view's relationship between the two lines |
| R-5 | A centre block in the chart, as `kundali.md` §4.4 specified | That design was for a South Indian grid with a 144 × 120 void. The North Indian chart has no void; house 1's kite is at the centre and holds bodies |
| R-6 | Draw dignity as a mark rather than as weight — a dot, a ring, a corner tick | A mark beside a two-letter label at 10px is 20px of ink against 14px of word. Both references decline to draw dignity by default, and JH frames its optional highlight as beginner scaffolding rather than as a practitioner's need (§1.2, §2.5) |
| R-7 | `--accent` for the lagna | D-023. Gold means today, and a chart of now is entirely today — `panel.css:942-943` already makes this argument and is right |
| R-8 | A back chevron in the chart's leading slot | `Header.tsx:104-141` already refuses it, correctly: the chart has its own status item and is a peer of the calendar, and a chevron would leave the back end still believing the chart was showing |
| R-9 | Nine graha colours, as Drik Panchang uses | §4.2. It would look the most authentic and it is the clearest possible violation of D-023 |

---

## 6. The 414px question, answered

**Keep the taller panel. Change how it is chosen.**

The height itself is right, and the argument is a measurement rather than a preference:

- The chart is square and drawn to the window's inner width, 318.
- Fitting it into the 264px region scales the whole viewBox by 0.83, because the type is inside
  the viewBox (`Chakra.tsx:317-319`, `panel.css:930-934`).
- 10px graha labels become 8.3px, and 9px rashi labels become 7.5px, at the **Default** scale.
- At the Compact scale (0.8, DESIGN §2.5) those become 6.6px and 6.0px.
- DESIGN §2.5 sets the floor explicitly: "Below 0.8 the 10px labels stop being legible."

So the 264 chart is below the app's own stated legibility floor at one of the four scales it
ships, and at the floor at another. `panel.rs:36-43` reached the same conclusion by a different
route ("a square that had to fit the region would be 234") and reached it correctly.

**Jagannatha Hora is the worked example of choosing the other way.** Its packed six-chart mode
keeps the type size and shrinks the chart, and the result is verifiable in its own screenshots:
compartments overlap and clip — `Mo Su (Me` running past a cell boundary, `(Sa) Ra` overprinting
(§2.7). Its answer is not smaller type but a **display-width gate** — packed mode is offered
only above 1280px and carries a `Turn OFF packed chart mode` item. A 320pt popover has no wider
display to fall back to, so the equivalent move is the one already made: give the chart the
height it needs.

**But the mechanism is wrong on three counts**, and each is fixable without touching the
height (§5.5):

1. The height is decided by the front end after the window is on screen, so the first frame of
   every chart open is drawn at the wrong size (§3.3).
2. It is expressed as a *transition* — `set_tall(true)`, `set_tall(false)` — which is what made
   opening settings from the chart resize the window under the pointer, and which already cost
   one focus bug (`panel.rs:433-437`).
3. Three comments and one specification section still say the height is constant (§3.3), so the
   next person to read them will be misled.

Recast as "this status item's panel is 320 × 414, fixed for the session", all three go away and
DESIGN §2.2's actual guarantee — *the popover does not resize while you are using it* — is
restored in full. That guarantee is the one that matters in a menu bar. "There is exactly one
size" was only ever the means to it.

---

## 7. Five seconds: what shows, what hovers, what is not drawn

The chart has its own status item, so the reader has already said what they want. In the
seconds the popover is up they are looking for one of two things: **what is rising**, and
**where the Moon and the personal grahas stand**.

### On the surface, always

| Fact | Where | Why it cannot wait for a hover |
|---|---|---|
| The rising sign | header title, `Dhanu Lagna` | It is the reason the chart is a chart, and it turns over every ~2 hours |
| How far into it | caption, `12°38′` | It moves 1° per four minutes; it is what says whether the lagna is about to change |
| The instant | caption | Makes the drawing checkable against the clock |
| The place | caption | The lagna's correctness depends on it (§3.2), and both references print it (§1.4, §2.3) |
| Which sign each compartment is | the three-letter label | A first-time reader has not learnt the fixed layout; DESIGN §11.3 forbids position alone carrying a meaning |
| Which sign each graha is in | the two-letter label in its compartment | The whole content of the chart |
| Retrograde | brackets, `(Sa)` | Binary, changes which way a reading goes, and costs no width (§4.1) |
| Combustion | the warm wash | Already the app's shared vocabulary (D-024), already behind the name it belongs to |
| Exalted / debilitated | weight, dimness | The two dignity facts a practitioner scans for. Two states, not four (§5.2) |
| The degraded sentence, when it applies | in place of the caption | D-006, DESIGN §9.4 (§5.3) |

### On hover, and in the spoken list

The full graha name. The position to the arcminute. The house number. **Own sign**, moved off
the face by §5.2. Every dignity in words. This is `describe()` (`Chakra.tsx:481-493`) as it
already stands, plus the house — and §5.6 makes all of it reachable without a pointer.

Both references put the degree exactly here, and neither puts it on the chart face: Drik
Panchang in a nine-column table with `title` tooltips (§1.5), Jagannatha Hora in a six-column
table whose `Longitude` is sign-relative DMS, the same shape as Chandra's `degrees_in_rashi`
(§2.4).

### Not drawn at all

House numbers (`kundali.md` §3.4 settled this; the ordinal is spoken). Nakshatra and pada per
graha — a second chart's worth of text. Aspects and drishti lines. Any varga. Arudhas and
special lagnas, which are the bulk of what fills a Jagannatha Hora cell (§2.2) and which
Chandra has never computed. Any interpretation. A watermark. A legend — neither reference has
one (§1.4) and every mark the chart makes is named in the hover and the spoken list.

---

## 8. Verification

| # | Check | Method |
|---|---|---|
| V-1 | The chart is 318 × 318 and touches both inner edges | Screenshot the panel; the drawn polygon's bounding box must be 318 wide and 318 tall, with no letterbox (§3.4) |
| V-2 | The outer frame is the same weight as the internal divisions | Pixel-sample across the chart's left edge and across a diagonal; the two strokes must have equal alpha (§3.5) |
| V-3 | Every rule in `panel.css`'s chart block references a token | Grep the block for a numeric `font-size`, `font-weight` outside the scale, or a literal colour. Zero hits (§3.6, D-014) |
| V-4 | `--chart-label`, if kept, clears 4.5:1 | Render over a white desktop, screenshot, compute against the sampled ground — the same probe `day-view-states.md` §9 Q3 asks for (§3.7) |
| V-5 | No underline anywhere in the chart | Assert the computed `text-decoration-line` of every `.chakra__graha` is `none` for all four dignity cases (§3.8) |
| V-6 | The window is never shown at a height the page is not drawing | Record the window size at `show()` and the `.panel-frame` height in the first painted frame; they must be equal for both status items (§3.3, §5.5) |
| V-7 | The panel does not resize during a session | Open the chart, open settings, go back. Assert `inner_size` is unchanged throughout (§5.5) |
| V-8 | The sixty-second refetch does not unmount the chart | Hold the panel focused for 70 s with a `MutationObserver` on `.chakra-view`; the `<svg>` must not be removed (§3.11) |
| V-9 | The degraded sentence appears | Force `Source::Moshier`; assert the provenance text renders and the chart is 304 square (§3.10, §5.3) |
| V-10 | The spoken list exists and is navigable | Read the live accessibility tree, as DESIGN §11.4 requires — twelve items, in house order, each with its degrees (§3.12, §5.6) |
| V-11 | Five bodies in a corner triangle do not collide at 11px | Render a synthetic chart with Su, Me, Ve, Mo and Ma in one sign, for each of the four corner triangles, at scales 0.8 and 1.4 (§5.2) |

---

## 9. Open questions

1. **Should the header say `Dhanu Lagna` and lose the word "kundali" from the surface?** §5.1
   recommends it, because every other header in the panel names a value and this one names a
   genre. The feature keeps its name in the settings section and the tray tooltip. Is that
   enough, or should the header read `Lagna Kundali · Dhanu` and accept a longer title?

2. **Should the caption carry the place, at the cost of the instant on a narrow city name?**
   §5.1 puts `12°38′ · 02:51 PM · PATAN` in one 288px line and drops tokens right-to-left when
   it does not fit. Both references print the place (§1.4, §2.3) and `kundali.md` §4.4 argued
   it is not decoration. Do you want the place, and do you want it to win over the instant?

3. **`--chart-label`: delete it, or raise it to `#a4b8cb`?** It sets 9px text at about 4.0:1
   (§3.7) — the measurement that retired `--text-tertiary`. Deleting it restores "exactly two
   hues" and removes an encoding; raising it keeps the separation the CSS comment argues for
   at `panel.css:957-963`. If it is kept, DESIGN §4 and `tokens.css:47` both need amending to
   say three.

4. **Should own sign leave the chart face?** §5.2 recommends it: D-024 forbids the underline
   that draws it, and both references ship dignity off the face by default — JH's optional
   highlight is described as an aid "to aid beginners" (§2.5). The alternative is to
   find a non-underline mark for it, which R-6 argues does not exist at 10px. Losing it means a
   practitioner scanning for a graha at home has to hover.

5. **Should the North Indian compartments print the rashi *number* rather than its name?**
   `Chakra.tsx:341-346` chose names in all three formats and gives its reason ("a number is a
   lookup"). **Both references write a number in the North Indian chart and nothing at all in
   the South Indian one.** Drik Panchang prints the numeral by default and offers names as an
   opt-in at half opacity (§1.2); Jagannatha Hora prints small green digits 1–12 at each
   house's inner vertex in North Indian style and leaves South Indian cells unlabelled (§2.2).
   The agreement is exact and it is against the current implementation. A number is also 1–2
   characters against 3, which is width the corner triangles need. Names, numbers, or names as
   a setting?

6. **Is `Lagna Kundali` still the right name for the feature?** `kundali.md` §5.3 argued for
   `Gochara` and `docs/TODO.md` §E1 records the decision against it. Nothing in this document
   reopens that; it is listed only because the header change in §5.1 removes the name from the
   one surface it appears on, which is a reasonable moment to confirm it.

7. **Should the sixty-second refresh stay?** `kundali.md` §5.5 decided against a timer and the
   implementation added one (§3.11). The timer only runs while the chart is the view and the
   panel is open, so D-017 is not engaged. Keeping it means fixing the blank (§5.7); dropping
   it means the printed instant is what makes the chart checkable, which is what §5.5 argued.
   Which document is now right?

8. **DESIGN §2.2 has to change either way.** §5.5 proposes "two fixed heights, chosen by the
   status item". The alternative is to state the current behaviour honestly — "one view resizes
   the window" — and keep the front end owning it. Do you want the invariant restated, or the
   exception documented?
