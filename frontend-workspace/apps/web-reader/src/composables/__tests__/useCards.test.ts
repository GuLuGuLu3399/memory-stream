// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach } from "vitest";
import { nextTick } from "vue";
import { useCards } from "../useCards";
import { api } from "../../api";
import type { CardWithRelations } from "../../api";

vi.mock("../../api", () => ({
  api: {
    listCards: vi.fn(),
    getCard: vi.fn(),
    getGraph: vi.fn(),
  },
}));

vi.mock("../../api/schemas", () => ({
  CardListResponseSchema: { parse: vi.fn((data: any) => data) },
  CardDetailResponseSchema: { parse: vi.fn((data: any) => data) },
  GraphResultSchema: { parse: vi.fn((data: any) => data) },
}));

vi.mock("../useCardCache", () => ({
  getCached: vi.fn(),
  setCache: vi.fn(),
  clearCache: vi.fn(),
}));

vi.mock("wasm-engine", () => ({
  default: vi.fn(() => Promise.resolve()),
  render_from_ast: vi.fn((astJson: string) => `<div>${astJson}</div>`),
  process_markdown: vi.fn((rawMd: string) => ({
    html: `<p>${rawMd}</p>`,
    ast_json: "{}",
  })),
}));

import { getCached, setCache } from "../useCardCache";

describe("useCards", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(getCached).mockReturnValue(null);
  });

  describe("loadIndex", () => {
    it("should load card index successfully", async () => {
      const mockData = {
        data: [
          {
            id: "card-1",
            title: "Card 1",
            raw_md: "Content",
            excerpt: "Excerpt 1",
            category_id: null,
            created_at: "2024-01-01",
            updated_at: "2024-01-02",
            metrics: null,
            ast_data: null,
          },
          {
            id: "card-2",
            title: "Card 2",
            raw_md: "Content",
            excerpt: "Excerpt 2",
            category_id: null,
            created_at: "2024-01-01",
            updated_at: "2024-01-02",
            metrics: {
              card_id: "card-2",
              view_count: 10,
              hot_score: 5,
              updated_at: "",
            },
            ast_data: null,
          },
        ],
        has_more: false,
        total_count: 2,
      };

      vi.mocked(api.listCards).mockResolvedValue(mockData);
      vi.mocked(api.getGraph).mockResolvedValue({ nodes: [], edges: [] });

      const { cardIndex, loading, error, loadIndex } = useCards();
      const promise = loadIndex();
      expect(loading.value).toBe(true);
      await promise;
      await nextTick();

      expect(loading.value).toBe(false);
      expect(error.value).toBe("");
      expect(cardIndex.value).toHaveLength(2);
      expect(cardIndex.value[0].title).toBe("Card 1");
      expect(cardIndex.value[1].hot_score).toBe(5);
    });

    it("should handle API errors gracefully", async () => {
      vi.mocked(api.listCards).mockRejectedValue(new Error("Network error"));

      const { cardIndex, error, loadIndex } = useCards();
      await loadIndex();
      await nextTick();

      expect(error.value).toBe("Network error");
      expect(cardIndex.value).toEqual([]);
    });
  });

  describe("loadDetail", () => {
    it("should return cached detail if available", async () => {
      const cached = {
        id: "card-1",
        title: "Cached",
        html: "<p>Cached</p>",
        rawMd: "Cached",
        updatedAt: "2024-01-01",
        tocData: null,
      };
      vi.mocked(getCached).mockReturnValue(cached);

      const { loadDetail } = useCards();
      const result = await loadDetail("card-1");

      expect(result).toEqual(cached);
      expect(api.getCard).not.toHaveBeenCalled();
    });

    it("should fetch and process card detail", async () => {
      const mockDetail: CardWithRelations = {
        id: "card-1",
        title: "Card 1",
        raw_md: "# Test",
        excerpt: "",
        ast_data: { children: [{ type: "Paragraph" }] },
        toc_data: [{ level: 1, text: "Test", slug: "test", children: [] }],
        category_id: null,
        created_at: "2024-01-01",
        updated_at: "2024-01-02",
        metrics: null,
      };

      vi.mocked(api.getCard).mockResolvedValue(mockDetail);

      const { loadDetail } = useCards();
      const result = await loadDetail("card-1");

      expect(result).not.toBeNull();
      expect(result!.id).toBe("card-1");
      expect(result!.title).toBe("Card 1");
      expect(result!.tocData).toEqual(mockDetail.toc_data);
      expect(setCache).toHaveBeenCalledWith("card-1", expect.anything());
    });

    it("should handle API errors and return null", async () => {
      vi.mocked(api.getCard).mockRejectedValue(new Error("Not found"));

      const { loadDetail } = useCards();
      const result = await loadDetail("invalid-id");

      expect(result).toBeNull();
    });
  });
});
