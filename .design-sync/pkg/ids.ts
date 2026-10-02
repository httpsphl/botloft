// Ids and times for the sample data: unique per page, prefixed like the
// daemon's (`bot_`, `msg_`...), and times counted back from now.

let counter = 0;

export const id = (prefix: string) => `${prefix}_${String(++counter).padStart(6, "0")}`;

export const ago = (minutes: number) => Date.now() - minutes * 60_000;
