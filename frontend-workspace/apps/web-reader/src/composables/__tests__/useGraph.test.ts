// @vitest-environment jsdom
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { nextTick } from "vue";
import { useGraph } from "../useGraph";
import { api } from "../../api";

// Mock the API module
vi.mock("../../api", () => ({
  api: {
    getFullGraph: vi.fn(),
    getGraph: vi.fn(),
  },
}));

// Mock the schemas module
vi.mock("../../api/schemas", () => ({
  GraphResultSchema: {
    parse: vi.fn((data) => data),
  },
}));

describe("useGraph", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  describe("initial state", () => {
    it("should initialize with empty nodes and edges", () => {
      const { nodes, edges, loading, error, isEmpty } = useGraph();

      expect(nodes.value).toEqual([]);
      expect(edges.value).toEqual([]);
      expect(loading.value).toBe(false);
      expect(error.value).toBe("");
      expect(isEmpty.value).toBe(false);
    });
  });

  describe("loadFullGraph", () => {
    it("should load full graph data successfully", async () => {
      const mockGraphData = {
        nodes: [
          { id: "node-1", title: "Card 1" },
          { id: "node-2", title: "Card 2" },
        ],
        edges: [{ source: "node-1", target: "node-2", relation: "sequence" }],
      };

      vi.mocked(api.getFullGraph).mockResolvedValue(mockGraphData);

      const { nodes, edges, loading, error, isEmpty, loadFullGraph } =
        useGraph();

      const promise = loadFullGraph();
      expect(loading.value).toBe(true);
      await promise;
      await nextTick();

      expect(loading.value).toBe(false);
      expect(error.value).toBe("");
      expect(isEmpty.value).toBe(false);
      expect(nodes.value).toHaveLength(2);
      expect(edges.value).toHaveLength(1);
    });

    it("should set isEmpty to true when no nodes returned", async () => {
      vi.mocked(api.getFullGraph).mockResolvedValue({ nodes: [], edges: [] });

      const { nodes, edges, isEmpty, loadFullGraph } = useGraph();

      await loadFullGraph();
      await nextTick();

      expect(isEmpty.value).toBe(true);
      expect(nodes.value).toEqual([]);
      expect(edges.value).toEqual([]);
    });

    it("should handle API errors gracefully", async () => {
      vi.mocked(api.getFullGraph).mockRejectedValue(new Error("Network error"));

      const { nodes, edges, loading, error, loadFullGraph } = useGraph();

      await loadFullGraph();
      await nextTick();

      expect(loading.value).toBe(false);
      expect(error.value).toBe("Network error");
      expect(nodes.value).toEqual([]);
      expect(edges.value).toEqual([]);
    });

    it("should mark orphan nodes when no edges connect them", async () => {
      const mockGraphData = {
        nodes: [
          { id: "orphan-node", title: "Orphan Card" },
          { id: "connected-node-1", title: "Connected 1" },
          { id: "connected-node-2", title: "Connected 2" },
        ],
        edges: [
          {
            source: "connected-node-1",
            target: "connected-node-2",
            relation: "reference",
          },
        ],
      };

      vi.mocked(api.getFullGraph).mockResolvedValue(mockGraphData);

      const { nodes, loadFullGraph } = useGraph();

      await loadFullGraph();
      await nextTick();

      const orphanNode = nodes.value.find((n) => n.id === "orphan-node");
      const connectedNode1 = nodes.value.find(
        (n) => n.id === "connected-node-1",
      );

      expect(orphanNode?.data.isOrphan).toBe(true);
      expect(connectedNode1?.data.isOrphan).toBe(false);
    });

    it("should mark sequence nodes based on edge relation", async () => {
      const mockGraphData = {
        nodes: [
          { id: "seq-node-1", title: "Seq 1" },
          { id: "seq-node-2", title: "Seq 2" },
          { id: "ref-node", title: "Ref" },
        ],
        edges: [
          { source: "seq-node-1", target: "seq-node-2", relation: "sequence" },
          { source: "seq-node-2", target: "ref-node", relation: "reference" },
        ],
      };

      vi.mocked(api.getFullGraph).mockResolvedValue(mockGraphData);

      const { nodes, loadFullGraph } = useGraph();

      await loadFullGraph();
      await nextTick();

      const seqNode1 = nodes.value.find((n) => n.id === "seq-node-1");
      const seqNode2 = nodes.value.find((n) => n.id === "seq-node-2");

      expect(seqNode1?.data.type).toBe("sequence");
      expect(seqNode2?.data.type).toBe("sequence");
    });

    it("should convert edges with correct styling for sequence relation", async () => {
      const mockGraphData = {
        nodes: [
          { id: "node-1", title: "Card 1" },
          { id: "node-2", title: "Card 2" },
        ],
        edges: [{ source: "node-1", target: "node-2", relation: "sequence" }],
      };

      vi.mocked(api.getFullGraph).mockResolvedValue(mockGraphData);

      const { edges, loadFullGraph } = useGraph();

      await loadFullGraph();
      await nextTick();

      const edge = edges.value[0];
      expect(edge.animated).toBe(true);
      expect((edge.style as Record<string, unknown>)?.stroke).toBe("#00e5ff");
      expect((edge.style as Record<string, unknown>)?.strokeWidth).toBe(2);
    });

    it("should use default depth of 2", async () => {
      vi.mocked(api.getGraph).mockResolvedValue({ nodes: [], edges: [] });

      const { load } = useGraph();

      await load("some-card");

      expect(api.getGraph).toHaveBeenCalledWith("some-card", 2);
    });

    it("should set isEmpty when no nodes returned", async () => {
      vi.mocked(api.getGraph).mockResolvedValue({ nodes: [], edges: [] });

      const { isEmpty, load } = useGraph();

      await load("card-id");
      await nextTick();

      expect(isEmpty.value).toBe(true);
    });

    it("should handle errors during depth load", async () => {
      vi.mocked(api.getGraph).mockRejectedValue(new Error("Card not found"));

      const { error, load } = useGraph();

      await load("invalid-id");
      await nextTick();

      expect(error.value).toBe("Card not found");
    });

    it("should reset loading state on error", async () => {
      vi.mocked(api.getGraph).mockRejectedValue(new Error("Failed"));

      const { loading, load } = useGraph();

      await load("card-id");

      expect(loading.value).toBe(false);
    });
  });

  describe("node conversion", () => {
    it("should create nodes with correct Vue Flow structure", async () => {
      const mockGraphData = {
        nodes: [{ id: "test-node", title: "Test Card" }],
        edges: [],
      };

      vi.mocked(api.getFullGraph).mockResolvedValue(mockGraphData);

      const { nodes, loadFullGraph } = useGraph();

      await loadFullGraph();
      await nextTick();

      const node = nodes.value[0];
      expect(node.id).toBe("test-node");
      expect(node.type).toBe("card");
      expect(node.position).toEqual({ x: 0, y: 0 });
      expect(node.data.title).toBe("Test Card");
      expect(node.data.date).toBe("");
    });

    it("should create edges with correct Vue Flow structure", async () => {
      const mockGraphData = {
        nodes: [
          { id: "node-a", title: "A" },
          { id: "node-b", title: "B" },
        ],
        edges: [{ source: "node-a", target: "node-b", relation: "reference" }],
      };

      vi.mocked(api.getFullGraph).mockResolvedValue(mockGraphData);

      const { edges, loadFullGraph } = useGraph();

      await loadFullGraph();
      await nextTick();

      const edge = edges.value[0];
      expect(edge.id).toBe("e-node-a-node-b");
      expect(edge.source).toBe("node-a");
      expect(edge.target).toBe("node-b");
      expect(edge.type).toBe("default");
      expect(edge.data.type).toBe("reference");
    });
  });
});
