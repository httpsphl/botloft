// Following the end of a bot's chat (spec 15.3): what arrives comes into
// view for who reads the latest. A reply taller than the chat is the
// exception: following stops at its start, so the owner reads it from the
// top and it grows below, instead of landing on its last line. Once the
// owner reads past its start, following goes back to the end.

/** Room left above a long reply: the bot's name and the line before it. */
const ABOVE_PX = 80;

/** Where to scroll to follow the chat; `list` holds its rows. */
export function followTop(scroller: HTMLElement, list: HTMLElement | null): number {
  const end = scroller.scrollHeight;
  // The newest reply, if the newest row is the bot's: after a message from
  // the owner, the end is what they look for.
  const replies = list?.lastElementChild?.querySelectorAll("[data-reply]");
  const reply = replies?.[replies.length - 1];
  if (!reply || scroller.clientHeight === 0) {
    return end;
  }
  const box = reply.getBoundingClientRect();
  const view = scroller.getBoundingClientRect();
  // Boxes come zoomed under a CSS zoom (the dev preview's), scrolling does not.
  const scale = view.height / scroller.clientHeight || 1;
  const height = box.height / scale;
  const top = (box.top - view.top) / scale;
  if (height <= scroller.clientHeight - ABOVE_PX || top < 0) {
    return end;
  }
  return Math.min(end, scroller.scrollTop + top - ABOVE_PX);
}
