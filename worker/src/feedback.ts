/**
 * Feedback, from the form the app links out to.
 *
 * The app does not send this - it has no network code and three surfaces say
 * so. Its About pane opens a browser at a page on the site, which posts here.
 * That costs some completion rate against an in-app form, and buys keeping the
 * claim "nothing is sent anywhere" literally true, which for an app holding
 * somebody's home coordinates is worth more.
 *
 * **Stored first, emailed second.** The row is the record and the email is a
 * notification. A credential that expires, a provider that is down or a quota
 * that runs out must not lose a message somebody took the trouble to write and
 * has no way of knowing vanished.
 */

import type { Env } from "./index";

/**
 * The bounds, and the only place they are written.
 *
 * `site/feedback.html` enforces the same four numbers in the browser so a
 * mistake is caught without a round trip, and `tools/check-limits.sh` fails
 * the build if the two ever disagree. The form is a convenience; this is the
 * authority, because anything can post here.
 */
export const LIMITS = {
  /** Long enough for a real bug report, short enough that the endpoint is not
   *  storage. Measured against the longest useful report: a paragraph of
   *  symptoms, a paragraph of steps and a version string. */
  message: 4000,
  /** Below this, a message is a test or a slip of the keyboard. */
  floor: 4,
  version: 32,
  os: 120,
} as const;

export interface Feedback {
  message: string;
  contact: string;
  version: string;
  os: string;
}

/** What the form sent, if it is usable. */
export function parse(form: FormData): Feedback | { error: string } {
  const text = (name: string, max: number) =>
    String(form.get(name) ?? "")
      .trim()
      .slice(0, max);

  // A field no person sees and no person fills. Bots fill everything, so an
  // empty one is weak evidence of a human and a full one is strong evidence of
  // a bot. Accepted and discarded rather than refused: a bot told it failed
  // tries again differently.
  if (text("website", 200)) {
    return { error: "honeypot" };
  }

  // Refused, not truncated. `slice` would have taken the first 4000 characters
  // of a longer report, stored them, and answered "Thank you. It has arrived."
  // - so somebody who wrote five thousand would have lost the last thousand
  // with no way of ever knowing. The whole point of storing before emailing is
  // that a message nobody can tell went missing is the worst outcome here.
  const whole = String(form.get("message") ?? "").trim();
  if (whole.length < LIMITS.floor) {
    return { error: "Please write a little more than that." };
  }
  if (whole.length > LIMITS.message) {
    return {
      error: `That is ${whole.length} characters and the limit is ${LIMITS.message}. Please trim it - nothing has been sent, so your text is still in the box.`,
    };
  }
  const message = whole;

  const contact = text("contact", 200);
  // Not validated beyond this. A strict address pattern rejects real addresses
  // and the only cost of a wrong one is a reply that bounces - which is the
  // sender's problem to notice, not a reason to refuse their report.
  if (contact && !contact.includes("@")) {
    return { error: "That does not look like an email address." };
  }

  return {
    message,
    contact,
    version: text("version", LIMITS.version),
    os: text("os", LIMITS.os),
  };
}

/**
 * Sends the notification, if a sender is configured.
 *
 * Deliberately provider-shaped rather than provider-specific: the API key is a
 * secret and the endpoint is a variable, so moving from one sender to another
 * is configuration rather than a deploy. Inbound mail for the domain is
 * unaffected by any of this - sending authorisation and MX are separate, which
 * is what lets the domain keep receiving where it already does.
 */
export async function notify(
  env: Env,
  feedback: Feedback,
  country: string | null,
): Promise<{ delivered: string; error: string | null }> {
  if (!env.MAIL_API_KEY || !env.MAIL_FROM || !env.MAIL_TO) {
    return { delivered: "pending", error: "no sender configured" };
  }

  const lines = [
    feedback.message,
    "",
    "—",
    `Version: ${feedback.version || "not given"}`,
    `System:  ${feedback.os || "not given"}`,
    `Country: ${country ?? "unknown"}`,
    `Reply:   ${feedback.contact || "no address given"}`,
  ];

  try {
    const response = await fetch("https://api.resend.com/emails", {
      method: "POST",
      headers: {
        authorization: `Bearer ${env.MAIL_API_KEY}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({
        from: env.MAIL_FROM,
        to: [env.MAIL_TO],
        // The reply goes to the person, not to the robot - so answering is
        // hitting reply rather than copying an address out of the body.
        ...(feedback.contact ? { reply_to: feedback.contact } : {}),
        subject: `Chandra feedback${feedback.version ? ` · ${feedback.version}` : ""}`,
        text: lines.join("\n"),
      }),
    });

    if (!response.ok) {
      return {
        delivered: "failed",
        error: `${response.status} ${(await response.text()).slice(0, 200)}`,
      };
    }
    return { delivered: "sent", error: null };
  } catch (thrown) {
    return { delivered: "failed", error: String(thrown).slice(0, 200) };
  }
}
