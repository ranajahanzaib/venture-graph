import Graph from "graphology";
import { useEffect, useRef } from "react";
import Sigma from "sigma";

type Investor = { id: string; name: string; investor_type: string };
type Edge = { source: string; target: string; weight: number };
type Props = {
  data?: { nodes: Investor[]; edges: Edge[] };
  selected: Investor | null;
  onSelect: (investor: Investor | null) => void;
};

export default function GraphView({ data, selected, onSelect }: Props) {
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!ref.current || !data?.nodes.length) return;

    const graph = new Graph({ type: "undirected", multi: false });
    const degrees = new Map<string, number>();
    for (const edge of data.edges) {
      degrees.set(edge.source, (degrees.get(edge.source) ?? 0) + 1);
      degrees.set(edge.target, (degrees.get(edge.target) ?? 0) + 1);
    }

    data.nodes.forEach((node, index) => {
      const angle = index * 2.4;
      const radius = 0.3 + (index % 13) / 13;
      graph.addNode(node.id, {
        label: node.name,
        x: Math.cos(angle) * radius,
        y: Math.sin(angle) * radius,
        size: Math.min(8, 3 + Math.log2((degrees.get(node.id) ?? 0) + 1)),
        color: node.id === selected?.id ? "#e7cf76" : "#a6d77b",
      });
    });

    data.edges.forEach((edge) => {
      if (!graph.hasNode(edge.source) || !graph.hasNode(edge.target)) return;
      if (graph.hasEdge(edge.source, edge.target)) return;
      graph.addEdge(edge.source, edge.target, {
        size: Math.max(0.6, Math.min(2.8, Math.log2(edge.weight + 1) * 0.45)),
        color: "#465848",
      });
    });

    const renderer = new Sigma(graph, ref.current, {
      styles: {
        nodes: {
          x: { attribute: "x" },
          y: { attribute: "y" },
          size: { attribute: "size", defaultValue: 5 },
          color: { attribute: "color", defaultValue: "#a6d77b" },
          label: { attribute: "label" },
          labelColor: "#d9e2d5",
          labelVisibility: {
            attribute: "labelVisibility",
            defaultValue: "auto",
          },
          labelSize: 11,
        },
        edges: {
          size: { attribute: "size", defaultValue: 1 },
          color: { attribute: "color", defaultValue: "#465848" },
        },
      },
      settings: {
        itemSizesReference: "screen",
        stagePadding: 35,
        labelRenderedSizeThreshold: 3,
      },
    });
    renderer.on("clickNode", ({ node }) => {
      const investor = data.nodes.find((item) => item.id === node);
      if (investor) onSelect(investor);
    });
    renderer.on("clickStage", () => onSelect(null));
    return () => renderer.kill();
  }, [data, onSelect, selected]);

  return (
    <div ref={ref} className="graph-view">
      {selected && (
        <div className="selected-pill">
          <i />
          {selected.name}
          <button onClick={() => onSelect(null)}>×</button>
        </div>
      )}
    </div>
  );
}
