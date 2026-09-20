/**
 * The download endpoint, and the only place Chandra is counted.
 *
 * The app itself sends nothing - it has no network code at all, and both the
 * About pane and the landing page say so. Everything known about usage is
 * therefore known from here: a request for the disk image, and a request for
 * the page that offers it.
 *
 * What this is careful about:
 *
 * - **No IP address is stored.** A salted truncated hash stands in, which
 *   distinguishes a repeat visitor from a new one within a release and
 *   identifies nobody. The salt is a secret and rotating it makes old hashes
 *   uncorrelatable with new ones, on purpose.
 * - **No full referrer.** The host only. A full URL carries search queries and
 *   private forum paths.
 * - **Bots are recorded, not dropped.** See `classify.ts`.
 * - **Logging never fails a download.** The write is fire-and-forget behind
 *   `waitUntil`; a database that is down must not stop somebody installing the
 *   app.
 */

import { classify } from "./classify";
import { notify, parse } from "./feedback";

export interface Env {
  DOWNLOADS: R2Bucket;
  ANALYTICS: D1Database;
  /** Salt for the visitor hash. `wrangler secret put VISITOR_SALT`. */
  VISITOR_SALT: string;
  /** Outbound mail. Absent means feedback is stored and not emailed, which is
   *  a working state rather than a broken one - `make feedback` reads it. */
  MAIL_API_KEY?: string;
  MAIL_FROM?: string;
  MAIL_TO?: string;
}

/** What a request asks for when it wants whatever the current release is.
 *  Resolved against the bucket and served directly; there is no redirect. */
const LATEST = "latest";

/** The only prefix this worker may read from.
 *
 * The bucket has since been split so Chandra has its own, but this stays: it
 * is what was missing when the key came straight from the URL path, and
 * `/download/<anything>` served *any* object in a bucket shared with another
 * project - twenty-three of them - through a public endpoint. Broken access
 * control, in eleven characters of missing check. A bucket that is not shared
 * today is not a reason to depend on it never being shared again. */
const PREFIX = "chandra/";

/** The only shape a release artefact may have.
 *
 * An allowlist, not a sanitiser. Stripping `..` and slashes is a game of
 * thinking of every encoding first; naming the one thing that is allowed is
 * not. Anything that is not exactly a Chandra disk image at a three part
 * version is refused before it reaches the bucket. */
const ARTEFACT = /^Chandra-\d+\.\d+\.\d+-universal\.dmg$/;

/** The object a request is allowed to reach, or nothing. */
function keyFor(asked: string): string | null {
  if (asked === LATEST) return null; // resolved separately
  // `decodeURIComponent` first, so a percent-encoded traversal is judged as
  // what it decodes to rather than as its encoding.
  let name: string;
  try {
    name = decodeURIComponent(asked);
  } catch {
    return null;
  }
  if (!ARTEFACT.test(name)) return null;
  return PREFIX + name;
}

/** A visitor hash: salted, truncated, and never reversible to an address. */
async function visitorHash(
  request: Request,
  salt: string,
): Promise<string | null> {
  const ip = request.headers.get("cf-connecting-ip");
  if (!ip || !salt) return null;
  const agent = request.headers.get("user-agent") ?? "";
  const data = new TextEncoder().encode(`${salt}:${ip}:${agent}`);
  const digest = await crypto.subtle.digest("SHA-256", data);
  // Sixteen hex characters. Enough that a collision is not a practical concern
  // at this volume, short enough to be useless as an identifier elsewhere.
  return [...new Uint8Array(digest)]
    .slice(0, 8)
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

/** The host the page says sent the visitor, if it is one and is not us.
 *
 * Host only, and validated here rather than trusted: this arrives in a query
 * string anyone can write. A visitor moving between pages of this site is not
 * an inbound referral and would otherwise swamp the table with its own host.
 */
function inboundHost(asked: string | null): string | null {
  if (!asked) return null;
  const host = asked.slice(0, 128).toLowerCase();
  if (!/^[a-z0-9.-]+$/.test(host)) return null;
  if (ALLOWED.some((origin) => new URL(origin).host === host)) return null;
  return host;
}

function referrerHost(request: Request): string | null {
  const referrer = request.headers.get("referer");
  if (!referrer) return null;
  try {
    return new URL(referrer).host || null;
  } catch {
    return null;
  }
}

/** The version in `Chandra-0.1.0-universal.dmg`, if it is there. */
function versionOf(key: string): string | null {
  return /(\d+\.\d+\.\d+)/.exec(key)?.[1] ?? null;
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext) {
    const url = new URL(request.url);

    if (url.pathname === "/health") {
      return new Response("ok", { headers: { "cache-control": "no-store" } });
    }

    if (url.pathname.startsWith("/download")) {
      return download(request, env, ctx, url);
    }

    if (url.pathname === "/feedback") {
      return feedback(request, env, ctx);
    }

    if (url.pathname === "/beacon") {
      return beacon(request, env, ctx, url);
    }

    return new Response("Not found", { status: 404 });
  },
} satisfies ExportedHandler<Env>;

/**
 * Serves the disk image and records the request.
 *
 * `/download` alone means the current release, which is what the landing page
 * links to: the page then never has to be redeployed to point at a new version,
 * and a link shared anywhere keeps working.
 */
async function download(
  request: Request,
  env: Env,
  ctx: ExecutionContext,
  url: URL,
): Promise<Response> {
  const asked = url.pathname.replace(/^\/download\/?/, "") || LATEST;
  const key = asked === LATEST ? await currentRelease(env) : keyFor(asked);

  if (!key) {
    // The same answer for "no release yet" and "that is not a thing you may
    // ask for". A distinct message would turn this endpoint into a way to
    // test which keys exist in the bucket.
    return new Response("No release yet.", {
      status: 404,
      headers: { "cache-control": "no-store" },
    });
  }

  // Belt and braces. `keyFor` and `currentRelease` both produce prefixed keys;
  // this is here so that a future third source of keys cannot skip the check
  // by simply not knowing about it.
  if (!key.startsWith(PREFIX)) {
    return new Response("Not found", { status: 404 });
  }

  const object = await env.DOWNLOADS.get(key);
  if (!object) {
    return new Response("Not found", { status: 404 });
  }

  const shape = classify(request);
  ctx.waitUntil(
    recordDownload(env, {
      asset: key,
      version: versionOf(key),
      classification: shape,
      request,
    }),
  );

  const headers = new Headers();
  object.writeHttpMetadata(headers);
  headers.set("etag", object.httpEtag);
  headers.set("content-type", "application/x-apple-diskimage");
  headers.set(
    "content-disposition",
    `attachment; filename="${key.split("/").pop()}"`,
  );
  // A release artefact never changes under its own name, so it may be cached
  // hard. `/download` is a pointer and must not be. Held for a year, a new
  // release would be invisible to every browser and every CDN edge that had
  // followed the link before - which is everyone who took the previous version.
  headers.set(
    "cache-control",
    asked === LATEST
      ? "public, max-age=300, must-revalidate"
      : "public, max-age=31536000, immutable",
  );
  return new Response(object.body, { headers });
}

/**
 * Which object is the current release.
 *
 * Read from the bucket rather than configured, so publishing a release is one
 * upload and nothing else: the newest `.dmg` by key wins, and keys carry a
 * version. A pointer file would be a second thing to remember and therefore a
 * second thing to forget.
 */
async function currentRelease(env: Env): Promise<string | null> {
  const listed = await env.DOWNLOADS.list({ prefix: PREFIX });
  // `ARTEFACT`, not `.endsWith(".dmg")`. This is the second source of keys and
  // it must pass the same test as the first: anything else in the prefix - a
  // hand-uploaded file, a build from another branch - would otherwise become
  // what `/download` serves, and `versionOf` would return null for it, so it
  // would vanish from the release-uptake query as well.
  const images = listed.objects
    .map((object) => object.key)
    .filter((key) => ARTEFACT.test(key.slice(PREFIX.length)))
    .sort(compareVersions);
  return images.at(-1) ?? null;
}

/** Newest last. Compares the version triples, not the strings, so 0.10 beats
 *  0.9 - which a lexical sort gets backwards. */
function compareVersions(a: string, b: string): number {
  const parse = (key: string) =>
    (versionOf(key) ?? "0.0.0").split(".").map(Number);
  const [x, y] = [parse(a), parse(b)];
  for (let index = 0; index < 3; index += 1) {
    const difference = (x[index] ?? 0) - (y[index] ?? 0);
    if (difference !== 0) return difference;
  }
  return a.localeCompare(b);
}

/**
 * A page view, reported by the landing page.
 *
 * A one pixel response rather than a script: nothing is read from the visitor's
 * browser, nothing is stored on their machine, and there is no cookie - so
 * there is nothing to ask consent for and nothing to disclose that this comment
 * does not already say.
 */
async function beacon(
  request: Request,
  env: Env,
  ctx: ExecutionContext,
  url: URL,
): Promise<Response> {
  const shape = classify(request);
  ctx.waitUntil(
    recordVisit(env, {
      classification: shape,
      request,
      path: url.searchParams.get("p")?.slice(0, 128) ?? "/",
      // Sent by the page, not read from `Referer`.
      //
      // The beacon is a `fetch` from the landing page itself, so its `Referer`
      // is the landing page - which made `visit.referrer_host` able to hold
      // exactly one value and the comment on the page, promising "which host
      // linked here", describe something that was never recorded. The host is
      // taken from `document.referrer` there and passed here.
      referrer: inboundHost(url.searchParams.get("r")),
    }),
  );

  return new Response(null, {
    status: 204,
    headers: {
      "cache-control": "no-store",
      "access-control-allow-origin": "*",
    },
  });
}

/**
 * Takes a message from the form on the site.
 *
 * Stored before it is sent, and answered before the sending finishes: the
 * person is told their message arrived the moment it is safely in the database,
 * because that is the moment it is true. Waiting on an email provider would
 * make them watch a spinner for something that has already succeeded.
 */
async function feedback(
  request: Request,
  env: Env,
  ctx: ExecutionContext,
): Promise<Response> {
  const origin = request.headers.get("origin") ?? "";
  const cors: Record<string, string> = {
    "access-control-allow-methods": "POST, OPTIONS",
    "access-control-allow-headers": "content-type",
    "cache-control": "no-store",
  };
  // Omitted rather than set to "null" when the origin is not allowed. `null`
  // is a real origin - a sandboxed iframe, a `data:` document - so sending it
  // grants those contexts the access it was meant to deny. No header at all is
  // the refusal.
  if (ALLOWED.includes(origin)) {
    cors["access-control-allow-origin"] = origin;
  }

  if (request.method === "OPTIONS") {
    return new Response(null, { status: 204, headers: cors });
  }
  if (request.method !== "POST") {
    return new Response("Method not allowed", { status: 405, headers: cors });
  }

  let form: FormData;
  try {
    form = await request.formData();
  } catch {
    return json({ error: "Could not read the form." }, 400, cors);
  }

  const parsed = parse(form);
  if ("error" in parsed) {
    // The honeypot is answered as though it worked. A bot told it failed comes
    // back having changed something; one told it succeeded does not.
    if (parsed.error === "honeypot") {
      return json({ ok: true }, 200, cors);
    }
    return json({ error: parsed.error }, 400, cors);
  }

  const cf = (request as { cf?: Record<string, unknown> }).cf ?? {};
  const country = (cf["country"] as string | undefined) ?? null;
  const at = new Date().toISOString();

  let id: number | null = null;
  try {
    const written = await env.ANALYTICS.prepare(
      `INSERT INTO feedback (at, message, contact, version, os, country)
       VALUES (?, ?, ?, ?, ?, ?) RETURNING id`,
    )
      .bind(
        at,
        parsed.message,
        parsed.contact || null,
        parsed.version || null,
        parsed.os || null,
        country,
      )
      .first<{ id: number }>();
    id = written?.id ?? null;
  } catch (error) {
    // The one failure worth telling the sender about. If it is not stored, it
    // is lost, and they should write it down somewhere else.
    console.error("chandra: could not store feedback", error);
    return json({ error: "Could not save that. Please try again." }, 500, cors);
  }

  // The email happens after the answer. It is a notification about a row that
  // already exists, not the delivery itself.
  ctx.waitUntil(
    (async () => {
      const outcome = await notify(env, parsed, country);
      if (id === null) return;
      try {
        await env.ANALYTICS.prepare(
          `UPDATE feedback SET delivered = ?, error = ? WHERE id = ?`,
        )
          .bind(outcome.delivered, outcome.error, id)
          .run();
      } catch (error) {
        console.error("chandra: could not record delivery", error);
      }
    })(),
  );

  return json({ ok: true }, 200, cors);
}

/** Where the form may be served from. Anywhere else gets no CORS header and
 *  the browser refuses the response, which is the point. */
const ALLOWED = [
  "https://chandra.paraxis.dev",
  // `tools/release.sh` deploys `--project-name=chandra`, so this is the host
  // the form is served from until the custom domain is attached. It read
  // `chandra-dis.pages.dev`, which exists nowhere else in the repository: a
  // FormData POST is CORS-simple and needs no preflight, so the row was stored
  // and the browser then blocked the response - leaving the sender told "that
  // did not go through" and invited to send it again.
  "https://chandra.pages.dev",
];

function json(
  body: unknown,
  status: number,
  headers: Record<string, string>,
): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { ...headers, "content-type": "application/json" },
  });
}

/** What every row carries, whichever table it lands in. */
async function common(
  env: Env,
  request: Request,
): Promise<{
  at: string;
  day: string;
  country: string | null;
  visitor: string | null;
}> {
  const at = new Date().toISOString();
  const cf = (request as { cf?: Record<string, unknown> }).cf ?? {};
  return {
    at,
    day: at.slice(0, 10),
    country: (cf["country"] as string | undefined) ?? null,
    visitor: await visitorHash(request, env.VISITOR_SALT),
  };
}

async function recordDownload(
  env: Env,
  event: {
    asset: string | null;
    version: string | null;
    classification: ReturnType<typeof classify>;
    request: Request;
  },
): Promise<void> {
  try {
    const { at, day, country, visitor } = await common(env, event.request);
    const { verdict, reason, osVersion, arch, browser } = event.classification;
    await env.ANALYTICS.prepare(
      `INSERT INTO download
         (at, day, asset, version, verdict, reason,
          os_version, arch, browser, country, referrer_host, visitor)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
    )
      .bind(
        at,
        day,
        event.asset,
        event.version,
        verdict,
        reason,
        osVersion,
        arch,
        browser,
        country,
        // A real inbound referrer. The download link was followed from
        // somewhere and the browser says where.
        referrerHost(event.request),
        visitor,
      )
      .run();
  } catch (error) {
    swallow(error);
  }
}

async function recordVisit(
  env: Env,
  event: {
    classification: ReturnType<typeof classify>;
    request: Request;
    path: string;
    referrer: string | null;
  },
): Promise<void> {
  try {
    const { at, day, country, visitor } = await common(env, event.request);
    const { verdict, reason } = event.classification;
    await env.ANALYTICS.prepare(
      `INSERT INTO visit
         (at, day, path, verdict, reason, country, referrer_host, visitor)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?)`,
    )
      .bind(
        at,
        day,
        event.path,
        verdict,
        reason,
        country,
        event.referrer,
        visitor,
      )
      .run();
  } catch (error) {
    swallow(error);
  }
}

/** One table's worth of arithmetic is not worth losing a download over. */
function swallow(error: unknown): void {
  // Runs in `waitUntil`, after the response has been sent. A failure to count
  // must never be a failure to download.
  console.error("chandra: could not record event", error);
}
