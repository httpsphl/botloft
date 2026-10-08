import { describe, expect, it } from "vitest";
import { FakeBotloft } from "./fake";
import { openFakeCloudLink } from "./fakeCloud";
import { joinFakePhone } from "./fakeMobile";

describe("the fake's phones", () => {
  it("needs the account, shows the code a phone brings, connects it and cuts it off", async () => {
    const fake = new FakeBotloft();
    const seen: string[] = [];
    fake.subscribe((event) => seen.push(event.name));

    await expect(fake.call("mobile.pair_start")).rejects.toMatchObject({ reason: "not_signed_in" });
    await fake.call("cloud.signin", { email: "ana@exemplo.com" });
    openFakeCloudLink(fake);

    const started = await fake.call("mobile.pair_start");
    expect(started.url).toContain("#p=");
    expect((await fake.call("mobile.status")).pending?.pairId).toBe(started.pairId);
    await expect(
      fake.call("mobile.pair_confirm", { pairId: started.pairId, accept: true }),
    ).rejects.toMatchObject({ reason: "conflict" });

    joinFakePhone(fake, "Celular da Ana", "482913");
    expect(seen).toContain("mobile.pair_request");
    expect((await fake.call("mobile.status")).pending?.joined).toEqual({
      name: "Celular da Ana",
      code: "482913",
    });
    await fake.call("mobile.pair_confirm", { pairId: started.pairId, accept: true });
    const status = await fake.call("mobile.status");
    expect(status.pending).toBeUndefined();
    expect(status.phones).toHaveLength(1);
    expect(status.phones[0]?.name).toBe("Celular da Ana");

    const gone = await fake.call("mobile.revoke", { phoneId: status.phones[0]?.id ?? "" });
    expect(gone.phones).toEqual([]);
    await expect(fake.call("mobile.revoke", { phoneId: "dev_none" })).rejects.toMatchObject({
      reason: "not_found",
    });
  });

  it("is cleared when the account is left", async () => {
    const fake = new FakeBotloft();
    await fake.call("cloud.signin", { email: "ana@exemplo.com" });
    openFakeCloudLink(fake);
    const started = await fake.call("mobile.pair_start");
    joinFakePhone(fake);
    await fake.call("mobile.pair_confirm", { pairId: started.pairId, accept: true });
    await fake.call("cloud.signout");
    expect((await fake.call("mobile.status")).phones).toEqual([]);
  });
});
