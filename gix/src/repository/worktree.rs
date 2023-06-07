use crate::{config::cache::util::ApplyLeniencyDefault, worktree, Worktree};

/// Interact with individual worktrees and their information.
impl crate::Repository {
    /// Return a list of all _linked_ worktrees sorted by private git dir path as a lightweight proxy.
    ///
    /// Note that these need additional processing to become usable, but provide a first glimpse a typical worktree information.
    pub fn worktrees(&self) -> std::io::Result<Vec<worktree::Proxy<'_>>> {
        let mut res = Vec::new();
        let iter = match std::fs::read_dir(self.common_dir().join("worktrees")) {
            Ok(iter) => iter,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(res),
            Err(err) => return Err(err),
        };
        for entry in iter {
            let entry = entry?;
            let worktree_git_dir = entry.path();
            if worktree_git_dir.join("gitdir").is_file() {
                res.push(worktree::Proxy {
                    parent: self,
                    git_dir: worktree_git_dir,
                })
            }
        }
        res.sort_by(|a, b| a.git_dir.cmp(&b.git_dir));
        Ok(res)
    }
    /// Return the repository owning the main worktree, typically from a linked worktree.
    ///
    /// Note that it might be the one that is currently open if this repository doesn't point to a linked worktree.
    /// Also note that the main repo might be bare.
    #[allow(clippy::result_large_err)]
    pub fn main_repo(&self) -> Result<crate::Repository, crate::open::Error> {
        crate::ThreadSafeRepository::open_opts(self.common_dir(), self.options.clone()).map(Into::into)
    }

    /// Return the currently set worktree if there is one, acting as platform providing a validated worktree base path.
    ///
    /// Note that there would be `None` if this repository is `bare` and the parent [`Repository`][crate::Repository] was instantiated without
    /// registered worktree in the current working dir.
    pub fn worktree(&self) -> Option<Worktree<'_>> {
        self.work_dir().map(|path| Worktree { parent: self, path })
    }

    /// Return true if this repository is bare, and has no main work tree.
    ///
    /// This is not to be confused with the [`worktree()`][crate::Repository::worktree()] worktree, which may exists if this instance
    /// was opened in a worktree that was created separately.
    pub fn is_bare(&self) -> bool {
        self.config.is_bare && self.work_dir().is_none()
    }

    /// Open a new copy of the index file and decode it entirely.
    ///
    /// It will use the `index.threads` configuration key to learn how many threads to use.
    /// Note that it may fail if there is no index.
    pub fn open_index(&self) -> Result<gix_index::File, worktree::open_index::Error> {
        let thread_limit = self
            .config
            .resolved
            .string("index", None, "threads")
            .map(|value| crate::config::tree::Index::THREADS.try_into_index_threads(value))
            .transpose()
            .with_lenient_default(self.config.lenient_config)?;
        gix_index::File::at(
            self.index_path(),
            self.object_hash(),
            gix_index::decode::Options {
                thread_limit,
                min_extension_block_in_bytes_for_threading: 0,
                expected_checksum: None,
            },
        )
        .map_err(Into::into)
    }

    /// Return a shared worktree index which is updated automatically if the in-memory snapshot has become stale as the underlying file
    /// on disk has changed.
    ///
    /// The index file is shared across all clones of this repository.
    pub fn index(&self) -> Result<worktree::Index, worktree::open_index::Error> {
        self.index
            .recent_snapshot(
                || self.index_path().metadata().and_then(|m| m.modified()).ok(),
                || {
                    self.open_index().map(Some).or_else(|err| match err {
                        worktree::open_index::Error::IndexFile(gix_index::file::init::Error::Io(err))
                            if err.kind() == std::io::ErrorKind::NotFound =>
                        {
                            Ok(None)
                        }
                        err => Err(err),
                    })
                },
            )
            .and_then(|opt| match opt {
                Some(index) => Ok(index),
                None => Err(worktree::open_index::Error::IndexFile(
                    gix_index::file::init::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("Could not find index file at {:?} for opening.", self.index_path()),
                    )),
                )),
            })
    }
}