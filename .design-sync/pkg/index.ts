// The pieces of Botloft's interface that the design system shares with
// claude.ai/design: the app's own components, re-exported as they are, and
// its stylesheet (tokens, fonts and the Tailwind utilities the app uses).
// Built by build.mjs; nothing here reimplements a component.

import "./styles.css";

export { BOTLOFT_COLOR, BotAvatar, ListAvatar, type Mood } from "../../app/src/features/bots/BotAvatar";
export { BotStateBadge } from "../../app/src/features/bots/BotStateBadge";
export { ChiefBadge } from "../../app/src/features/bots/ChiefBadge";
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
export { setLocaleChoice } from "../../app/src/i18n";
export * as icons from "./icons.gen";
