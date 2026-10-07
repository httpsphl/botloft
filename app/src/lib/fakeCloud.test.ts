import { describe, expect, it } from "vitest";
import { FakeBotloft } from "./fake";
import { expireFakeCloudLink, openFakeCloudLink } from "./fakeCloud";

describe("the fake's account and cloud copies", () => {
  it("signs in when the link is opened, keeps copies and signs out", async () => {
    const fake = new FakeBotloft();
    const seen: string[] = [];
    fake.subscribe((event) => seen.push(event.name));

    expect((await fake.call("cloud.status")).signedIn).toBe(false);
    await expect(fake.call("cloud.copies")).rejects.toMatchObject({ reason: "not_signed_in" });
    await expect(fake.call("cloud.signin", { email: "nobody" })).rejects.toMatchObject({
      reason: "bad_email",
    });

    expect((await fake.call("cloud.signin", { email: "Ana@Exemplo.com" })).wait).toBe(600);
    expect((await fake.call("cloud.status")).pending).toBe(true);
    openFakeCloudLink(fake);
    const status = await fake.call("cloud.status");
    expect(status).toMatchObject({ signedIn: true, email: "ana@exemplo.com", pending: false });
    expect(seen).toContain("cloud.signed_in");

    await expect(fake.call("cloud.upload", { passphrase: "short" })).rejects.toMatchObject({
      reason: "short_passphrase",
    });
    const copy = await fake.call("cloud.upload", { passphrase: "correct horse" });
    expect((await fake.call("cloud.copies")).copies).toEqual([copy]);
    expect((await fake.call("cloud.status")).used).toBe(copy.size);
    expect(seen).toContain("cloud.progress");
    expect((await fake.call("cloud.download", { id: copy.id })).path).toContain("cloud-download");

    await fake.call("cloud.delete", { id: copy.id });
    expect((await fake.call("cloud.copies")).copies).toHaveLength(0);
    await fake.call("cloud.signout");
    expect((await fake.call("cloud.status")).signedIn).toBe(false);
  });

  it("keeps only the newest five copies, and a link can run out", async () => {
    const fake = new FakeBotloft();
    await fake.call("cloud.signin", { email: "ana@exemplo.com" });
    expireFakeCloudLink(fake);
    expect((await fake.call("cloud.status")).pending).toBe(false);

    await fake.call("cloud.signin", { email: "ana@exemplo.com" });
    openFakeCloudLink(fake);
    for (let n = 0; n < 6; n++) {
      await fake.call("cloud.upload", { passphrase: "correct horse" });
    }
    const { copies } = await fake.call("cloud.copies");
    expect(copies).toHaveLength(5);
    expect(copies[0]?.id).toBe("cpy_fake6");
  });
});
