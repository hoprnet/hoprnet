use std::{
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use crate::{file, File, Graph, MAX_COMMITS};

/// The error returned by initializations functions like [`Graph::at()`].
#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
pub enum Error {
    #[error("{}", .path.display())]
    File {
        #[source]
        err: file::Error,
        path: PathBuf,
    },
    #[error("Commit-graph files mismatch: '{}' uses hash {hash1:?}, but '{}' uses hash {hash2:?}", .path1.display(), .path2.display())]
    HashVersionMismatch {
        path1: PathBuf,
        hash1: gix_hash::Kind,
        path2: PathBuf,
        hash2: gix_hash::Kind,
    },
    #[error("Did not find any files that look like commit graphs at '{}'", .0.display())]
    InvalidPath(PathBuf),
    #[error("Could not open commit-graph file at '{}'", .path.display())]
    Io {
        #[source]
        err: std::io::Error,
        path: PathBuf,
    },
    #[error(
        "Commit-graph files contain {0} commits altogether, but only {} commits are allowed",
        MAX_COMMITS
    )]
    TooManyCommits(u64),
}

/// Instantiate a `Graph` from various sources.
impl Graph {
    /// Instantiate a commit graph from `path` which may be a directory containing graph files or the graph file itself.
    pub fn at(path: &Path) -> Result<Self, Error> {
        Self::try_from(path)
    }

    /// Instantiate a commit graph from the directory containing all of its files.
    pub fn from_commit_graphs_dir(path: &Path) -> Result<Self, Error> {
        let commit_graphs_dir = path;
        let chain_file_path = commit_graphs_dir.join("commit-graph-chain");
        let chain_file = std::fs::File::open(&chain_file_path).map_err(|e| Error::Io {
            err: e,
            path: chain_file_path.clone(),
        })?;
        let mut files = Vec::new();
        for line in BufReader::new(chain_file).lines() {
            let hash = line.map_err(|e| Error::Io {
                err: e,
                path: chain_file_path.clone(),
            })?;
            let graph_file_path = commit_graphs_dir.join(format!("graph-{hash}.graph"));
            files.push(File::at(&graph_file_path).map_err(|e| Error::File {
                err: e,
                path: graph_file_path.clone(),
            })?);
        }
        Self::new(files)
    }

    /// Instantiate a commit graph from a `.git/objects/info/commit-graph` or
    /// `.git/objects/info/commit-graphs/graph-*.graph` file.
    pub fn from_file(path: &Path) -> Result<Self, Error> {
        let file = File::at(path).map_err(|e| Error::File {
            err: e,
            path: path.to_owned(),
        })?;
        Self::new(vec![file])
    }

    /// Instantiate a commit graph from an `.git/objects/info` directory.
    pub fn from_info_dir(info_dir: &Path) -> Result<Self, Error> {
        Self::from_file(&info_dir.join("commit-graph"))
            .or_else(|_| Self::from_commit_graphs_dir(&info_dir.join("commit-graphs")))
    }

    /// Create a new commit graph from a list of `files`.
    pub fn new(files: Vec<File>) -> Result<Self, Error> {
        let num_commits: u64 = files.iter().map(|f| u64::from(f.num_commits())).sum();
        if num_commits > u64::from(MAX_COMMITS) {
            return Err(Error::TooManyCommits(num_commits));
        }

        for window in files.windows(2) {
            let f1 = &window[0];
            let f2 = &window[1];
            if f1.object_hash() != f2.object_hash() {
                return Err(Error::HashVersionMismatch {
                    path1: f1.path().to_owned(),
                    hash1: f1.object_hash(),
                    path2: f2.path().to_owned(),
                    hash2: f2.object_hash(),
                });
            }
        }

        Ok(Self { files })
    }
}

impl TryFrom<&Path> for Graph {
    type Error = Error;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        if path.is_file() {
            // Assume we are looking at `.git/objects/info/commit-graph` or
            // `.git/objects/info/commit-graphs/graph-*.graph`.
            Self::from_file(path)
        } else if path.is_dir() {
            if path.join("commit-graph-chain").is_file() {
                Self::from_commit_graphs_dir(path)
            } else {
                Self::from_info_dir(path)
            }
        } else {
            Err(Error::InvalidPath(path.to_owned()))
        }
    }
}
