-- Insight points, not a dashboard.
--
-- Each block below answers one question that could change a decision. A chart
-- of downloads over time answers none of them, which is why there isn't one.
--
-- Run:  make insights        (all of them)
--       npx wrangler d1 execute chandra-analytics --remote --command "<one>"
--
-- `verdict = 'human'` throughout. Bot rows are kept so the classifier can be
-- audited - the last block is that audit - but they are never a download.

-- ---------------------------------------------------------------------------
-- 1. How many people actually have it.
--
-- Distinct visitors, not requests: a retry, a resumed download and a second
-- machine all inflate the raw count. This is the number to quote.
-- ---------------------------------------------------------------------------
SELECT
  'real downloads'            AS insight,
  COUNT(DISTINCT visitor)     AS people,
  COUNT(*)                    AS requests,
  MIN(day)                    AS since
FROM download
WHERE verdict = 'human';

-- ---------------------------------------------------------------------------
-- 2. What the page is worth.
--
-- Of the people who looked, how many took it. A low number means the page is
-- not making the case; a high number with few visitors means the problem is
-- reach, not the page. Those need opposite responses, which is why the two
-- halves are here together rather than in separate reports.
-- ---------------------------------------------------------------------------
SELECT
  'conversion'                                        AS insight,
  (SELECT COUNT(DISTINCT visitor) FROM visit
    WHERE verdict = 'human')                          AS looked,
  (SELECT COUNT(DISTINCT visitor) FROM download
    WHERE verdict = 'human')                          AS took,
  ROUND(
    100.0 * (SELECT COUNT(DISTINCT visitor) FROM download WHERE verdict = 'human')
    / NULLIF((SELECT COUNT(DISTINCT visitor) FROM visit WHERE verdict = 'human'), 0),
    1)                                                AS percent;

-- ---------------------------------------------------------------------------
-- 3. Whether the minimum macOS version is costing anything.
--
-- `LSMinimumSystemVersion` is 11.0, and that was asserted rather than measured.
-- If nobody on 11 or 12 ever downloads, raising it is free and removes a claim
-- nothing tests. If they do, it is load-bearing and must stay supported.
-- ---------------------------------------------------------------------------
SELECT
  'macos version'                          AS insight,
  COALESCE(os_version, 'unknown')          AS os,
  COUNT(DISTINCT visitor)                  AS people
FROM download
WHERE verdict = 'human'
GROUP BY os
ORDER BY CAST(os AS REAL) DESC;

-- ---------------------------------------------------------------------------
-- 4. Whether the universal build earns its build time.
--
-- Read this one carefully. Safari and Chrome both still report `Intel Mac OS X`
-- on Apple Silicon, so `unknown` is genuinely unknown and is *not* Intel - the
-- honest reading is "at least this many are Apple Silicon". If apple-silicon is
-- already most of what can be identified, an Intel slice is not worth keeping;
-- it can never prove the opposite.
-- ---------------------------------------------------------------------------
SELECT
  'architecture'          AS insight,
  arch,
  COUNT(DISTINCT visitor) AS people
FROM download
WHERE verdict = 'human'
GROUP BY arch
ORDER BY people DESC;

-- ---------------------------------------------------------------------------
-- 5. Where people come from, so effort goes where it works.
--
-- Host only. A full referrer would carry search terms and private forum paths,
-- which is not a trade worth making for a marketing number.
-- ---------------------------------------------------------------------------
SELECT
  'referrer'                                 AS insight,
  COALESCE(referrer_host, 'direct or hidden') AS source,
  COUNT(DISTINCT visitor)                    AS people
FROM download
WHERE verdict = 'human'
GROUP BY source
ORDER BY people DESC
LIMIT 15;

-- ---------------------------------------------------------------------------
-- 6. Whether a release landed.
--
-- Downloads in the seven days after each version first appeared. A release
-- nobody took is worth knowing about before the next one is planned.
-- ---------------------------------------------------------------------------
SELECT
  'release uptake'        AS insight,
  version,
  MIN(day)                AS released,
  COUNT(DISTINCT visitor) AS people
FROM download
WHERE verdict = 'human' AND version IS NOT NULL
GROUP BY version
ORDER BY released DESC;

-- ---------------------------------------------------------------------------
-- 7. Reach, by country.
--
-- Two letters, from Cloudflare's own edge. Useful for one decision only: what
-- to translate, and whether a second calendar convention is worth supporting.
-- ---------------------------------------------------------------------------
SELECT
  'countries'                     AS insight,
  COALESCE(country, '??')         AS country,
  COUNT(DISTINCT visitor)         AS people
FROM download
WHERE verdict = 'human'
GROUP BY country
ORDER BY people DESC
LIMIT 15;

-- ---------------------------------------------------------------------------
-- 8. The audit of the classifier itself.
--
-- The point of keeping bot rows. If one reason dominates, it is either a real
-- pattern or a rule that is too broad - and this is the only way to tell. A
-- count with no way to check it is a guess with a database behind it.
--
-- `not a mac` deserves particular attention: it should be large and dull. If
-- it ever carries browser fetch headers and a plausible macOS version, the
-- rule is wrong and the rows are still there to recount.
-- ---------------------------------------------------------------------------
SELECT
  'classifier'                                     AS insight,
  verdict,
  reason,
  COUNT(*)                                         AS requests,
  ROUND(100.0 * COUNT(*) / (SELECT COUNT(*) FROM download), 1) AS percent
FROM download
GROUP BY verdict, reason
ORDER BY requests DESC
LIMIT 25;
