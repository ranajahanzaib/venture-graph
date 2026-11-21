//! Graph algorithms over undirected, weighted investor networks.
//!
//! Functions are deterministic for a fixed node and edge insertion order. Weighted
//! degree uses the sum of shared-deal weights; path costs are deliberately separate
//! from strengths (`1 / weight` for strongest-route mode).

use petgraph::{
    algo::dijkstra,
    graph::{NodeIndex, UnGraph},
    visit::EdgeRef,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use uuid::Uuid;

pub type InvestorGraph = UnGraph<Uuid, f64>;

#[derive(Debug, Clone, PartialEq)]
pub struct InvestorMetrics {
    pub degree: usize,
    pub weighted_degree: f64,
    pub pagerank: f64,
    pub clustering_coefficient: f64,
    pub betweenness: f64,
}

pub fn degree(graph: &InvestorGraph, node: NodeIndex) -> usize {
    graph.neighbors(node).count()
}

pub fn weighted_degree(graph: &InvestorGraph, node: NodeIndex) -> f64 {
    graph.edges(node).map(|edge| edge.weight().max(0.0)).sum()
}

/// PageRank with weighted transitions and uniform redistribution for dangling nodes.
pub fn pagerank(
    graph: &InvestorGraph,
    damping: f64,
    iterations: usize,
    tolerance: f64,
) -> Vec<f64> {
    let count = graph.node_count();
    if count == 0 {
        return Vec::new();
    }
    let damping = damping.clamp(0.0, 1.0);
    let mut ranks = vec![1.0 / count as f64; count];
    for _ in 0..iterations {
        let dangling: f64 = graph
            .node_indices()
            .filter(|node| weighted_degree(graph, *node) <= 0.0)
            .map(|node| ranks[node.index()])
            .sum();
        let base = (1.0 - damping) / count as f64 + damping * dangling / count as f64;
        let mut next = vec![base; count];
        for node in graph.node_indices() {
            let total = weighted_degree(graph, node);
            if total <= 0.0 {
                continue;
            }
            for edge in graph.edges(node) {
                let other = edge.target();
                next[other.index()] +=
                    damping * ranks[node.index()] * edge.weight().max(0.0) / total;
            }
        }
        let delta: f64 = next.iter().zip(&ranks).map(|(a, b)| (a - b).abs()).sum();
        ranks = next;
        if delta <= tolerance {
            break;
        }
    }
    ranks
}

/// Exact unweighted betweenness centrality using Brandes' algorithm.
pub fn betweenness_centrality(graph: &InvestorGraph, normalized: bool) -> Vec<f64> {
    let count = graph.node_count();
    let mut centrality = vec![0.0; count];
    for source in graph.node_indices() {
        let dependency = brandes_source(graph, source);
        for (index, value) in dependency.into_iter().enumerate() {
            centrality[index] += value;
        }
    }
    // Undirected paths are counted from both endpoints.
    centrality.iter_mut().for_each(|value| *value /= 2.0);
    if normalized && count > 2 {
        let scale = 2.0 / ((count - 1) * (count - 2)) as f64;
        centrality.iter_mut().for_each(|value| *value *= scale);
    }
    centrality
}

/// Deterministic approximation of unweighted Brandes centrality using evenly
/// spaced source nodes. When `sample_count >= node_count`, this is exact.
pub fn sampled_betweenness_centrality(
    graph: &InvestorGraph,
    sample_count: usize,
    normalized: bool,
) -> Vec<f64> {
    let count = graph.node_count();
    if count == 0 || sample_count == 0 {
        return vec![0.0; count];
    }
    let sources = sample_count.min(count);
    if sources == count {
        return betweenness_centrality(graph, normalized);
    }
    let mut centrality = vec![0.0; count];
    for sample in 0..sources {
        let source_index = sample * count / sources;
        let dependency = brandes_source(graph, NodeIndex::new(source_index));
        for (index, value) in dependency.into_iter().enumerate() {
            centrality[index] += value;
        }
    }
    let scale = if normalized && count > 2 {
        count as f64 / sources as f64 / ((count - 1) * (count - 2)) as f64
    } else {
        count as f64 / sources as f64 / 2.0
    };
    centrality.iter_mut().for_each(|value| *value *= scale);
    centrality
}

fn brandes_source(graph: &InvestorGraph, source: NodeIndex) -> Vec<f64> {
    let count = graph.node_count();
    let mut stack = Vec::with_capacity(count);
    let mut predecessors: Vec<Vec<usize>> = vec![Vec::new(); count];
    let mut paths = vec![0.0; count];
    let mut distance = vec![-1_i64; count];
    paths[source.index()] = 1.0;
    distance[source.index()] = 0;
    let mut queue = VecDeque::from([source]);
    while let Some(node) = queue.pop_front() {
        stack.push(node);
        for neighbor in graph.neighbors(node) {
            if distance[neighbor.index()] < 0 {
                distance[neighbor.index()] = distance[node.index()] + 1;
                queue.push_back(neighbor);
            }
            if distance[neighbor.index()] == distance[node.index()] + 1 {
                paths[neighbor.index()] += paths[node.index()];
                predecessors[neighbor.index()].push(node.index());
            }
        }
    }
    let mut dependency = vec![0.0; count];
    while let Some(node) = stack.pop() {
        for predecessor in &predecessors[node.index()] {
            if paths[node.index()] > 0.0 {
                dependency[*predecessor] +=
                    paths[*predecessor] / paths[node.index()] * (1.0 + dependency[node.index()]);
            }
        }
    }
    dependency[source.index()] = 0.0;
    dependency
}

pub fn connected_components(graph: &InvestorGraph) -> Vec<Vec<Uuid>> {
    let mut seen = BTreeSet::new();
    let mut groups = Vec::new();
    for start in graph.node_indices() {
        if !seen.insert(start) {
            continue;
        }
        let mut queue = VecDeque::from([start]);
        let mut group = Vec::new();
        while let Some(node) = queue.pop_front() {
            group.push(graph[node]);
            for neighbor in graph.neighbors(node) {
                if seen.insert(neighbor) {
                    queue.push_back(neighbor);
                }
            }
        }
        group.sort();
        groups.push(group);
    }
    groups.sort_by(|a, b| a.first().cmp(&b.first()));
    groups
}

pub fn shortest_path(
    graph: &InvestorGraph,
    start: NodeIndex,
    end: NodeIndex,
) -> Option<Vec<NodeIndex>> {
    let distances = dijkstra(graph, start, Some(end), |_| 1.0_f64);
    reconstruct_path(
        graph,
        start,
        end,
        |node| distances.get(&node).copied(),
        |_| 1.0,
    )
}

/// Minimize summed reciprocal shared-deal weights.
pub fn strongest_connection_path(
    graph: &InvestorGraph,
    start: NodeIndex,
    end: NodeIndex,
) -> Option<Vec<NodeIndex>> {
    let distances = dijkstra(graph, start, Some(end), |edge| 1.0 / edge.weight().max(1.0));
    reconstruct_path(
        graph,
        start,
        end,
        |node| distances.get(&node).copied(),
        |weight| 1.0 / weight.max(1.0),
    )
}

fn reconstruct_path(
    graph: &InvestorGraph,
    start: NodeIndex,
    end: NodeIndex,
    distance: impl Fn(NodeIndex) -> Option<f64>,
    cost: impl Fn(f64) -> f64,
) -> Option<Vec<NodeIndex>> {
    distance(end)?;
    let mut current = end;
    let mut path = vec![end];
    while current != start {
        let current_cost = distance(current)?;
        let previous = graph.neighbors(current).find(|neighbor| {
            let Some(edge) = graph.find_edge(current, *neighbor) else {
                return false;
            };
            let weight = *graph.edge_weight(edge).unwrap_or(&1.0);
            distance(*neighbor)
                .is_some_and(|previous| (previous + cost(weight) - current_cost).abs() < 1e-9)
        })?;
        current = previous;
        path.push(current);
    }
    path.reverse();
    Some(path)
}

pub fn neighbors_within(
    graph: &InvestorGraph,
    start: NodeIndex,
    max_hops: usize,
) -> BTreeSet<NodeIndex> {
    let mut seen = BTreeSet::from([start]);
    let mut queue = VecDeque::from([(start, 0_usize)]);
    while let Some((node, depth)) = queue.pop_front() {
        if depth == max_hops {
            continue;
        }
        for neighbor in graph.neighbors(node) {
            if seen.insert(neighbor) {
                queue.push_back((neighbor, depth + 1));
            }
        }
    }
    seen
}

pub fn common_neighbors(graph: &InvestorGraph, a: NodeIndex, b: NodeIndex) -> Vec<Uuid> {
    let a_neighbors: BTreeSet<_> = graph.neighbors(a).collect();
    graph
        .neighbors(b)
        .filter(|node| a_neighbors.contains(node))
        .map(|node| graph[node])
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn jaccard_similarity(graph: &InvestorGraph, a: NodeIndex, b: NodeIndex) -> f64 {
    let left: BTreeSet<_> = graph.neighbors(a).collect();
    let right: BTreeSet<_> = graph.neighbors(b).collect();
    let union = left.union(&right).count();
    if union == 0 {
        0.0
    } else {
        left.intersection(&right).count() as f64 / union as f64
    }
}

pub fn local_clustering_coefficient(graph: &InvestorGraph, node: NodeIndex) -> f64 {
    let neighbors: Vec<_> = graph.neighbors(node).collect();
    let possible = neighbors.len() * neighbors.len().saturating_sub(1) / 2;
    if possible == 0 {
        return 0.0;
    }
    let mut actual = 0;
    for i in 0..neighbors.len() {
        for j in (i + 1)..neighbors.len() {
            if graph.find_edge(neighbors[i], neighbors[j]).is_some() {
                actual += 1;
            }
        }
    }
    actual as f64 / possible as f64
}

pub fn density(graph: &InvestorGraph) -> f64 {
    let nodes = graph.node_count();
    if nodes < 2 {
        0.0
    } else {
        2.0 * graph.edge_count() as f64 / (nodes * (nodes - 1)) as f64
    }
}

/// Deterministic synchronous label propagation. Labels are stable UUIDs of seed nodes;
/// this is a heuristic partition, not a statistical or ground-truth classification.
pub fn label_propagation(graph: &InvestorGraph, max_iterations: usize) -> BTreeMap<Uuid, Uuid> {
    let nodes: Vec<_> = graph.node_indices().collect();
    let mut labels: Vec<_> = nodes.iter().map(|node| graph[*node]).collect();
    for _ in 0..max_iterations {
        let previous = labels.clone();
        let mut changed = false;
        for node in &nodes {
            let mut votes: BTreeMap<Uuid, f64> = BTreeMap::new();
            for edge in graph.edges(*node) {
                let neighbor = if edge.source() == *node {
                    edge.target()
                } else {
                    edge.source()
                };
                *votes.entry(previous[neighbor.index()]).or_default() += edge.weight().max(0.0);
            }
            if let Some((&label, _)) = votes
                .iter()
                .max_by(|a, b| a.1.total_cmp(b.1).then_with(|| b.0.cmp(a.0)))
                && labels[node.index()] != label
            {
                labels[node.index()] = label;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    nodes
        .into_iter()
        .map(|node| (graph[node], labels[node.index()]))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (InvestorGraph, [NodeIndex; 5]) {
        let mut graph = InvestorGraph::default();
        let nodes = [1, 2, 3, 4, 5].map(|id| graph.add_node(Uuid::from_u128(id)));
        graph.add_edge(nodes[0], nodes[1], 10.0);
        graph.add_edge(nodes[1], nodes[2], 2.0);
        graph.add_edge(nodes[0], nodes[2], 1.0);
        graph.add_edge(nodes[2], nodes[3], 1.0);
        (graph, nodes)
    }

    #[test]
    fn computes_degree_weighted_degree_and_density() {
        let (graph, n) = fixture();
        assert_eq!(degree(&graph, n[1]), 2);
        assert_eq!(weighted_degree(&graph, n[1]), 12.0);
        assert!((density(&graph) - 0.4).abs() < 1e-12);
    }

    #[test]
    fn computes_common_neighbors_jaccard_and_clustering() {
        let (graph, n) = fixture();
        assert_eq!(common_neighbors(&graph, n[0], n[1]), vec![graph[n[2]]]);
        assert!((jaccard_similarity(&graph, n[0], n[1]) - 1.0 / 3.0).abs() < 1e-12);
        assert!((local_clustering_coefficient(&graph, n[1]) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn finds_hop_and_strongest_paths_and_components() {
        let (graph, n) = fixture();
        assert_eq!(
            shortest_path(&graph, n[0], n[3]),
            Some(vec![n[0], n[2], n[3]])
        );
        assert_eq!(
            strongest_connection_path(&graph, n[0], n[3]),
            Some(vec![n[0], n[1], n[2], n[3]])
        );
        assert_eq!(connected_components(&graph).len(), 2);
    }

    #[test]
    fn centrality_and_community_outputs_cover_every_node() {
        let (graph, n) = fixture();
        let ranks = pagerank(&graph, 0.85, 100, 1e-12);
        assert_eq!(ranks.len(), graph.node_count());
        assert!((ranks.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        let between = betweenness_centrality(&graph, false);
        assert_eq!(between.len(), graph.node_count());
        assert!(between[n[2].index()] > 0.0);
        assert_eq!(label_propagation(&graph, 20).len(), graph.node_count());
        assert_eq!(neighbors_within(&graph, n[0], 1).len(), 3);
        assert_eq!(
            sampled_betweenness_centrality(&graph, graph.node_count(), false),
            between
        );
        let approximate = sampled_betweenness_centrality(&graph, 2, true);
        assert_eq!(approximate.len(), graph.node_count());
        assert!(
            approximate
                .iter()
                .all(|score| score.is_finite() && *score >= 0.0)
        );
    }
}
