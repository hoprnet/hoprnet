pub(crate) type CommitsStorage =
    gix_features::threading::OwnShared<gix_fs::SharedFileSnapshotMut<Vec<gix_hash::ObjectId>>>;
/// A lazily loaded and auto-updated list of commits which are at the shallow boundary (behind which there are no commits available),
/// sorted to allow bisecting.
pub type Commits = gix_fs::SharedFileSnapshot<Vec<gix_hash::ObjectId>>;

///
pub mod read {
    pub use gix_shallow::read::Error;
}

///
pub mod write {
    pub use gix_shallow::write::Error;
}
