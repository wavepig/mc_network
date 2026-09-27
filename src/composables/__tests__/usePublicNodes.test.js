// @vitest-environment happy-dom
import { beforeEach, describe, expect, it } from "vitest";
import { DEFAULT_NODES, usePublicNodes } from "../usePublicNodes";

describe("usePublicNodes", () => {
  beforeEach(() => localStorage.clear());

  it("starts with four default nodes", () => {
    const { nodes } = usePublicNodes();
    expect(nodes.value).toEqual(DEFAULT_NODES);
    expect(nodes.value).toHaveLength(4);
  });

  it("persists edits to localStorage", () => {
    const first = usePublicNodes();
    first.remove(0);
    const second = usePublicNodes();
    expect(second.nodes.value).toHaveLength(3);
  });

  it("restoreDefault brings back the four", () => {
    const { nodes, remove, restoreDefault } = usePublicNodes();
    remove(0);
    restoreDefault();
    expect(nodes.value).toEqual(DEFAULT_NODES);
  });

  it("add appends a node", () => {
    const { nodes, add } = usePublicNodes();
    add("tcp://example.com:1");
    expect(nodes.value.at(-1)).toBe("tcp://example.com:1");
  });
});
