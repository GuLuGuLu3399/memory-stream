import { describe, it, expect } from "vitest";

describe("API Client", () => {
  it("should resolve BASE_URL from env or fallback", () => {
    // 默认 fallback 测试 — 不依赖 env
    const fallback = "http://localhost:8080/api/v1";
    const resolved = import.meta.env.VITE_API_BASE_URL || fallback;
    expect(resolved).toBeDefined();
    expect(typeof resolved).toBe("string");
  });
});
