// Fetches the official icon of each claude.ai tool the app knows (spec
// 34.7) into src/assets/tools/<key>.png (or .jpg), from where claude.ai's own
// directory takes it: Google's product images, or the site's icon through
// Google's favicon service. Run by hand when the catalog grows
// (`node scripts/tool-icons.mjs [key...]`); the app never fetches them.
// The marks belong to their owners and only name the service (README).

import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const OUT = join(dirname(fileURLToPath(import.meta.url)), "..", "src", "assets", "tools");

const favicon = (site) =>
  `https://t0.gstatic.com/faviconV2?client=SOCIAL&type=FAVICON&fallback_opts=TYPE,SIZE,URL&url=${site}&size=256`;
const google = (product) =>
  `https://www.gstatic.com/images/branding/product/2x/${product}_2020q4_48dp.png`;

export const SOURCES = {
  gmail: google("gmail"),
  google_calendar: google("calendar"),
  google_drive: google("drive"),
  google_docs: google("docs"),
  google_sheets: google("sheets"),
  google_slides: google("slides"),
  microsoft_365: favicon("https://www.microsoft.com"),
  calendly: favicon("https://calendly.com"),
  slack: favicon("https://api.slack.com"),
  zoom: favicon("https://app.zoom.us"),
  webex: favicon("https://www.webex.com"),
  intercom: favicon("https://www.intercom.com"),
  granola: favicon("https://www.granola.ai"),
  fireflies: favicon("https://fireflies.ai"),
  otter: favicon("https://otter.ai"),
  fathom: favicon("https://fathom.ai"),
  notion: favicon("https://www.notion.so"),
  atlassian: favicon("https://www.atlassian.com"),
  box: favicon("https://www.box.com"),
  dropbox: favicon("https://dash.dropbox.com"),
  docusign: favicon("https://www.docusign.com"),
  asana: favicon("https://asana.com"),
  linear: favicon("https://linear.app"),
  clickup: favicon("https://clickup.com"),
  monday: favicon("https://monday.com"),
  trello: favicon("https://trello.com"),
  todoist: favicon("https://todoist.com"),
  airtable: favicon("https://airtable.com"),
  miro: favicon("https://miro.com"),
  figma: favicon("https://www.figma.com"),
  canva: favicon("https://canva.com"),
  adobe: favicon("https://www.adobe.com"),
  webflow: favicon("https://webflow.com"),
  framer: favicon("https://www.framer.com"),
  wix: favicon("https://www.wix.com"),
  wordpress: favicon("https://wordpress.com"),
  shopify: favicon("https://www.shopify.com"),
  higgsfield: favicon("https://www.higgsfield.ai"),
  heygen: favicon("https://app.heygen.com"),
  gamma: favicon("https://gamma.app"),
  vercel: favicon("https://vercel.com"),
  netlify: favicon("https://www.netlify.com"),
  cloudflare: favicon("https://www.cloudflare.com"),
  supabase: favicon("https://supabase.com"),
  neon: favicon("https://neon.com"),
  mongodb: favicon("https://www.mongodb.com"),
  sentry: favicon("https://sentry.io"),
  datadog: favicon("https://www.datadoghq.com"),
  pagerduty: favicon("https://www.pagerduty.com"),
  grafana: favicon("https://play.grafana.org"),
  postman: favicon("https://www.postman.com"),
  hugging_face: favicon("https://huggingface.co"),
  exa: favicon("https://dashboard.exa.ai"),
  jam: favicon("https://jam.dev"),
  bigquery:
    "https://storage.googleapis.com/media-assets-299d7136-cb52-d546-ee02-34bc1307c35f/mcp-directory/icons/bigquery.png",
  snowflake: favicon("https://www.snowflake.com"),
  databricks: favicon("https://www.databricks.com"),
  amplitude: favicon("https://amplitude.com"),
  mixpanel: favicon("https://mixpanel.com"),
  posthog: favicon("https://posthog.com"),
  hex: favicon("https://hex.tech"),
  hubspot: favicon("https://www.hubspot.com"),
  salesforce: favicon("https://login.salesforce.com"),
  mailchimp: favicon("https://mailchimp.com"),
  apollo: favicon("https://www.apollo.io"),
  zoominfo: favicon("https://www.zoominfo.com"),
  clay: favicon("https://www.clay.com"),
  attio: favicon("https://app.attio.com"),
  klaviyo: favicon("https://www.klaviyo.com"),
  similarweb: favicon("https://www.similarweb.com"),
  ahrefs: favicon("https://ahrefs.com"),
  stripe: favicon("https://stripe.com"),
  paypal: favicon("https://www.paypal.com"),
  square: favicon("https://issquareup.com"),
  quickbooks: favicon("https://quickbooks.intuit.com"),
  xero: favicon("https://www.xero.com"),
  spotify: favicon("https://www.spotify.com"),
  booking: favicon("https://www.booking.com"),
  zapier: favicon("https://zapier.com"),
  make: favicon("https://www.make.com"),
  n8n: favicon("https://n8n.io"),
};

const PNG = [137, 80, 78, 71, 13, 10, 26, 10];

async function fetchOne(key) {
  const response = await fetch(SOURCES[key]);
  const bytes = new Uint8Array(await response.arrayBuffer());
  if (!response.ok) {
    return `${key}: HTTP ${response.status}`;
  }
  const png = PNG.every((byte, at) => bytes[at] === byte);
  const jpeg = bytes[0] === 0xff && bytes[1] === 0xd8 && bytes[2] === 0xff;
  if (!png && !jpeg) {
    return `${key}: not an image (${response.headers.get("content-type")})`;
  }
  const view = new DataView(bytes.buffer);
  const size = png ? `${view.getUint32(16)}x${view.getUint32(20)}` : "jpeg";
  for (const old of [".png", ".jpg"]) {
    rmSync(join(OUT, `${key}${old}`), { force: true });
  }
  writeFileSync(join(OUT, `${key}${png ? ".png" : ".jpg"}`), bytes);
  return `${key}: ${size}, ${bytes.length} bytes`;
}

const wanted = process.argv.slice(2);
const keys = wanted.length > 0 ? wanted : Object.keys(SOURCES);
mkdirSync(OUT, { recursive: true });
for (const key of keys) {
  if (!SOURCES[key]) {
    console.log(`${key}: no source`);
    continue;
  }
  console.log(await fetchOne(key).catch((error) => `${key}: ${error.message}`));
}
