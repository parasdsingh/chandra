-- Download events, one row per request that asked for a file.
--
-- Bots are recorded, not discarded. A count that silently drops rows cannot be
-- audited: if the classifier is wrong, nothing shows it. Every row carries the
-- verdict and the reason for it, so a rule can be re-examined against the
-- traffic it actually judged - and a wrong rule can be corrected in SQL after
-- the fact rather than needing the data to have been kept differently.
CREATE TABLE IF NOT EXISTS download (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  at            TEXT    NOT NULL,          -- ISO 8601, UTC
  day           TEXT    NOT NULL,          -- YYYY-MM-DD, for cheap grouping
  asset         TEXT    NOT NULL,          -- the object key requested
  version       TEXT,                      -- parsed out of the key
  -- The verdict. `human` is the only thing that counts as a download.
  verdict       TEXT    NOT NULL,          -- human | bot | probe
  reason        TEXT    NOT NULL,          -- why, so the rule can be audited
  -- What the request said about itself.
  os_version    TEXT,                      -- 10.15, 11, 14, ... from the UA
  arch          TEXT,                      -- apple-silicon | intel | unknown
  browser       TEXT,                      -- safari | chrome | firefox | other
  country       TEXT,                      -- two letters, from Cloudflare
  referrer_host TEXT,                      -- host only, never the full URL
  -- A salted hash, truncated. Enough to tell a repeat from a new visitor
  -- within a release, not enough to identify anybody. No IP is stored.
  visitor       TEXT
);

CREATE INDEX IF NOT EXISTS download_day     ON download (day);
CREATE INDEX IF NOT EXISTS download_verdict ON download (verdict, day);
CREATE INDEX IF NOT EXISTS download_visitor ON download (visitor);

-- Page views, so the one number that matters - how many people who looked went
-- on to download - can be computed at all. Same shape, same rules.
CREATE TABLE IF NOT EXISTS visit (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  at            TEXT    NOT NULL,
  day           TEXT    NOT NULL,
  path          TEXT    NOT NULL,
  verdict       TEXT    NOT NULL,
  reason        TEXT    NOT NULL,
  country       TEXT,
  referrer_host TEXT,
  visitor       TEXT
);

CREATE INDEX IF NOT EXISTS visit_day ON visit (day);

-- Feedback, as sent from the form the app links to.
--
-- Stored first, emailed second, and the email is a notification rather than the
-- record. A sending credential that expires, a provider that is down or a
-- monthly quota that runs out must not lose somebody's message - they took the
-- trouble to write it, and they have no way of knowing it vanished.
CREATE TABLE IF NOT EXISTS feedback (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  at        TEXT    NOT NULL,
  message   TEXT    NOT NULL,
  -- Optional. Blank means they did not want a reply, which is a valid thing to
  -- want, and the form says so rather than demanding an address.
  contact   TEXT,
  version   TEXT,                 -- the app version, prefilled by the link
  os        TEXT,                 -- what the browser says, for reproducing
  country   TEXT,
  -- Whether the notification went out. `pending` is the honest state when no
  -- sender is configured; nothing is lost, it is simply unread until someone
  -- looks. `make feedback` is that someone.
  delivered TEXT    NOT NULL DEFAULT 'pending',
  error     TEXT
);

CREATE INDEX IF NOT EXISTS feedback_at        ON feedback (at);
CREATE INDEX IF NOT EXISTS feedback_delivered ON feedback (delivered);
