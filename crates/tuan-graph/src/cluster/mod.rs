// Coded by ChatGPT......

use crate::graph::{Graph, NodeId};
use std::collections::HashMap;

pub struct Cluster {
    pub id: usize,
    pub members: Vec<NodeId>,
}

impl Graph {
    fn undirected_adj_weighted(&self) -> HashMap<NodeId, Vec<(NodeId, f32)>> {
        // Arêtes non pondérées -> 1.0 ; symétrisation + fusion
        let mut adj: HashMap<NodeId, Vec<(NodeId, f32)>> = HashMap::new();
        for &u in self.nodes.keys() {
            adj.entry(u).or_default();
        }
        let mut acc: HashMap<(NodeId, NodeId), f32> = HashMap::new();
        for e in &self.edges {
            if e.from == e.to {
                continue;
            }
            *acc.entry((e.from, e.to)).or_insert(0.0) += 1.0;
            *acc.entry((e.to, e.from)).or_insert(0.0) += 1.0;
        }
        for ((u, v), w) in acc {
            adj.entry(u).or_default().push((v, w));
        }
        for vs in adj.values_mut() {
            vs.sort_by_key(|x| x.0);
            let mut fused: Vec<(NodeId, f32)> = Vec::with_capacity(vs.len());
            for &(v, w) in vs.iter() {
                if let Some(last) = fused.last_mut() {
                    if last.0 == v {
                        last.1 += w;
                        continue;
                    }
                }
                fused.push((v, w));
            }
            *vs = fused;
        }
        adj
    }

    // ---------- Qualité de partition ----------
    fn modularity_gamma(
        comm: &Vec<usize>,
        adj: &Vec<Vec<(usize, f32)>>,
        degree: &Vec<f32>,
        gamma: f32,
    ) -> f32 {
        // Q_γ = (1/m2) * sum_{i,j in same C} [A_ij - γ * k_i k_j / m2]
        // Implémentation efficace via sommes par communauté
        let n = adj.len();
        let m2: f32 = degree.iter().sum();
        if m2 <= 0.0 {
            return 0.0;
        }

        let mut tot: HashMap<usize, f32> = HashMap::new();
        let mut in_w: HashMap<usize, f32> = HashMap::new();

        for i in 0..n {
            let c = comm[i];
            *tot.entry(c).or_insert(0.0) += degree[i];
            for &(j, w) in &adj[i] {
                if comm[j] == c {
                    *in_w.entry(c).or_insert(0.0) += w;
                }
            }
        }
        // in_w compte chaque arête intra deux fois (i->j et j->i). Pas grave : cohérent avec m2 = 2m.
        let mut q = 0.0f32;
        for (c, in_c) in in_w {
            let tot_c = *tot.get(&c).unwrap_or(&0.0);
            q += (in_c / m2) - gamma * (tot_c / m2) * (tot_c / m2);
        }
        q
    }

    fn gini(sizes: &[usize]) -> f32 {
        // Gini sur tailles (0 = uniforme, 1 = toute la masse dans un cluster)
        let n = sizes.len();
        if n == 0 {
            return 0.0;
        }
        let mut v: Vec<f32> = sizes.iter().map(|&x| x as f32).collect();
        v.sort_by(|a, b| a.total_cmp(b));
        let sum: f32 = v.iter().sum();
        if sum == 0.0 {
            return 0.0;
        }
        let mut acc = 0.0f32;
        for (i, &x) in v.iter().enumerate() {
            acc += (i as f32 + 1.0) * x;
        }
        (2.0 * acc) / (n as f32 * sum) - (n as f32 + 1.0) / (n as f32)
    }

    fn louvain_with_gamma(
        adj: &Vec<Vec<(usize, f32)>>,
        degree: &Vec<f32>,
        max_levels: usize,
        gamma: f32,
    ) -> (Vec<Vec<usize>>, Vec<usize>) {
        // Retourne (members par super-nœud final, partition au dernier niveau)
        let n0 = adj.len();
        let mut members: Vec<Vec<usize>> = (0..n0).map(|i| vec![i]).collect();
        let mut adj_cur = adj.clone();
        let mut deg_cur = degree.clone();
        let mut comm: Vec<usize> = (0..adj_cur.len()).collect();

        // local move (ΔQγ) — déterministe (ordre fixe) pour stabilité
        let local_move =
            |adj_l: &Vec<Vec<(usize, f32)>>, deg_l: &Vec<f32>, comm_l: &mut Vec<usize>| -> bool {
                let n = adj_l.len();
                let m2 = deg_l.iter().sum::<f32>();
                if m2 <= 0.0 {
                    return false;
                }

                let mut tot: Vec<f32> = vec![0.0; n];
                for i in 0..n {
                    tot[comm_l[i]] += deg_l[i];
                }

                let mut moved_any = false;
                let mut moved = true;
                let max_passes = 20;
                let mut pass = 0;

                while moved && pass < max_passes {
                    moved = false;
                    pass += 1;

                    for i in 0..n {
                        let ci = comm_l[i];
                        let ki = deg_l[i];
                        if ki == 0.0 {
                            continue;
                        }

                        // poids vers communautés voisines
                        let mut neigh_w_by_c: HashMap<usize, f32> = HashMap::new();
                        for &(j, w_ij) in &adj_l[i] {
                            let cj = comm_l[j];
                            *neigh_w_by_c.entry(cj).or_insert(0.0) += w_ij;
                        }

                        // retire temporairement i de sa comm
                        tot[ci] -= ki;

                        // cherche la meilleure communauté voisine (garde rester si <= 0)
                        let mut best_c = ci;
                        let mut best_gain = 0.0f32;

                        for (&c, &k_i_in_c) in &neigh_w_by_c {
                            if c == ci {
                                continue;
                            }
                            // ΔQγ ∝ k_i,in(c) - γ * ki * tot(c) / m2
                            let gain = k_i_in_c - gamma * ki * (tot[c] / m2);
                            if gain > best_gain || (gain == best_gain && c < best_c) {
                                best_gain = gain;
                                best_c = c;
                            }
                        }

                        if best_c != ci {
                            comm_l[i] = best_c;
                            tot[best_c] += ki;
                            moved = true;
                            moved_any = true;
                        } else {
                            // remet i
                            tot[ci] += ki;
                        }
                    }
                }

                moved_any
            };

        // agrégation
        let aggregate = |adj_l: &Vec<Vec<(usize, f32)>>,
                         comm_l: &Vec<usize>,
                         members_l: &Vec<Vec<usize>>|
         -> (
            Vec<Vec<(usize, f32)>>,
            Vec<f32>,
            Vec<Vec<usize>>,
            Vec<usize>,
        ) {
            let n = adj_l.len();
            let mut seen = HashMap::new();
            let mut next = 0usize;
            let mut cid_map: Vec<usize> = vec![0; n];
            for &c in comm_l {
                seen.entry(c).or_insert_with(|| {
                    let x = next;
                    next += 1;
                    x
                });
            }
            for i in 0..n {
                cid_map[i] = *seen.get(&comm_l[i]).unwrap();
            }
            let k = next;

            let mut acc: HashMap<(usize, usize), f32> = HashMap::new();
            for i in 0..n {
                let ci = cid_map[i];
                for &(j, w) in &adj_l[i] {
                    let cj = cid_map[j];
                    *acc.entry((ci, cj)).or_insert(0.0) += w;
                }
            }

            let mut adj2: Vec<Vec<(usize, f32)>> = vec![Vec::new(); k];
            for ((a, b), w) in acc {
                if w <= 0.0 {
                    continue;
                }
                adj2[a].push((b, w));
            }
            for vs in adj2.iter_mut() {
                vs.sort_by_key(|x| x.0);
                let mut fused: Vec<(usize, f32)> = Vec::with_capacity(vs.len());
                for &(v, w) in vs.iter() {
                    if let Some(last) = fused.last_mut() {
                        if last.0 == v {
                            last.1 += w;
                            continue;
                        }
                    }
                    fused.push((v, w));
                }
                *vs = fused;
            }

            let mut deg2 = vec![0.0f32; k];
            for a in 0..k {
                deg2[a] = adj2[a].iter().map(|&(_, w)| w).sum::<f32>();
            }

            let mut mem2: Vec<Vec<usize>> = vec![Vec::new(); k];
            for i in 0..n {
                let ci = cid_map[i];
                mem2[ci].extend_from_slice(&members_l[i]);
            }
            for v in mem2.iter_mut() {
                v.sort_unstable();
                v.dedup();
            }

            ((adj2), (deg2), (mem2), (0..k).collect())
        };

        let levels = max_levels.max(1).min(20);
        let mut level = 0usize;
        let mut improved = true;

        while improved && level < levels {
            improved = local_move(&adj_cur, &deg_cur, &mut comm);
            if !improved {
                break;
            }
            let (adj2, deg2, mem2, comm2) = aggregate(&adj_cur, &comm, &members);
            adj_cur = adj2;
            deg_cur = deg2;
            members = mem2;
            // repartition triviale au nouveau niveau
            comm = comm2;
            level += 1;
        }

        // Partition finale = chaque super-nœud est une communauté
        let final_partition: Vec<usize> = (0..adj_cur.len()).collect();
        (members, final_partition)
    }

    pub fn clusterize(&self, max_iters: usize) -> Vec<Cluster> {
        // 1) Graphe non orienté pondéré
        let adj_map = self.undirected_adj_weighted();
        let nodes_vec: Vec<NodeId> = adj_map.keys().copied().collect();
        let mut idx_of: HashMap<NodeId, usize> = HashMap::new();
        for (i, &u) in nodes_vec.iter().enumerate() {
            idx_of.insert(u, i);
        }
        let n = nodes_vec.len();

        let mut adj: Vec<Vec<(usize, f32)>> = vec![Vec::new(); n];
        let mut degree: Vec<f32> = vec![0.0; n];
        for (&u, neigh) in &adj_map {
            let iu = idx_of[&u];
            for &(v, w) in neigh {
                let iv = idx_of[&v];
                adj[iu].push((iv, w));
                degree[iu] += w;
            }
        }

        // 2) Balayage de γ + scoring
        let gammas: &[f32] = &[0.6, 0.8, 1.0, 1.2, 1.5, 2.0, 2.5];
        let max_levels = max_iters.max(1).min(20);
        // pénalités (à ajuster finement selon ton graphe)
        const ALPHA: f32 = 0.35; // poids pénalité Gini
        const BETA: f32 = 0.40; // poids pénalité max_share

        let mut best_score = f32::NEG_INFINITY;
        let mut best_members: Vec<Vec<usize>> = Vec::new();

        for &gamma in gammas {
            let (members_gamma, _partition_gamma) =
                Self::louvain_with_gamma(&adj, &degree, max_levels, gamma);

            // reconstruit comm (mapping noeud -> cid final)
            // ici chaque super-nœud final = une communauté, et members_gamma[ci] liste des nœuds originaux
            let mut comm_final: Vec<usize> = vec![0; n];
            for (cid, idxs) in members_gamma.iter().enumerate() {
                for &i0 in idxs {
                    comm_final[i0] = cid;
                }
            }

            // calcule métriques
            let sizes: Vec<usize> = members_gamma.iter().map(|v| v.len()).collect();
            let sum_sizes = sizes.iter().sum::<usize>().max(1) as f32;
            let max_share = sizes.iter().copied().max().unwrap_or(0) as f32 / sum_sizes;
            let qg = Self::modularity_gamma(&comm_final, &adj, &degree, gamma);
            let gini = Self::gini(&sizes);
            let score = qg - ALPHA * gini - BETA * max_share;

            if score > best_score {
                best_score = score;
                best_members = members_gamma;
            }
        }

        // 3) Construit les clusters (NodeId) à partir du meilleur résultat
        let mut clusters: Vec<Cluster> = Vec::new();
        for (cid, idxs) in best_members.into_iter().enumerate() {
            if idxs.is_empty() {
                continue;
            }
            let mut mem: Vec<NodeId> = idxs.into_iter().map(|i| nodes_vec[i]).collect();
            mem.sort_unstable();
            clusters.push(Cluster {
                id: cid,
                members: mem,
            });
        }
        clusters.sort_by_key(|c| c.id);
        clusters
    }
}
