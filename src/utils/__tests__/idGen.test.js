import { describe, expect, it } from "vitest";
import { randomName, randomSecret, CHARS } from "../idGen";

describe("idGen", () => {
  it("name uses mc-net prefix and two groups", () => {
    const name = randomName();
    expect(name).toMatch(/^mc-net-[0-9A-HJ-NP-Z]{4}-[0-9A-HJ-NP-Z]{4}$/);
  });

  it("secret is one dashed group", () => {
    expect(randomSecret()).toMatch(/^[0-9A-HJ-NP-Z]{4}-[0-9A-HJ-NP-Z]{4}$/);
  });

  it("charset excludes I and O", () => {
    expect(CHARS).not.toContain("I");
    expect(CHARS).not.toContain("O");
    expect(CHARS.length).toBe(34);
  });
});
