// The claude.ai tools the app knows by name (spec 34.7): the brand's mark
// from Simple Icons (CC0), or its initial when Simple Icons has none, its
// color, and the key of its description. "Discover" lists the ones the
// account does not have yet, by kind.

import type { SimpleIcon } from "simple-icons";
import {
  siAirtable,
  siAsana,
  siAtlassian,
  siBox,
  siCalendly,
  siClickup,
  siCloudflare,
  siDatabricks,
  siDatadog,
  siDropbox,
  siFathom,
  siFigma,
  siFramer,
  siGmail,
  siGooglebigquery,
  siGooglecalendar,
  siGoogledrive,
  siGrafana,
  siHubspot,
  siHuggingface,
  siIntercom,
  siLinear,
  siMailchimp,
  siMake,
  siMiro,
  siMixpanel,
  siMongodb,
  siN8n,
  siNeon,
  siNetlify,
  siNotion,
  siPagerduty,
  siPaypal,
  siPosthog,
  siPostman,
  siQuickbooks,
  siSentry,
  siShopify,
  siSnowflake,
  siSquare,
  siStripe,
  siSupabase,
  siTodoist,
  siTrello,
  siVercel,
  siWebex,
  siWebflow,
  siWix,
  siWordpress,
  siXero,
  siZapier,
  siZoom,
} from "simple-icons";
import type { Messages } from "../../i18n/en";

export type ToolKey = keyof Messages["connections"]["about"];
export type ToolKind = keyof Messages["connections"]["toolKinds"];

export interface KnownTool {
  key: ToolKey;
  /** As claude.ai names it, or how its name starts ("Zoom for Claude"). */
  name: string;
  kind: ToolKind;
  /** The brand's mark; without one, the tile shows the initial. */
  mark?: SimpleIcon;
  /** The brand's color, as `#rrggbb`. */
  color: string;
}

const tool = (key: ToolKey, name: string, kind: ToolKind, mark: SimpleIcon): KnownTool => ({
  key,
  name,
  kind,
  mark,
  color: `#${mark.hex}`,
});

/** A brand Simple Icons does not draw: its initial on its color. */
const initial = (key: ToolKey, name: string, kind: ToolKind, color: string): KnownTool => ({
  key,
  name,
  kind,
  color,
});

export const KNOWN_TOOLS: KnownTool[] = [
  tool("gmail", "Gmail", "mail", siGmail),
  tool("google_calendar", "Google Calendar", "mail", siGooglecalendar),
  initial("microsoft_365", "Microsoft 365", "mail", "#0078D4"),
  tool("calendly", "Calendly", "mail", siCalendly),
  initial("slack", "Slack", "talk", "#4A154B"),
  tool("zoom", "Zoom", "talk", siZoom),
  tool("webex", "Webex Meetings", "talk", siWebex),
  tool("intercom", "Intercom", "talk", siIntercom),
  initial("granola", "Granola", "talk", "#1F1F1C"),
  initial("fireflies", "Fireflies", "talk", "#D6246E"),
  initial("otter", "Otter.ai", "talk", "#1C4CF4"),
  tool("fathom", "Fathom", "talk", siFathom),
  tool("google_drive", "Google Drive", "files", siGoogledrive),
  tool("notion", "Notion", "files", siNotion),
  tool("atlassian", "Atlassian", "files", siAtlassian),
  tool("box", "Box", "files", siBox),
  tool("dropbox", "Dropbox", "files", siDropbox),
  initial("docusign", "Docusign", "files", "#4C00FF"),
  tool("asana", "Asana", "work", siAsana),
  tool("linear", "Linear", "work", siLinear),
  tool("clickup", "ClickUp", "work", siClickup),
  initial("monday", "monday.com", "work", "#6161FF"),
  tool("trello", "Trello", "work", siTrello),
  tool("todoist", "Todoist", "work", siTodoist),
  tool("airtable", "Airtable", "work", siAirtable),
  tool("miro", "Miro", "work", siMiro),
  tool("figma", "Figma", "design", siFigma),
  initial("canva", "Canva", "design", "#00C4CC"),
  initial("adobe", "Adobe", "design", "#FA0F00"),
  tool("webflow", "Webflow", "design", siWebflow),
  tool("framer", "Framer", "design", siFramer),
  tool("wix", "Wix", "design", siWix),
  tool("wordpress", "WordPress.com", "design", siWordpress),
  tool("shopify", "Shopify", "design", siShopify),
  tool("vercel", "Vercel", "code", siVercel),
  tool("netlify", "Netlify", "code", siNetlify),
  tool("cloudflare", "Cloudflare", "code", siCloudflare),
  tool("supabase", "Supabase", "code", siSupabase),
  tool("neon", "Neon", "code", siNeon),
  tool("mongodb", "MongoDB", "code", siMongodb),
  tool("sentry", "Sentry", "code", siSentry),
  tool("datadog", "Datadog", "code", siDatadog),
  tool("pagerduty", "PagerDuty", "code", siPagerduty),
  tool("grafana", "Grafana", "code", siGrafana),
  tool("postman", "Postman", "code", siPostman),
  tool("hugging_face", "Hugging Face", "code", siHuggingface),
  tool("bigquery", "Google Cloud BigQuery", "data", siGooglebigquery),
  tool("snowflake", "Snowflake", "data", siSnowflake),
  tool("databricks", "Databricks", "data", siDatabricks),
  initial("amplitude", "Amplitude", "data", "#1E61F0"),
  tool("mixpanel", "Mixpanel", "data", siMixpanel),
  tool("posthog", "PostHog", "data", siPosthog),
  tool("hubspot", "HubSpot", "sales", siHubspot),
  initial("salesforce", "Salesforce", "sales", "#00A1E0"),
  tool("mailchimp", "Intuit Mailchimp", "sales", siMailchimp),
  tool("stripe", "Stripe", "money", siStripe),
  tool("paypal", "PayPal", "money", siPaypal),
  tool("square", "Square", "money", siSquare),
  tool("quickbooks", "Intuit QuickBooks", "money", siQuickbooks),
  tool("xero", "Xero", "money", siXero),
  tool("zapier", "Zapier", "automation", siZapier),
  tool("make", "Make", "automation", siMake),
  tool("n8n", "n8n", "automation", siN8n),
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

/** How a color reads on white (WCAG contrast, 1 to 21). */
export function contrastOnWhite(color: string): number {
  const channel = (at: number) => {
    const value = Number.parseInt(color.slice(at, at + 2), 16) / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  };
  const light = 0.2126 * channel(1) + 0.7152 * channel(3) + 0.0722 * channel(5);
  return 1.05 / (light + 0.05);
}
