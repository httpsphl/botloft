// The conversations of an agent under its line in the sidebar (spec 40.7):
// an agent with more than one gets an arrow; open, it lists the main one
// and the others with the time of their last activity and the unread mark,
// and ends in "New conversation". Each other one has Rename and Archive.

import { ChevronRight, MessageSquarePlus } from "lucide-react";
import { type FormEvent, useCallback, useState } from "react";
import { createPortal } from "react-dom";
import { type Messages, useT } from "../../i18n";
import { when } from "../../lib/format";
import { type Bot, type Conversation, FIELD_LIMITS } from "../../lib/protocol.gen";
import { useApi, useApp } from "../../store/context";
import { openConversationOf } from "../../store/conversations";
import { isConversationUnread } from "../../store/seen";
import { Button } from "../../ui/Button";
import { Confirm } from "../../ui/Confirm";
import { ContextMenu, menuPoint, type Point } from "../../ui/ContextMenu";
import { Dialog } from "../../ui/Dialog";
import { TextField } from "../../ui/Field";
import { attempt } from "../../ui/toast";
import { row } from "./SidebarPages";

const NONE: Conversation[] = [];

/** What a conversation is called in the list and in the chat's header. */
export function conversationTitle(conversation: Conversation, t: Messages["conversations"]) {
  return conversation.main ? t.main : (conversation.title ?? t.untitled);
}

export function SidebarConversations({ bot }: { bot: Bot }) {
  const t = useT().conversations;
  const list = useApp((state) => state.conversations[bot.id] ?? NONE);
  const openOther = useApp(
    (state) => state.selectedBotId === bot.id && openConversationOf(state, bot.id) !== list[0]?.id,
  );
  const [shown, setShown] = useState(false);
  const newConversation = useApp((state) => state.newConversation);
  if (bot.agent !== "claude" || list.length < 2) {
    return null;
  }
  // Open on another conversation, the list shows where the chat is.
  const expanded = shown || openOther;
  return (
    <div className="pl-12">
      <button
        type="button"
        aria-expanded={expanded}
        onClick={() => setShown(!expanded)}
        className="flex items-center gap-1 rounded-md px-1 py-0.5 text-muted text-xs hover:text-ink"
      >
        <ChevronRight
          aria-hidden
          size={12}
          className={`transition-transform duration-150 ${expanded ? "rotate-90" : ""}`}
        />
        {t.count(list.length)}
      </button>
      {expanded && (
        <ul aria-label={t.listOf(bot.name)} className="flex flex-col gap-0.5 pt-0.5">
          {list.map((conversation) => (
            <ConversationLine key={conversation.id} bot={bot} conversation={conversation} />
          ))}
          <li>
            <button
              type="button"
              onClick={() => void attempt(t.newFailed, () => newConversation(bot.id))}
              className={`${row} gap-1.5 px-2 py-1 text-muted text-xs`}
            >
              <MessageSquarePlus aria-hidden size={12} />
              {t.new}
            </button>
          </li>
        </ul>
      )}
    </div>
  );
}

function ConversationLine({ bot, conversation }: { bot: Bot; conversation: Conversation }) {
  const t = useT().conversations;
  const open = useApp(
    (state) =>
      state.selectedBotId === bot.id && openConversationOf(state, bot.id) === conversation.id,
  );
  const unread = useApp((state) => isConversationUnread(state, conversation));
  const openConversation = useApp((state) => state.openConversation);
  const [menuAt, setMenuAt] = useState<Point | null>(null);
  const [asking, setAsking] = useState<"rename" | "archive" | null>(null);
  const closeMenu = useCallback(() => setMenuAt(null), []);
  const title = conversationTitle(conversation, t);
  return (
    <li>
      <button
        type="button"
        aria-current={open ? "page" : undefined}
        aria-label={[title, unread && t.unread].filter(Boolean).join(", ")}
        onClick={() => openConversation(bot.id, conversation.id)}
        onContextMenu={(event) => {
          if (!conversation.main) {
            event.preventDefault();
            setMenuAt(menuPoint(event));
          }
        }}
        className={`${row} gap-2 px-2 py-1 text-xs ${open || menuAt ? "bg-sunken" : ""}`}
      >
        <span className={`min-w-0 flex-1 truncate ${unread ? "font-semibold" : ""}`}>{title}</span>
        <time
          className="shrink-0 text-muted"
          dateTime={new Date(conversation.lastActivity).toISOString()}
        >
          {when(conversation.lastActivity)}
        </time>
        {unread && <span aria-hidden className="size-2 shrink-0 rounded-full bg-accent" />}
      </button>
      {menuAt && (
        <ContextMenu
          label={t.menuOf(title)}
          items={[
            { label: t.rename, onSelect: () => setAsking("rename") },
            { label: t.archive, onSelect: () => setAsking("archive") },
          ]}
          at={menuAt}
          onClose={closeMenu}
        />
      )}
      {asking === "rename" &&
        createPortal(
          <RenameDialog conversation={conversation} onClose={() => setAsking(null)} />,
          document.body,
        )}
      {asking === "archive" &&
        createPortal(
          <ArchiveConfirm
            conversation={conversation}
            title={title}
            onClose={() => setAsking(null)}
          />,
          document.body,
        )}
    </li>
  );
}

function RenameDialog({ conversation, onClose }: { conversation: Conversation; onClose(): void }) {
  const t = useT();
  const words = t.conversations;
  const api = useApi();
  const [title, setTitle] = useState(conversation.title ?? "");
  const [busy, setBusy] = useState(false);
  const submit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    const renamed = await attempt(words.renameFailed, () =>
      api.call("conversations.rename", { conversationId: conversation.id, title }),
    );
    setBusy(false);
    if (renamed) {
      onClose();
    }
  };
  return (
    <Dialog
      title={words.renameTitle}
      onClose={onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>
            {t.common.cancel}
          </Button>
          <Button
            variant="primary"
            type="submit"
            form="rename-conversation"
            disabled={busy || title.trim() === ""}
          >
            {words.save}
          </Button>
        </>
      }
    >
      <form id="rename-conversation" onSubmit={(event) => void submit(event)}>
        <TextField
          label={words.titleLabel}
          value={title}
          max={FIELD_LIMITS.name}
          maxLength={FIELD_LIMITS.name}
          autoFocus
          onChange={(event) => setTitle(event.target.value)}
        />
      </form>
    </Dialog>
  );
}

function ArchiveConfirm({
  conversation,
  title,
  onClose,
}: {
  conversation: Conversation;
  title: string;
  onClose(): void;
}) {
  const words = useT().conversations;
  const api = useApi();
  return (
    <Confirm
      title={words.archiveTitle(title)}
      confirmLabel={words.archive}
      onClose={onClose}
      onConfirm={() =>
        attempt(words.archiveFailed, () =>
          api.call("conversations.archive", { conversationId: conversation.id }),
        )
      }
    >
      {words.archiveBody}
    </Confirm>
  );
}
