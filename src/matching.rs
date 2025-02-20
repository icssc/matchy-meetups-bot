use crate::helpers::{Match, Pairing};
use anyhow::{bail, ensure, Context, Result};
use itertools::Itertools;
use petgraph::algo::{maximum_matching, Matching};
use petgraph::matrix_graph::MatrixGraph;
use petgraph::Undirected;
use rand::prelude::SliceRandom;
use rand::SeedableRng;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

type NodeId = u16; // for adjacency matrix

/// Shuffles and returns an immutable vec
fn shuffled<T>(mut vec: Vec<T>, seed: u64) -> Vec<T> {
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    vec.shuffle(&mut rng);
    vec
}

/// Creates "pairs" from the vector (up to one triple is created if there is not an even number).
/// Each pair is represented as a smaller vector
/// within the larger returned vector.
pub fn random_pair<T: Clone>(vec: Vec<T>, seed: u64) -> Pairing<T> {
    if vec.len() <= 1 {
        panic!("Cannot pair with <= 1 elements.")
    }
    let vec = shuffled(vec, seed);

    let chunks = vec.chunks_exact(2);
    let remainder = chunks.remainder();
    let mut x: Vec<Vec<T>> = chunks.map(|chunk| chunk.to_vec()).collect();
    x.last_mut().unwrap().extend_from_slice(remainder);
    Pairing(x, Vec::new())
}

type UnMatrix = MatrixGraph<(), (), Undirected, Option<()>, NodeId>;

/// (lower_id, upper_id)
#[derive(Eq, Hash, PartialEq)]
struct ConstraintEdge {
    lower: NodeId,
    upper: NodeId,
}

impl ConstraintEdge {
    pub fn new((a, b): (NodeId, NodeId)) -> Self {
        ConstraintEdge {
            lower: min(a, b),
            upper: max(a, b),
        }
    }
}

/// Creates "pairs" from the vector (Some triples may be created if necessary).
/// Uses a graph matching algorithm.
pub fn graph_pair<T: Hash + Eq + Copy>(
    vec: Vec<T>,
    previous_pairings: &Vec<Match<T>>,
    seed: u64,
) -> Result<Pairing<T>> {
    if vec.len() < 2 {
        bail!("Cannot pair with < 2 elements.");
    }
    if vec.len() > 200 {
        bail!("Exceeded the 200-element limit of graph_pair() (this can be increased if we verify performance)");
    }
    let vec = shuffled(vec, seed);

    let (graph, constraints) = build_matching_graph(&vec, previous_pairings);
    let matching = maximum_matching(&graph);

    let matched: Vec<Match<NodeId>> = matching
        .edges()
        .map(|(a, b)| vec![a.index() as NodeId, b.index() as NodeId])
        .collect();

    // this assumption is used when iterating over matchings in add_remainder
    ensure!(!matched.is_empty(), "Matching was unexpectedly empty");

    let (imperfect_match_pairs, remainder) = pair_unmatched(&graph, &matching);

    // add remainder to matched
    let (matched_with_remainder, remainder_match_score) =
        add_remainder_to_pairing(matched, remainder, &constraints)?;

    let index_to_element = |i: NodeId| vec[i as usize];

    let imperfect_matches = {
        let mut v: Vec<_> = imperfect_match_pairs
            .iter()
            .flatten()
            .cloned()
            .map(index_to_element)
            .collect();
        if let Some(remainder) = remainder {
            if remainder_match_score > 0 {
                v.push(index_to_element(remainder));
            }
        }
        v
    };

    let matched_with_remainder = matched_with_remainder
        .into_iter()
        .chain(imperfect_match_pairs)
        .map(|m| m.into_iter().map(index_to_element).collect())
        .collect();

    Ok(Pairing(matched_with_remainder, imperfect_matches))
}

fn build_matching_graph<T: Hash + Eq + Copy>(
    vec: &Vec<T>,
    previous_pairings: &Vec<Match<T>>,
) -> (UnMatrix, HashSet<ConstraintEdge>) {
    let nodes: HashMap<&T, NodeId> = vec
        .iter()
        .enumerate()
        .map(|(i, x)| {
            (
                x,
                i.try_into()
                    .expect("`vec` length should fit within NodeId (u16) due to above check"),
            )
        })
        .collect();

    let constraints: HashSet<ConstraintEdge> = previous_pairings
        .iter()
        .flat_map(|m| {
            // convert a Match into an iterable of edges of type NodeId
            // each edge has the smaller index first
            m.iter()
                .flat_map(|u| nodes.get(u)) // filters out constraints not in `vec`
                .map(|u| u.clone())
                .tuple_combinations()
                .map(ConstraintEdge::new)
        })
        .collect();

    (
        UnMatrix::from_edges(
            nodes
                .values()
                .map(|i| *i)
                .tuple_combinations()
                .filter(|e| !constraints.contains(&ConstraintEdge::new(*e))),
        ),
        constraints,
    )
}

/// Pairs the nodes not in the matching, returning the pairs and a possible remainder
fn pair_unmatched(
    graph: &UnMatrix,
    matching: &Matching<&UnMatrix>,
) -> (Vec<Match<NodeId>>, Option<NodeId>) {
    let unmatched: Vec<NodeId> = (0..(graph.node_count() as NodeId))
        .filter(|n| !matching.contains_node((*n).into()))
        .collect();
    let unmatched_pairs = unmatched.chunks_exact(2);
    assert!(
        unmatched_pairs.remainder().len() <= 1,
        "got more than 1 remainder"
    );
    (
        unmatched_pairs.clone().map(|p| p.to_vec()).collect(),
        unmatched_pairs.remainder().first().cloned(),
    )
}

/// Returns a new pairing with the remainder added to the most compatible Match,
/// and returns the remainder score.
fn add_remainder_to_pairing(
    mut matched: Vec<Match<NodeId>>,
    remainder: Option<NodeId>,
    constraints: &HashSet<ConstraintEdge>,
) -> Result<(Vec<Match<NodeId>>, usize)> {
    match remainder {
        Some(remainder) => {
            let (remainder_match_score, remainder_match) = matched
                .iter_mut()
                .map(|v| {
                    let count = v
                        .iter()
                        .filter(|x| constraints.contains(&ConstraintEdge::new((**x, remainder))))
                        .count();
                    (count, v)
                })
                .min()
                .map(|(score, m)| (score, m))
                .context("Unexpectedly encountered empty matched iterable")?;

            remainder_match.push(remainder);

            // the remainder match score is the number of people in `remainder_match` that `remainder`
            // has constraints against it. lower is better.
            Ok((matched, remainder_match_score))
        }
        _ => Ok((matched, 0)),
    }
}
