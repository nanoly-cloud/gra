use hashbrown::HashMap;

// Function to generate k-mers from a given sequence
fn generate_kmers(sequence: &str, k: usize) -> Vec<&str> {
    let mut kmers = Vec::new();
    for i in 0..=sequence.len() - k {
        kmers.push(&sequence[i..i + k]);
    }
    kmers
}

// Function to build a k-mer graph
async fn build(sequence: &str, k: usize) -> HashMap<&str, Vec<&str>> {
    let kmers = generate_kmers(sequence, k);
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();

    for i in 0..kmers.len() - 1 {
        let kmer = kmers[i];
        let next_kmer = kmers[i + 1];

        // Add edge from current k-mer to next k-mer
        graph.entry(kmer).or_insert_with(Vec::new).push(next_kmer);
    }

    graph
}

// fn main() {
//     let sequence = "ATCGATCG";
//     let k = 3;
//     let kmer_graph = build_kmer_graph(sequence, k);

//     // Print the k-mer graph
//     for (kmer, edges) in kmer_graph {
//         println!("{} -> {:?}", kmer, edges);
//     }
// }
