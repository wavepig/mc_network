import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd, args) => ({ cmd, args, mode: { mode: "idle" } })),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
}));

import { invoke } from "@tauri-apps/api/core";
import { startClient, startServer, stopNetwork } from "../useNetwork";

describe("useNetwork", () => {
  beforeEach(() => vi.clearAllMocks());

  it("startServer invokes create_server_network with publicNodes", async () => {
    await startServer({ name: "n", secret: "s", publicNodes: ["u"] });
    expect(invoke).toHaveBeenCalledWith("create_server_network", {
      name: "n",
      secret: "s",
      publicNodes: ["u"],
    });
  });

  it("startClient passes mcPort", async () => {
    await startClient({ name: "n", secret: "s", publicNodes: ["u"], mcPort: 25565 });
    expect(invoke).toHaveBeenCalledWith("create_client_network", {
      name: "n",
      secret: "s",
      publicNodes: ["u"],
      mcPort: 25565,
    });
  });

  it("stopNetwork invokes stop_network", async () => {
    await stopNetwork();
    expect(invoke).toHaveBeenCalledWith("stop_network", undefined);
  });
});
