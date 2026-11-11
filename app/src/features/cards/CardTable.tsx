// A table in an agent's card (spec 8.11), with the chat's table look.
// Cells are text as the agent wrote them; numbers line up on the right.

const NUMBER = /^[-+]?[\d.,\s]*\d[\d.,\s]*%?$/;

export function CardTable({ columns, rows }: { columns: string[]; rows: string[][] }) {
  // A column of numbers only lines up on the right.
  const numeric = columns.map(
    (_, at) => rows.length > 0 && rows.every((row) => !row[at] || NUMBER.test(row[at] ?? "")),
  );
  return (
    <div className="chat-md">
      <table>
        <thead>
          <tr>
            {columns.map((column, at) => (
              <th key={column} style={numeric[at] ? { textAlign: "right" } : undefined}>
                {column}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, line) => (
            // biome-ignore lint/suspicious/noArrayIndexKey: rows have no ids and never move
            <tr key={line}>
              {row.map((cell, at) => (
                <td
                  // biome-ignore lint/suspicious/noArrayIndexKey: cells have no ids
                  key={at}
                  style={numeric[at] ? { textAlign: "right" } : undefined}
                  className={numeric[at] ? "tabular-nums" : undefined}
                >
                  {cell}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
