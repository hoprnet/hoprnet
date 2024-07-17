///
#[allow(clippy::empty_docs)]
pub mod edit {
    use crate::config;

    /// The error returned by [`edit_references(…)`][crate::Repository::edit_references()], and others
    /// which ultimately create a reference.
    #[derive(Debug, thiserror::Error)]
    #[allow(missing_docs)]
    pub enum Error {
        #[error(transparent)]
        FileTransactionPrepare(#[from] gix_ref::file::transaction::prepare::Error),
        #[error(transparent)]
        FileTransactionCommit(#[from] gix_ref::file::transaction::commit::Error),
        #[error(transparent)]
        NameValidation(#[from] gix_validate::reference::name::Error),
        #[error("Could not interpret core.filesRefLockTimeout or core.packedRefsTimeout, it must be the number in milliseconds to wait for locks or negative to wait forever")]
        LockTimeoutConfiguration(#[from] config::lock_timeout::Error),
        #[error(transparent)]
        ParseCommitterTime(#[from] crate::config::time::Error),
    }
}

///
#[allow(clippy::empty_docs)]
pub mod peel {
    /// The error returned by [`Reference::peel_to_id_in_place(…)`](crate::Reference::peel_to_id_in_place()) and
    /// [`Reference::into_fully_peeled_id(…)`](crate::Reference::into_fully_peeled_id()).
    #[derive(Debug, thiserror::Error)]
    #[allow(missing_docs)]
    pub enum Error {
        #[error(transparent)]
        ToId(#[from] gix_ref::peel::to_id::Error),
        #[error(transparent)]
        PackedRefsOpen(#[from] gix_ref::packed::buffer::open::Error),
    }
}

///
#[allow(clippy::empty_docs)]
pub mod head_id {
    /// The error returned by [`Repository::head_id(…)`](crate::Repository::head_id()).
    #[derive(Debug, thiserror::Error)]
    #[allow(missing_docs)]
    pub enum Error {
        #[error(transparent)]
        Head(#[from] crate::reference::find::existing::Error),
        #[error(transparent)]
        PeelToId(#[from] crate::head::peel::into_id::Error),
    }
}

///
#[allow(clippy::empty_docs)]
pub mod head_commit {
    /// The error returned by [`Repository::head_commit`(…)](crate::Repository::head_commit()).
    #[derive(Debug, thiserror::Error)]
    #[allow(missing_docs)]
    pub enum Error {
        #[error(transparent)]
        Head(#[from] crate::reference::find::existing::Error),
        #[error(transparent)]
        PeelToCommit(#[from] crate::head::peel::to_commit::Error),
    }
}

///
#[allow(clippy::empty_docs)]
pub mod head_tree_id {
    /// The error returned by [`Repository::head_tree_id`(…)](crate::Repository::head_tree_id()).
    #[derive(Debug, thiserror::Error)]
    #[allow(missing_docs)]
    pub enum Error {
        #[error(transparent)]
        HeadCommit(#[from] crate::reference::head_commit::Error),
        #[error(transparent)]
        DecodeCommit(#[from] gix_object::decode::Error),
    }
}

///
#[allow(clippy::empty_docs)]
pub mod find {
    ///
    #[allow(clippy::empty_docs)]
    pub mod existing {
        /// The error returned by [`find_reference(…)`][crate::Repository::find_reference()], and others.
        #[derive(Debug, thiserror::Error)]
        #[allow(missing_docs)]
        pub enum Error {
            #[error(transparent)]
            Find(#[from] crate::reference::find::Error),
            #[error("The reference did not exist")]
            NotFound,
        }
    }

    /// The error returned by [`try_find_reference(…)`][crate::Repository::try_find_reference()].
    #[derive(Debug, thiserror::Error)]
    #[allow(missing_docs)]
    pub enum Error {
        #[error(transparent)]
        Find(#[from] gix_ref::file::find::Error),
        #[error(transparent)]
        PackedRefsOpen(#[from] gix_ref::packed::buffer::open::Error),
    }
}
