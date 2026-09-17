/**
 * Is this request a person downloading the app, or something else?
 *
 * The question matters because a download count is the only number the project
 * has, and it is the one most easily inflated: a public .dmg URL is fetched by
 * search crawlers, link previewers in every chat app, security scanners, mirror
 * bots and AI agents, none of which will ever run it. Counting those makes the
 * number meaningless in the direction that flatters.
 *
 * Three rules shape this.
 *
 * **Classify, never block.** Every request is served, whatever the verdict. A
 * classifier that blocks turns a false positive into a person who cannot get
 * the app, and there is no feedback loop that would reveal it. A classifier
 * that only counts is wrong in a way that costs nothing but a number.
 *
 * **Say why.** Every verdict carries the rule that produced it, stored beside
 * it. A count that cannot be audited is a guess with a database behind it; with
 * the reason recorded, a rule that turns out to be wrong can be corrected in
 * SQL over data already collected rather than needing it to have been kept
 * differently.
 *
 * **Prove you are a Mac.** This is the unusual advantage here, and it does more
 * work than any blocklist: the artefact is a macOS disk image, so a genuine
 * downloader is a browser on a Mac. That is a positive test - an allowlist of
 * the one thing a real user looks like - rather than an endless chase after
 * whatever a crawler currently calls itself.
 */

export type Verdict = "human" | "bot" | "probe";

export interface Classification {
  verdict: Verdict;
  reason: string;
  osVersion: string | null;
  arch: "apple-silicon" | "intel" | "unknown";
  browser: string;
}

/**
 * Agents that say what they are. Matched case-insensitively as substrings.
 *
 * Not the defence - the Mac test below is. This exists so the *reason* on a row
 * names the thing rather than saying "not a Mac browser", which makes the
 * traffic legible when someone asks where a spike came from.
 *
 * AI crawlers are listed first because they are the newest and the most likely
 * to be missing from a list copied from somewhere else.
 */
const DECLARED_BOTS: readonly string[] = [
  // AI crawlers and agents.
  "gptbot",
  "oai-searchbot",
  "chatgpt-user",
  "claudebot",
  "claude-web",
  "anthropic-ai",
  "perplexitybot",
  "perplexity-user",
  "google-extended",
  "applebot-extended",
  "ccbot",
  "bytespider",
  "amazonbot",
  "meta-externalagent",
  "facebookbot",
  "cohere-ai",
  "diffbot",
  "omgili",
  "timpibot",
  "youbot",
  "ai2bot",
  "imagesiftbot",
  "webzio",
  // Search and general crawlers.
  "googlebot",
  "bingbot",
  "yandexbot",
  "baiduspider",
  "duckduckbot",
  "slurp",
  "applebot",
  "petalbot",
  "sogou",
  "exabot",
  "seznambot",
  "mj12bot",
  "ahrefsbot",
  "semrushbot",
  "dotbot",
  "dataforseo",
  "screaming frog",
  // Link previewers. These fetch on behalf of a person, but the person is
  // reading a chat message, not installing anything.
  "slackbot",
  "discordbot",
  "twitterbot",
  "telegrambot",
  "whatsapp",
  "linkedinbot",
  "redditbot",
  "skypeuripreview",
  "embedly",
  "quora link preview",
  "pinterest",
  "vkshare",
  "facebookexternalhit",
  // Tooling. A real person may well use these - but not to install a Mac app,
  // and a mirror script should not read as adoption.
  "curl/",
  "wget",
  "python-requests",
  "python-urllib",
  "libwww-perl",
  "go-http-client",
  "java/",
  "okhttp",
  "axios",
  "node-fetch",
  "got (",
  "httpie",
  "aria2",
  "insomnia",
  "postman",
  // Monitoring and security.
  "uptimerobot",
  "pingdom",
  "statuscake",
  "site24x7",
  "datadog",
  "newrelic",
  "zgrab",
  "masscan",
  "nmap",
  "censys",
  "shodan",
  "internetmeasurement",
  "expanse",
  // Headless browsers announcing themselves.
  "headlesschrome",
  "phantomjs",
  "electron/",
  "puppeteer",
  "playwright",
];

/** Datacentre networks. Almost nobody installs a Mac app from one. */
const DATACENTRE = [
  "amazon",
  "aws",
  "google",
  "microsoft",
  "azure",
  "digitalocean",
  "linode",
  "hetzner",
  "ovh",
  "scaleway",
  "vultr",
  "oracle",
  "alibaba",
  "tencent",
  "contabo",
  "leaseweb",
  "choopa",
  "m247",
  "datacamp",
];

/** The macOS version in a browser user agent, as `14` or `10.15`. */
function macOsVersion(agent: string): string | null {
  const match = /mac os x (\d+)([._](\d+))?/i.exec(agent);
  if (!match) return null;
  const major = match[1]!;
  const minor = match[3];
  // Before Big Sur the minor number is the release: 10.15 is Catalina, and 10
  // alone says nothing. From 11 onward the major number is the release.
  return major === "10" && minor ? `10.${minor}` : major;
}

/**
 * Apple Silicon or Intel, where the request says.
 *
 * Safari and Chrome both still report `Intel Mac OS X` on Apple Silicon, for
 * compatibility - so this is a *floor*, not a census. Anything reporting arm64
 * is certainly Apple Silicon; the rest is unknown rather than Intel, and the
 * insight query says so rather than letting it read as a split.
 */
function architecture(agent: string): Classification["arch"] {
  if (/arm64|aarch64|apple silicon/i.test(agent)) return "apple-silicon";
  if (/intel mac os x/i.test(agent)) return "unknown";
  return "unknown";
}

function browserName(agent: string): string {
  if (/edg\//i.test(agent)) return "edge";
  if (/opr\/|opera/i.test(agent)) return "opera";
  if (/firefox\//i.test(agent)) return "firefox";
  if (/chrome\/|crios/i.test(agent)) return "chrome";
  if (/safari\//i.test(agent)) return "safari";
  return "other";
}

/**
 * Whether the request carries the marks a browser navigation makes.
 *
 * `Sec-Fetch-*` is sent by every current browser and by almost nothing else,
 * and cannot be set by page script. `Accept-Language` is sent by browsers and
 * routinely omitted by scripts. Neither is proof - both are forgeable by
 * anything deliberate - but together they separate a browser from a fetch loop
 * that has merely copied a user agent string, which is most of them.
 */
function looksLikeBrowser(headers: Headers): boolean {
  const site = headers.get("sec-fetch-site");
  const mode = headers.get("sec-fetch-mode");
  return Boolean(site || mode) && Boolean(headers.get("accept-language"));
}

export function classify(request: Request): Classification {
  const headers = request.headers;
  const agent = headers.get("user-agent") ?? "";
  const lower = agent.toLowerCase();
  const cf = (request as { cf?: Record<string, unknown> }).cf ?? {};

  const shape: Omit<Classification, "verdict" | "reason"> = {
    osVersion: macOsVersion(agent),
    arch: architecture(agent),
    browser: browserName(agent),
  };

  const as = (verdict: Verdict, reason: string): Classification => ({
    verdict,
    reason,
    ...shape,
  });

  // A HEAD or a Range is a question about the file, not a copy of it. Chat
  // apps and download managers both do this, and a download manager will be
  // back for the body - which is the request that gets counted.
  if (request.method === "HEAD") return as("probe", "head request");
  if (headers.has("range")) return as("probe", "range request");

  if (!agent) return as("bot", "no user agent");

  for (const name of DECLARED_BOTS) {
    if (lower.includes(name)) return as("bot", `declared: ${name.trim()}`);
  }

  // Cloudflare's own judgement, where the plan provides it. Checked before the
  // heuristics because a verified crawler is a fact rather than an inference.
  const verified = cf["verifiedBotCategory"];
  if (typeof verified === "string" && verified.length > 0) {
    return as("bot", `verified bot: ${verified}`);
  }
  const bot = cf["botManagement"] as { score?: number } | undefined;
  if (typeof bot?.score === "number" && bot.score <= 30) {
    return as("bot", `bot score ${bot.score}`);
  }

  // The positive test. The artefact only runs on macOS, so a real downloader
  // says it is a Mac. Anything else is something looking at the file rather
  // than someone about to use it - including a genuine person on Windows, whose
  // download is real but is not adoption of a Mac app.
  if (!/macintosh|mac os x/i.test(agent)) {
    return as("bot", "not a mac");
  }
  if (shape.browser === "other") {
    return as("bot", "mac user agent, unrecognised browser");
  }
  if (!looksLikeBrowser(headers)) {
    return as("bot", "no browser fetch headers");
  }

  const organisation = String(cf["asOrganization"] ?? "").toLowerCase();
  if (organisation && DATACENTRE.some((name) => organisation.includes(name))) {
    return as("bot", `datacentre: ${organisation}`);
  }

  return as("human", "mac browser");
}
