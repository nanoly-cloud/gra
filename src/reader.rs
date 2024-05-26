use anyhow::Result;
use blake3::keyed_hash;
use hashbrown::HashSet;
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};
use tracing::{debug, info, trace};

use crate::{
    hash::{Hash, HashOpts},
    models::{Block, Entry, BLOCK_SIZE},
    node::Node,
};

pub const BUF_SIZE: usize = 1024;

// pub fn add_path(node: &mut Node, path: &PathBuf) -> Result<()> {
//     info!("Adding Path: {:?}", path);
//     read_file(node, path)
// }

// pub fn read_file(node: &mut Node, path: &PathBuf) -> Result<()> {
//     let file = std::fs::File::open(path)?;
//     let mut reader = std::io::BufReader::new(file);
//     let mut buffer = [0u8; BUF_SIZE];
//     let scope = keyed_hash(path.to_owned().to_str().unwrap_or(""));
//     loop {
//         let bytes_read = reader.read(&mut buffer)?;
//         if bytes_read == 0 {
//             break;
//         }
//         let block = Block::new(
//             chrono::Utc::now(),
//             node.identity.public().to_peer_id(),
//             1,
//             Some(scope),
//             buffer[..bytes_read].to_vec(),
//             vec![].into(),
//         );
//         let key = node.insert(&block)?;
//         info!("Added file |  {key:?}: {block:?}");
//         // info!("Validating Presence: {:?}", node.contains(&key));
//     }
//     Ok(())
// }

// TODO: At least rename, or potential implement from/into
/// Function to process a file, or a directory and return the path hash and its chunks
pub fn add_path(path: &Path, scope: Option<Hash>) -> Result<Vec<Entry>> {
    let hash = Hash::new(
        path.to_string_lossy().as_bytes(),
        Some(HashOpts {
            key: scope.to_owned(),
        }),
    );

    let entries: Vec<Entry> = if path.is_file() {
        vec![process_file(path, scope.to_owned())?]
    } else {
        visit_dirs(path, scope.to_owned())?
    };

    Ok(entries)
}

fn process_file(path: &Path, scope: Option<Hash>) -> Result<Entry> {
    let hash = Hash::new(
        path.to_string_lossy().as_bytes(),
        Some(HashOpts { key: scope }),
    );
    let mut file = File::open(path)?;
    let mut buffer = [0; BLOCK_SIZE];
    let mut chunks: Vec<[u8; BLOCK_SIZE]> = Vec::new();
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        if n == BLOCK_SIZE {
            chunks.push(buffer);
        } else {
            let mut partial_chunk = [0; BLOCK_SIZE];
            partial_chunk[..n].copy_from_slice(&buffer[..n]);
            chunks.push(partial_chunk);
        }
    }

    let entry = Entry::new(hash, &Block::Bytes(chunks).into());
    Ok(entry)
}

// Recursive function to traverse directories and process files
fn visit_dirs(dir: &Path, scope: Option<Hash>) -> Result<Vec<Entry>> {
    let mut entries: Vec<Entry> = Vec::new();
    if dir.is_dir() {
        let read_result = fs::read_dir(dir)?;
        for entry in read_result {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                entries.append(&mut visit_dirs(&path, scope.to_owned())?);
            } else if path.is_file() {
                let file_size = path.metadata()?.len();
                entries.push(process_file(&path, scope.to_owned())?);
            }
        }
    }
    Ok(entries)
}
