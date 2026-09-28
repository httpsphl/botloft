// Rules the fake daemon shares between its handlers: errors, slugs and
// names, close enough to the daemon's for the UI.

import { FIELD_LIMITS, RpcErrorCode } from "./protocol.gen";
import { RpcError } from "./rpc";

export const invalid = (message: string) => new RpcError(RpcErrorCode.validation, message);
export const notFound = (what: string) => new RpcError(RpcErrorCode.notFound, `${what} not found`);
export const conflict = (message: string) => new RpcError(RpcErrorCode.conflict, message);

/** ASCII, lowercase, hyphens; close enough to the daemon's slugs. */
export function slugify(name: string, fallback: string): string {
  const slug = name
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 32);
  return slug || fallback;
}

/** A crew or bot name, trimmed and checked like the daemon does. */
export function checkName(value: string): string {
  const name = value.trim();
  if (!name) {
    throw invalid("name must not be empty");
  }
  if (name.length > FIELD_LIMITS.name) {
    throw invalid(`name must be at most ${FIELD_LIMITS.name} characters`);
  }
  return name;
}
