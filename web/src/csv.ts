/** Read only the first CSV record for mapping controls. Rust validates the actual file. */
export function csvHeaders(csv: string): string[] {
  if (!csv) return [];
  const fields: string[] = [];
  let field = "";
  let quoted = false;
  for (let i = 0; i < csv.length; i++) {
    const char = csv[i];
    if (char === '"') {
      if (quoted && csv[i + 1] === '"') {
        field += '"';
        i++;
      } else quoted = !quoted;
    } else if (!quoted && char === ",") {
      fields.push(field);
      field = "";
    } else if (!quoted && (char === "\n" || char === "\r")) {
      fields.push(field);
      return fields;
    } else field += char;
  }
  fields.push(field);
  return fields;
}
