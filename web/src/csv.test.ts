import { expect, it } from "vitest";
import { csvHeaders } from "./csv";
it("preserves quoted commas, escaped quotes and multiline header names", () => {
  expect(csvHeaders('t,"body,a","say ""b""","multi\nline"\r\n0,1,2,3')).toEqual(
    ["t", "body,a", 'say "b"', "multi\nline"],
  );
  expect(csvHeaders("")).toEqual([]);
});
