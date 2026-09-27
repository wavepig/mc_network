import { describe, expect, it } from "vitest";
import { formatLinkInfo, parseLinkInfo } from "../useClipboard";

describe("useClipboard", () => {
  it("formats single-line link info with port", () => {
    expect(formatLinkInfo("mc-net-A1B2-C3D4", "E5F6-G7H8", 25565)).toBe(
      "mc-net-A1B2-C3D4 E5F6-G7H8 25565"
    );
  });

  it("parses space separated trio", () => {
    expect(parseLinkInfo("mc-net-A1B2-C3D4 E5F6-G7H8 25565")).toEqual({
      name: "mc-net-A1B2-C3D4",
      secret: "E5F6-G7H8",
      port: 25565,
    });
  });

  it("parses two-part input without port", () => {
    expect(parseLinkInfo("mc-net-A1B2-C3D4 E5F6-G7H8")).toEqual({
      name: "mc-net-A1B2-C3D4",
      secret: "E5F6-G7H8",
    });
  });

  it("parses comma and slash separators", () => {
    expect(parseLinkInfo("mc-net-A1B2-C3D4,E5F6-G7H8").secret).toBe("E5F6-G7H8");
    expect(parseLinkInfo("mc-net-A1B2-C3D4/E5F6-G7H8").name).toBe("mc-net-A1B2-C3D4");
  });

  it("ignores invalid port in third segment", () => {
    expect(parseLinkInfo("mc-net-A1B2-C3D4 E5F6-G7H8 abc").port).toBeUndefined();
    expect(parseLinkInfo("mc-net-A1B2-C3D4 E5F6-G7H8 0").port).toBeUndefined();
    expect(parseLinkInfo("mc-net-A1B2-C3D4 E5F6-G7H8 99999").port).toBeUndefined();
  });

  it("returns null for unparseable text", () => {
    expect(parseLinkInfo("hello")).toBeNull();
    expect(parseLinkInfo("")).toBeNull();
  });
});
