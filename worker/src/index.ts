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

/** Where the redirect goes when someone asks for the current release. */
const LATEST = "latest";

/** The only prefix this worker may read from.
 *
 * The bucket is shared with another project. Without this, the key came
 * straight from the URL path, so `/download/<anything>` served *any* object in
 * it - twenty-three of them, belonging to something else - through a public
 * endpoint. Broken access control, in eleven characters of missing check. */
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
    record(env, "download", {
      asset: key,
      version: versionOf(key),
      classification: shape,
      request,
      path: url.pathname,
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
  // hard. `/download` itself must not be, or a new release would be invisible
  // to anyone who had followed the link before.
  headers.set("cache-control", "public, max-age=31536000, immutable");
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
  const images = listed.objects
    .map((object) => object.key)
    .filter((key) => key.endsWith(".dmg"))
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
    record(env, "visit", {
      asset: null,
      version: null,
      classification: shape,
      request,
      path: url.searchParams.get("p")?.slice(0, 128) ?? "/",
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
    "access-control-allow-origin": ALLOWED.includes(origin) ? origin : "null",
    "access-control-allow-methods": "POST, OPTIONS",
    "access-control-allow-headers": "content-type",
    "cache-control": "no-store",
  };

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
  "https://chandra-dis.pages.dev",
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

async function record(
  env: Env,
  table: "download" | "visit",
  event: {
    asset: string | null;
    version: string | null;
    classification: ReturnType<typeof classify>;
    request: Request;
    path: string;
  },
): Promise<void> {
  try {
    const now = new Date();
    const at = now.toISOString();
    const day = at.slice(0, 10);
    const cf =
      (event.request as { cf?: Record<string, unknown> }).cf ?? {};
    const country = (cf["country"] as string | undefined) ?? null;
    const visitor = await visitorHash(event.request, env.VISITOR_SALT);
    const { verdict, reason, osVersion, arch, browser } = event.classification;

    if (table === "download") {
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
          referrerHost(event.request),
          visitor,
        )
        .run();
      return;
    }

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
        referrerHost(event.request),
        visitor,
      )
      .run();
  } catch (error) {
    // Deliberately swallowed. This runs in `waitUntil`, after the response has
    // been sent, and a failure to count must never be a failure to download.
    console.error("chandra: could not record event", error);
  }
}
