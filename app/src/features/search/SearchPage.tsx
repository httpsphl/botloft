// Searching the chats (spec 8.8): a field, the crew to look in, and what
// was found as the owner types, newest first. A result opens its bot's
// chat at that point.

import { LoaderCircle, Search } from "lucide-react";
import { useEffect, useId, useRef, useState } from "react";
import { useShallow } from "zustand/react/shallow";
import { useT } from "../../i18n";
import { errorText } from "../../lib/api";
import type { CrewId, SearchHit } from "../../lib/protocol.gen";
import { crewList } from "../../store/app";
import { useApi, useApp } from "../../store/context";
import { Button } from "../../ui/Button";
import { Callout } from "../../ui/Callout";
import { Select } from "../../ui/Select";
import { SearchResult } from "./SearchResult";

const PAGE = 30;
/** Quiet time after a key before searching. */
const TYPING_MS = 250;

/** What the owner searched, kept while the app is open. */
const kept = { query: "", crew: "" as CrewId | "" };

interface Found {
  hits: SearchHit[];
  /** More may be older than the last one. */
  more: boolean;
  loading: boolean;
  error: string | null;
}

const NONE: Found = { hits: [], more: false, loading: false, error: null };

/** Letters and digits in the query: fewer than 2 are not searched. */
const enough = (query: string) => query.replace(/[^\p{L}\p{N}]/gu, "").length >= 2;

export function SearchPage() {
  const t = useT();
  const words = t.search;
  const api = useApi();
  const crews = useApp(useShallow(crewList));
  const [query, setQuery] = useState(kept.query);
  const [crew, setCrew] = useState<CrewId | "">(
    kept.crew && crews.some((c) => c.id === kept.crew) ? kept.crew : "",
  );
  const [found, setFound] = useState<Found>(NONE);
  /** Which search the shown results are for; older answers are dropped. */
  const latest = useRef(0);
  const fieldId = useId();

  kept.query = query;
  kept.crew = crew;
  const trimmed = query.trim();

  useEffect(() => {
    const ask = ++latest.current;
    if (!enough(trimmed)) {
      setFound(NONE);
      return;
    }
    setFound((current) => ({ ...current, loading: true, error: null }));
    const timer = setTimeout(() => {
      api
        .call("chat.search", { query: trimmed, limit: PAGE, ...(crew ? { crewId: crew } : {}) })
        .then(
          (hits) =>
            ask === latest.current &&
            setFound({ hits, more: hits.length === PAGE, loading: false, error: null }),
          (error) => ask === latest.current && setFound({ ...NONE, error: errorText(error) }),
        );
    }, TYPING_MS);
    return () => clearTimeout(timer);
  }, [api, trimmed, crew]);

  const loadMore = () => {
    const before = found.hits.at(-1)?.item.id;
    if (!before) {
      return;
    }
    const ask = latest.current;
    setFound((current) => ({ ...current, loading: true }));
    api
      .call("chat.search", {
        query: trimmed,
        limit: PAGE,
        before,
        ...(crew ? { crewId: crew } : {}),
      })
      .then(
        (older) =>
          ask === latest.current &&
          setFound((current) => ({
            hits: [...current.hits, ...older],
            more: older.length === PAGE,
            loading: false,
            error: null,
          })),
        (error) =>
          ask === latest.current &&
          setFound((current) => ({ ...current, loading: false, error: errorText(error) })),
      );
  };

  const scopes = [
    { value: "" as CrewId | "", label: words.allCrews },
    ...crews.map((c) => ({ value: c.id as CrewId | "", label: c.name })),
  ];
  const searched = enough(trimmed);

  return (
    <section aria-label={words.label} className="flex min-h-0 flex-1 flex-col">
      <header className="flex flex-wrap items-center gap-3 border-line border-b px-5 py-3.5">
        <label htmlFor={fieldId} className="sr-only">
          {words.field}
        </label>
        <div className="flex min-w-60 flex-1 items-center gap-2 rounded-xl border border-line-strong bg-panel px-3 focus-within:border-muted">
          <Search aria-hidden size={16} className="shrink-0 text-muted" />
          <input
            id={fieldId}
            type="search"
            // biome-ignore lint/a11y/noAutofocus: opening the search is asking to type
            autoFocus
            value={query}
            placeholder={words.placeholder}
            onChange={(event) => setQuery(event.target.value)}
            // What was searched before comes back selected, ready to replace.
            onFocus={(event) => event.currentTarget.select()}
            className="h-10 min-w-0 flex-1 bg-transparent text-base outline-none placeholder:text-muted"
          />
          {found.loading && (
            <LoaderCircle
              role="status"
              aria-label={words.searching}
              size={15}
              className="shrink-0 animate-spin text-muted"
            />
          )}
        </div>
        <div className="w-48">
          <Select label={words.scope} value={crew} options={scopes} onChange={setCrew} />
        </div>
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto p-5">
        <div className="mx-auto flex max-w-3xl flex-col gap-3">
          {found.error && (
            <Callout tone="danger" title={words.failed}>
              {found.error}
            </Callout>
          )}
          {!searched && (
            <p className="text-muted text-sm">{trimmed ? words.tooShort : words.hint}</p>
          )}
          {searched && !found.loading && !found.error && found.hits.length === 0 && (
            <p className="text-muted text-sm">{words.nothing(trimmed)}</p>
          )}
          {searched && found.hits.length > 0 && (
            <ol aria-label={words.results} className="flex flex-col gap-1">
              {found.hits.map((hit) => (
                <SearchResult key={hit.item.id} hit={hit} />
              ))}
            </ol>
          )}
          {searched && found.more && (
            <div className="flex justify-center">
              <Button size="sm" disabled={found.loading} onClick={loadMore}>
                {words.more}
              </Button>
            </div>
          )}
        </div>
      </div>
    </section>
  );
}
