use bstr::ByteSlice;
use git_features::parallel::in_parallel_with_mut_slice;
use git_features::progress::Progress;
use git_features::{interrupt, progress};
use git_hash::oid;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::index::checkout::PathCache;

pub mod checkout;
pub(crate) mod entry;

/// Note that interruption still produce an `Ok(…)` value, so the caller should look at `should_interrupt` to communicate the outcome.
pub fn checkout<Find, E>(
    index: &mut git_index::State,
    dir: impl Into<std::path::PathBuf>,
    find: Find,
    files: &mut impl Progress,
    bytes: &mut impl Progress,
    should_interrupt: &AtomicBool,
    options: checkout::Options,
) -> Result<checkout::Outcome, checkout::Error<E>>
where
    Find: for<'a> FnMut(&oid, &'a mut Vec<u8>) -> Result<git_object::BlobRef<'a>, E> + Send + Clone,
    E: std::error::Error + Send + Sync + 'static,
{
    let num_files = AtomicUsize::default();
    let dir = dir.into();

    let mut ctx = chunk::Context {
        buf: Vec::new(),
        path_cache: {
            let mut cache = PathCache::new(dir.clone());
            cache.unlink_on_collision = options.overwrite_existing;
            cache
        },
        find: find.clone(),
        options,
        num_files: &num_files,
    };
    let (_chunk_size, thread_limit, num_threads) = git_features::parallel::optimize_chunk_size_and_thread_limit(
        100,
        index.entries().len().into(),
        options.thread_limit,
        None,
    );
    let paths = index.take_paths_backing();

    let chunk::Outcome {
        mut collisions,
        mut errors,
        mut bytes_written,
        delayed,
    } = if num_threads == 1 {
        let mut out = chunk::Outcome::default();
        chunk::process(
            interrupt::Iter::new(index.entries_mut().into_iter(), should_interrupt).map(|e| {
                let path = (&paths[e.path.clone()]).as_bstr();
                (e, path)
            }),
            files,
            bytes,
            &mut out,
            &mut ctx,
        )?; // TODO: put paths back
        out
    } else {
        let results = in_parallel_with_mut_slice(
            index.entries_mut(),
            thread_limit,
            {
                let num_files = &num_files;
                move |_| {
                    (
                        progress::Discard,
                        progress::Discard,
                        chunk::Outcome::default(),
                        chunk::Context {
                            find: find.clone(),
                            path_cache: {
                                let mut cache = PathCache::new(dir.clone());
                                cache.unlink_on_collision = options.overwrite_existing;
                                cache
                            },
                            buf: Vec::new(),
                            options,
                            num_files,
                        },
                    )
                }
            },
            |item, (files, bytes, out, ctx)| {
                chunk::process(
                    std::iter::once(item).map(|e| {
                        let path = (&paths[e.path.clone()]).as_bstr();
                        (e, path)
                    }),
                    files,
                    bytes,
                    out,
                    ctx,
                )
            },
            || {
                files.set(num_files.load(Ordering::Relaxed));
                (!should_interrupt.load(Ordering::Relaxed)).then(|| std::time::Duration::from_millis(50))
            },
        )?; // TODO: put paths back in all cases

        chunk::aggregate(results.into_iter().map(|t| t.2))
    };

    for (mut entry, entry_path) in delayed {
        bytes_written += chunk::checkout_entry_handle_result(
            &mut entry,
            entry_path.as_bstr(),
            &mut errors,
            &mut collisions,
            files,
            bytes,
            &mut ctx,
        )? as u64;
    }

    Ok(checkout::Outcome {
        files_updated: ctx.num_files.load(Ordering::Relaxed),
        collisions,
        errors,
        bytes_written,
    })
}

mod chunk {
    use bstr::{BStr, BString};
    use git_features::progress::Progress;
    use git_hash::oid;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::index::{checkout, checkout::PathCache, entry};
    use crate::{index, os};

    mod reduce {
        use crate::index::checkout;
        use git_features::progress::Progress;
        use std::marker::PhantomData;
        use std::sync::atomic::{AtomicUsize, Ordering};

        pub(crate) struct Reduce<'a, P1, P2, E> {
            pub files: &'a mut P1,
            pub bytes: &'a mut P2,
            pub num_files: &'a AtomicUsize,
            pub aggregate: super::Outcome,
            pub marker: PhantomData<E>,
        }

        impl<'a, P1, P2, E> git_features::parallel::Reduce for Reduce<'a, P1, P2, E>
        where
            P1: Progress,
            P2: Progress,
            E: std::error::Error + Send + Sync + 'static,
        {
            type Input = Result<super::Outcome, checkout::Error<E>>;
            type FeedProduce = ();
            type Output = super::Outcome;
            type Error = checkout::Error<E>;

            fn feed(&mut self, item: Self::Input) -> Result<Self::FeedProduce, Self::Error> {
                let item = item?;
                let super::Outcome {
                    bytes_written,
                    delayed,
                    errors,
                    collisions,
                } = item;
                self.aggregate.bytes_written += bytes_written;
                self.aggregate.delayed.extend(delayed);
                self.aggregate.errors.extend(errors);
                self.aggregate.collisions.extend(collisions);

                self.bytes.set(self.aggregate.bytes_written as usize);
                self.files.set(self.num_files.load(Ordering::Relaxed));

                Ok(())
            }

            fn finalize(self) -> Result<Self::Output, Self::Error> {
                Ok(self.aggregate)
            }
        }
    }

    pub(crate) fn aggregate(items: impl IntoIterator<Item = Outcome>) -> Outcome {
        let mut acc = Outcome::default();
        for item in items {
            let Outcome {
                bytes_written,
                delayed,
                errors,
                collisions,
            } = item;
            acc.bytes_written += bytes_written;
            acc.delayed.extend(delayed);
            acc.errors.extend(errors);
            acc.collisions.extend(collisions);
        }
        acc
    }

    #[derive(Default)]
    pub struct Outcome {
        pub collisions: Vec<checkout::Collision>,
        pub errors: Vec<checkout::ErrorRecord>,
        pub delayed: Vec<(git_index::Entry, BString)>,
        pub bytes_written: u64,
    }

    pub struct Context<'a, Find> {
        pub find: Find,
        pub path_cache: PathCache,
        pub buf: Vec<u8>,
        pub options: checkout::Options,
        /// We keep these shared so that there is the chance for printing numbers that aren't looking like
        /// multiple of chunk sizes. Purely cosmetic. Otherwise it's the same as `files`.
        pub num_files: &'a AtomicUsize,
    }

    pub fn process<'entry, 'path, Find, E>(
        entries_with_paths: impl Iterator<Item = (&'entry mut git_index::Entry, &'path BStr)>,
        files: &mut impl Progress,
        bytes: &mut impl Progress,
        Outcome {
            bytes_written,
            errors,
            collisions,
            delayed,
        }: &mut Outcome,
        ctx: &mut Context<'_, Find>,
    ) -> Result<(), checkout::Error<E>>
    where
        Find: for<'a> FnMut(&oid, &'a mut Vec<u8>) -> Result<git_object::BlobRef<'a>, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        for (entry, entry_path) in entries_with_paths {
            // TODO: write test for that
            if entry.flags.contains(git_index::entry::Flags::SKIP_WORKTREE) {
                files.inc();
                continue;
            }

            // Symlinks always have to be delayed on windows as they have to point to something that exists on creation.
            // And even if not, there is a distinction between file and directory symlinks, hence we have to check what the target is
            // before creating it.
            // And to keep things sane, we just do the same on non-windows as well which is similar to what git does and adds some safety
            // around writing through symlinks (even though we handle this).
            // This also means that we prefer content in files over symlinks in case of collisions, which probably is for the better, too.
            if entry.mode == git_index::entry::Mode::SYMLINK {
                delayed.push((entry.to_owned(), entry_path.to_owned()));
                continue;
            }

            *bytes_written +=
                checkout_entry_handle_result(entry, entry_path, errors, collisions, files, bytes, ctx)? as u64;
        }

        Ok(())
    }

    pub fn checkout_entry_handle_result<Find, E>(
        entry: &mut git_index::Entry,
        entry_path: &BStr,
        errors: &mut Vec<checkout::ErrorRecord>,
        collisions: &mut Vec<checkout::Collision>,
        files: &mut impl Progress,
        bytes: &mut impl Progress,
        Context {
            find,
            path_cache,
            buf,
            options,
            num_files,
        }: &mut Context<'_, Find>,
    ) -> Result<usize, checkout::Error<E>>
    where
        Find: for<'a> FnMut(&oid, &'a mut Vec<u8>) -> Result<git_object::BlobRef<'a>, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        let res = entry::checkout(entry, entry_path, find, path_cache, *options, buf);
        files.inc();
        num_files.fetch_add(1, Ordering::SeqCst);
        match res {
            Ok(object_size) => {
                bytes.inc_by(object_size);
                Ok(object_size)
            }
            Err(index::checkout::Error::Io(err)) if os::indicates_collision(&err) => {
                // We are here because a file existed or was blocked by a directory which shouldn't be possible unless
                // we are on a file insensitive file system.
                files.fail(format!("{}: collided ({:?})", entry_path, err.kind()));
                collisions.push(checkout::Collision {
                    path: entry_path.into(),
                    error_kind: err.kind(),
                });
                Ok(0)
            }
            Err(err) => {
                if options.keep_going {
                    errors.push(checkout::ErrorRecord {
                        path: entry_path.into(),
                        error: Box::new(err),
                    });
                    Ok(0)
                } else {
                    Err(err)
                }
            }
        }
    }
}
