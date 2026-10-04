// A dialog leaves the page at once when its owner closes it, whatever
// closed it (its X, Escape, Cancel, Save). So it does not just vanish, a
// still copy of it stays a moment and fades out (dialog.css, spec 15.3).

/** Longest the copy stays, if its animation never reports its end. */
const EXIT_MS = 400;

/**
 * Leaves a fading copy of `node`, a dialog's backdrop, in its place, once
 * it is out of the page. One still in it was only mounted again (React's
 * strict mode in dev).
 */
export function exitDialog(node: HTMLElement | null): void {
  // Where nothing animates (tests), there is nothing to wait for.
  if (!node || typeof node.getAnimations !== "function") {
    return;
  }
  queueMicrotask(() => {
    if (!node.isConnected) {
      leave(node);
    }
  });
}

function leave(node: HTMLElement): void {
  const copy = node.cloneNode(true) as HTMLElement;
  // What the owner typed lives in the fields, not in their markup.
  const fields = node.querySelectorAll<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>(
    "input, textarea, select",
  );
  const copies = copy.querySelectorAll<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>(
    "input, textarea, select",
  );
  fields.forEach((field, index) => {
    const twin = copies[index];
    if (twin) {
      twin.value = field.value;
    }
  });
  copy.removeAttribute("id");
  copy.setAttribute("aria-hidden", "true");
  copy.inert = true;
  copy.classList.add("dialog-exit");
  document.body.append(copy);
  const remove = () => copy.remove();
  copy.addEventListener("animationend", (event) => event.target === copy && remove());
  setTimeout(remove, EXIT_MS);
}
