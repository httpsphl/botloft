// The claude.ai tools the app knows by name (spec 34.7): an icon and a color
// to draw each one, in our own drawing, and the key of its description.
// The "Discover" tab lists the ones the account does not have yet.

import {
  Box,
  CalendarDays,
  ChartNoAxesColumn,
  CheckSquare,
  CreditCard,
  Database,
  FileText,
  FolderOpen,
  GitBranch,
  Globe,
  Layers,
  LayoutGrid,
  ListChecks,
  type LucideIcon,
  Mail,
  MessageSquare,
  NotebookText,
  Palette,
  PenTool,
  Sheet,
  Triangle,
  Users,
  Workflow,
  Zap,
} from "lucide-react";

export type ToolKey =
  | "google_drive"
  | "gmail"
  | "google_calendar"
  | "google_docs"
  | "google_sheets"
  | "slack"
  | "notion"
  | "microsoft_365"
  | "canva"
  | "figma"
  | "atlassian"
  | "asana"
  | "linear"
  | "github"
  | "hubspot"
  | "clickup"
  | "vercel"
  | "supabase"
  | "stripe"
  | "wordpress"
  | "zapier"
  | "dropbox"
  | "monday";

export interface KnownTool {
  key: ToolKey;
  /** As claude.ai names it. */
  name: string;
  icon: LucideIcon;
  /** The tile's color; the icon is white on it. */
  color: string;
}

export const KNOWN_TOOLS: KnownTool[] = [
  { key: "google_calendar", name: "Google Calendar", icon: CalendarDays, color: "#1A73E8" },
  { key: "gmail", name: "Gmail", icon: Mail, color: "#D93025" },
  { key: "google_drive", name: "Google Drive", icon: FolderOpen, color: "#188038" },
  { key: "google_docs", name: "Google Docs", icon: FileText, color: "#4285F4" },
  { key: "google_sheets", name: "Google Sheets", icon: Sheet, color: "#0F9D58" },
  { key: "slack", name: "Slack", icon: MessageSquare, color: "#4A154B" },
  { key: "notion", name: "Notion", icon: NotebookText, color: "#37352F" },
  { key: "microsoft_365", name: "Microsoft 365", icon: LayoutGrid, color: "#D83B01" },
  { key: "canva", name: "Canva", icon: Palette, color: "#00A3B4" },
  { key: "figma", name: "Figma", icon: PenTool, color: "#A259FF" },
  { key: "atlassian", name: "Atlassian", icon: Layers, color: "#0052CC" },
  { key: "asana", name: "Asana", icon: ListChecks, color: "#E8615A" },
  { key: "linear", name: "Linear", icon: Workflow, color: "#5E6AD2" },
  { key: "github", name: "GitHub", icon: GitBranch, color: "#24292F" },
  { key: "hubspot", name: "HubSpot", icon: Users, color: "#F05C2E" },
  { key: "clickup", name: "ClickUp", icon: CheckSquare, color: "#7B68EE" },
  { key: "vercel", name: "Vercel", icon: Triangle, color: "#111111" },
  { key: "supabase", name: "Supabase", icon: Database, color: "#249361" },
  { key: "stripe", name: "Stripe", icon: CreditCard, color: "#635BFF" },
  { key: "wordpress", name: "WordPress.com", icon: Globe, color: "#21759B" },
  { key: "zapier", name: "Zapier", icon: Zap, color: "#FF4F00" },
  { key: "dropbox", name: "Dropbox", icon: Box, color: "#0061FF" },
  { key: "monday", name: "monday.com", icon: ChartNoAxesColumn, color: "#6161FF" },
];

/** A tool of the account by its name, when the app knows it. */
export function knownTool(name: string): KnownTool | undefined {
  const wanted = name.trim().toLowerCase();
  return KNOWN_TOOLS.find(
    (tool) =>
      tool.name.toLowerCase() === wanted ||
      // "Atlassian MCP", "Atlassian Rovo": the start is enough.
      wanted.startsWith(tool.name.toLowerCase()),
  );
}
