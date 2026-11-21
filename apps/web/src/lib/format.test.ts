import { describe, expect, it } from "vitest";
import { formatCount } from "./format";

describe("formatCount", () => {
  it("shows missing counts explicitly", () => expect(formatCount()).toBe("—"));
  it("formats available counts", () => expect(formatCount(1200)).toBe("1,200"));
});
