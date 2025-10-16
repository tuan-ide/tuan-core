use std::collections::HashMap;

use crate::graph::{Graph, NodeId};

pub struct Cluster {
    pub id: usize,
    pub members: Vec<NodeId>,
}

impl Graph {
    fn undirected_adj(&self) -> HashMap<NodeId, Vec<NodeId>> {
        let mut adj: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        for node_id in self.nodes.keys() {
            adj.entry(*node_id).or_default();
        }
        for e in &self.edges {
            if e.from != e.to {
                adj.entry(e.from).or_default().push(e.to);
                adj.entry(e.to).or_default().push(e.from);
            }
        }
        for v in adj.values_mut() {
            v.sort_unstable();
            v.dedup();
        }
        adj
    }

    pub fn clusterize(&self, max_iters: usize) -> Vec<Cluster> {
        let adj = self.undirected_adj();
        let nodes: Vec<NodeId> = self.nodes.keys().copied().collect();

        let mut label: HashMap<NodeId, NodeId> = nodes.iter().map(|&u| (u, u)).collect();

        let mut changed = true;
        let mut iter = 0;

        while changed && iter < max_iters {
            changed = false;
            for &u in &nodes {
                let neigh = match adj.get(&u) {
                    Some(n) if !n.is_empty() => n,
                    _ => continue,
                };
                let mut counts: HashMap<NodeId, usize> = HashMap::new();
                for &v in neigh {
                    let lv = *label.get(&v).unwrap();
                    *counts.entry(lv).or_insert(0) += 1;
                }
                if let Some((&best_label, _)) = counts
                    .iter()
                    .max_by_key(|(lab, cnt)| (*cnt, std::cmp::Reverse(**lab)))
                {
                    if best_label != *label.get(&u).unwrap() {
                        label.insert(u, best_label);
                        changed = true;
                    }
                }
            }
            iter += 1;
        }

        let mut label_to_cid: HashMap<NodeId, usize> = HashMap::new();
        let mut next_cid = 0usize;
        let mut clusters_map: HashMap<usize, Vec<NodeId>> = HashMap::new();

        for &u in &nodes {
            let lu = *label.get(&u).unwrap();
            let cid = *label_to_cid.entry(lu).or_insert_with(|| {
                let id = next_cid;
                next_cid += 1;
                id
            });
            clusters_map.entry(cid).or_default().push(u);
        }

        let mut clusters: Vec<Cluster> = clusters_map
            .into_iter()
            .map(|(id, mut members)| {
                members.sort_unstable();
                Cluster { id, members }
            })
            .collect();

        clusters.sort_by_key(|c| c.id);
        clusters
    }
}
