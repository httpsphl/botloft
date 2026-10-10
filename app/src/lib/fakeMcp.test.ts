import { describe, expect, it } from "vitest";
import { FakeBotloft } from "./fake";

describe("the fake's connected tools", () => {
  it("registers a tool, attaches it to an agent and lets it go with the tool", async () => {
    const fake = new FakeBotloft();
    const seen: string[] = [];
    fake.subscribe((event) => seen.push(event.name));
    const bot = fake.addBot(fake.addCrew("Ops").id, "SDR");

    const { servers } = await fake.call("mcp.save", {
      serverId: null,
      name: "LinkedIn",
      kind: "stdio",
      url: null,
      command: "uvx",
      args: ["linkedin-mcp-server"],
      headers: {},
      env: { LI_COOKIE: "secret-value" },
      description: "",
    });
    expect(servers[0]?.slug).toBe("linkedin");
    expect(JSON.stringify(servers)).not.toContain("secret-value");

    const id = servers[0]?.id as string;
    await fake.call("bot.mcp.set", { botId: bot.id, serverIds: [id] });
    expect((await fake.call("mcp.servers")).bots).toHaveLength(1);
    expect(seen).toContain("bot.mcp");

    const after = await fake.call("mcp.delete", { serverId: id });
    expect(after.servers).toHaveLength(0);
    expect(after.bots).toHaveLength(0);
  });

  it("refuses the name of Botloft's own tools", async () => {
    const fake = new FakeBotloft();
    await expect(
      fake.call("mcp.save", {
        serverId: null,
        name: "Botloft",
        kind: "http",
        url: "http://127.0.0.1:1/mcp",
        command: null,
        args: [],
        headers: {},
        env: {},
        description: "",
      }),
    ).rejects.toThrow();
  });
});
