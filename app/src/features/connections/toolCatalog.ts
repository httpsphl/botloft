// The claude.ai tools the app knows by name (spec 34.7): the kind it is
// and the key of its official icon and description. The icons are in
// src/assets/tools, fetched once by scripts/tool-icons.mjs from where
// claude.ai's own directory takes them. "Discover" lists the ones the
// account does not have yet, by kind.

import type { Messages } from "../../i18n/en";

export type ToolKey = keyof Messages["connections"]["about"];
export type ToolKind = keyof Messages["connections"]["toolKinds"];

export interface KnownTool {
  key: ToolKey;
  /** As claude.ai names it, or how its name starts ("Zoom for Claude"). */
  name: string;
  kind: ToolKind;
}

const ICONS = import.meta.glob<string>("../../assets/tools/*.{png,jpg}", {
  eager: true,
  query: "?no-inline",
  import: "default",
});

/** The tool's official icon, as a URL the app serves. */
export function toolIcon(key: ToolKey): string | undefined {
  return ICONS[`../../assets/tools/${key}.png`] ?? ICONS[`../../assets/tools/${key}.jpg`];
}

const tool = (key: ToolKey, name: string, kind: ToolKind): KnownTool => ({ key, name, kind });

export const KNOWN_TOOLS: KnownTool[] = [
  tool("gmail", "Gmail", "mail"),
  tool("google_calendar", "Google Calendar", "mail"),
  tool("microsoft_365", "Microsoft 365", "mail"),
  tool("calendly", "Calendly", "mail"),
  tool("slack", "Slack", "talk"),
  tool("zoom", "Zoom", "talk"),
  tool("webex", "Webex Meetings", "talk"),
  tool("intercom", "Intercom", "talk"),
  tool("granola", "Granola", "talk"),
  tool("fireflies", "Fireflies", "talk"),
  tool("otter", "Otter.ai", "talk"),
  tool("fathom", "Fathom", "talk"),
  tool("google_drive", "Google Drive", "files"),
  tool("google_docs", "Google Docs", "files"),
  tool("google_sheets", "Google Sheets", "files"),
  tool("google_slides", "Google Slides", "files"),
  tool("notion", "Notion", "files"),
  tool("atlassian", "Atlassian", "files"),
  tool("box", "Box", "files"),
  tool("dropbox", "Dropbox", "files"),
  tool("docusign", "Docusign", "files"),
  tool("asana", "Asana", "work"),
  tool("linear", "Linear", "work"),
  tool("clickup", "ClickUp", "work"),
  tool("monday", "monday.com", "work"),
  tool("trello", "Trello", "work"),
  tool("todoist", "Todoist", "work"),
  tool("airtable", "Airtable", "work"),
  tool("miro", "Miro", "work"),
  tool("figma", "Figma", "design"),
  tool("canva", "Canva", "design"),
  tool("adobe", "Adobe", "design"),
  tool("gamma", "Gamma", "design"),
  tool("higgsfield", "Higgsfield", "design"),
  tool("heygen", "HyperFrames by HeyGen", "design"),
  tool("webflow", "Webflow", "design"),
  tool("framer", "Framer", "design"),
  tool("wix", "Wix", "design"),
  tool("wordpress", "WordPress.com", "design"),
  tool("shopify", "Shopify", "design"),
  tool("vercel", "Vercel", "code"),
  tool("netlify", "Netlify", "code"),
  tool("cloudflare", "Cloudflare", "code"),
  tool("supabase", "Supabase", "code"),
  tool("neon", "Neon", "code"),
  tool("mongodb", "MongoDB", "code"),
  tool("sentry", "Sentry", "code"),
  tool("datadog", "Datadog", "code"),
  tool("pagerduty", "PagerDuty", "code"),
  tool("grafana", "Grafana", "code"),
  tool("postman", "Postman", "code"),
  tool("jam", "Jam", "code"),
  tool("hugging_face", "Hugging Face", "code"),
  tool("exa", "Exa", "code"),
  tool("bigquery", "Google Cloud BigQuery", "data"),
  tool("snowflake", "Snowflake", "data"),
  tool("databricks", "Databricks", "data"),
  tool("hex", "Hex", "data"),
  tool("amplitude", "Amplitude", "data"),
  tool("mixpanel", "Mixpanel", "data"),
  tool("posthog", "PostHog", "data"),
  tool("hubspot", "HubSpot", "sales"),
  tool("salesforce", "Salesforce", "sales"),
  tool("attio", "Attio", "sales"),
  tool("apollo", "Apollo.io", "sales"),
  tool("clay", "Clay", "sales"),
  tool("zoominfo", "ZoomInfo", "sales"),
  tool("mailchimp", "Intuit Mailchimp", "sales"),
  tool("klaviyo", "Klaviyo", "sales"),
  tool("ahrefs", "Ahrefs", "sales"),
  tool("similarweb", "Similarweb", "sales"),
  tool("stripe", "Stripe", "money"),
  tool("paypal", "PayPal", "money"),
  tool("square", "Square", "money"),
  tool("quickbooks", "Intuit QuickBooks", "money"),
  tool("xero", "Xero", "money"),
  tool("spotify", "Spotify", "everyday"),
  tool("booking", "Booking.com", "everyday"),
  tool("zapier", "Zapier", "automation"),
  tool("make", "Make", "automation"),
  tool("n8n", "n8n", "automation"),
];

/** The kinds, in the order Discover shows them. */
export const TOOL_KINDS: ToolKind[] = [
  "mail",
  "talk",
  "files",
  "work",
  "design",
  "code",
  "data",
  "sales",
  "money",
  "everyday",
  "automation",
];

/**
 * A tool of the account by its name, when the app knows it: the same name,
 * or one that starts with it as a whole word ("Atlassian MCP", "Zoom for
 * Claude"), so "Make" is not "Makeswift".
 */
export function knownTool(name: string): KnownTool | undefined {
  const wanted = name.trim().toLowerCase();
  return KNOWN_TOOLS.find((tool) => {
    const have = tool.name.toLowerCase();
    return wanted.startsWith(have) && !/[\p{L}\p{N}]/u.test(wanted.charAt(have.length));
  });
}
