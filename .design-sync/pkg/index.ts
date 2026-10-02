// The pieces of Botloft's interface that the design system shares with
// claude.ai/design: the app's own components, re-exported as they are, and
// its stylesheet (tokens, fonts and the Tailwind utilities the app uses).
// Built by build.mjs; nothing here reimplements a component. provider.tsx
// and scene.ts only wire the app's providers and build sample data.

import "./styles.css";

export { BOTLOFT_COLOR, BotAvatar, type Mood } from "../../app/src/features/bots/BotAvatar";
export { BotStateBadge } from "../../app/src/features/bots/BotStateBadge";
export { ChiefBadge } from "../../app/src/features/bots/ChiefBadge";
export { ApprovalCard } from "../../app/src/features/chat/ApprovalCard";
export { BotRun } from "../../app/src/features/chat/BotRun";
export { BrowserPanel } from "../../app/src/features/browser/BrowserPanel";
export { InboundRow } from "../../app/src/features/chat/InboundRow";
export { CrewBots } from "../../app/src/features/crews/CrewBots";
export { Sidebar } from "../../app/src/features/crews/Sidebar";
export { setLocaleChoice } from "../../app/src/i18n";
export { Button, type ButtonVariant } from "../../app/src/ui/Button";
export { Callout } from "../../app/src/ui/Callout";
export { Choices } from "../../app/src/ui/Choices";
export { Confirm } from "../../app/src/ui/Confirm";
export { Details } from "../../app/src/ui/Details";
export { Dialog } from "../../app/src/ui/Dialog";
export { SelectField, TextArea, TextField } from "../../app/src/ui/Field";
export { Menu, type MenuItem } from "../../app/src/ui/Menu";
export { Select } from "../../app/src/ui/Select";
export { SidePanel } from "../../app/src/ui/SidePanel";
export { Switch } from "../../app/src/ui/Switch";
export { type Tab, Tabs } from "../../app/src/ui/Tabs";
export { notifyError, Toaster } from "../../app/src/ui/toast";
export * as icons from "./icons.gen";
export { BotloftProvider, type BotloftProviderProps, type BrowserSeed } from "./provider";
export { AVATAR_PALETTE, chat, makeBot, makeCrew, makeMessage } from "./scene";
