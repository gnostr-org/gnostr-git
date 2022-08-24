use git_repository as git;

mod write_object {
    use crate::repository::object::empty_bare_repo;

    #[test]
    fn empty_tree() -> crate::Result {
        let (_tmp, repo) = empty_bare_repo()?;
        let oid = repo.write_object(&git_repository::objs::TreeRef::empty())?;
        assert_eq!(
            oid,
            git_repository::hash::ObjectId::empty_tree(repo.object_hash()),
            "it produces a well-known empty tree id"
        );
        Ok(())
    }
}

mod write_blob {
    use crate::repository::object::empty_bare_repo;
    use git_testtools::hex_to_id;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn from_slice() -> crate::Result {
        let (_tmp, repo) = empty_bare_repo()?;
        let oid = repo.write_blob(b"hello world")?;
        assert_eq!(oid, hex_to_id("95d09f2b10159347eece71399a7e2e907ea3df4f"));
        Ok(())
    }

    #[test]
    fn from_stream() -> crate::Result {
        let (_tmp, repo) = empty_bare_repo()?;
        let mut cursor = std::io::Cursor::new(b"hello world");
        let mut seek_cursor = cursor.clone();
        let oid = repo.write_blob_stream(&mut cursor)?;
        assert_eq!(oid, hex_to_id("95d09f2b10159347eece71399a7e2e907ea3df4f"));

        seek_cursor.seek(SeekFrom::Start(6))?;
        let oid = repo.write_blob_stream(&mut seek_cursor)?;
        assert_eq!(
            oid,
            hex_to_id("04fea06420ca60892f73becee3614f6d023a4b7f"),
            "it computes the object size correctly"
        );

        assert_eq!(
            oid.object()?.data,
            &b"world"[..],
            "the seek position is taken into account, so only part of the input data is written"
        );
        Ok(())
    }
}

mod find {

    #[test]
    fn find_and_try_find_with_and_without_object_cache() -> crate::Result {
        let mut repo = crate::basic_repo()?;

        assert_eq!(
            repo.worktrees()?.len(),
            0,
            "it's OK to query linked worktrees in a repo without worktrees"
        );
        for round in 1..=2 {
            match round {
                1 => repo.object_cache_size(None),
                2 => repo.object_cache_size(128 * 1024),
                _ => unreachable!("BUG"),
            }
            for commit_id in repo.head()?.peeled()?.id().expect("born").ancestors().all()? {
                let commit = commit_id?;
                assert_eq!(commit.object()?.kind, git_object::Kind::Commit);
                if round == 2 {
                    assert_eq!(
                        commit.object()?.kind,
                        git_object::Kind::Commit,
                        "repeated request triggers cache and doesn't fail"
                    );
                }
                assert_eq!(commit.try_object()?.expect("exists").kind, git_object::Kind::Commit,);
            }
        }
        Ok(())
    }
}

mod tag {
    #[test]
    fn simple() -> crate::Result {
        let (repo, _keep) = crate::repo_rw("make_basic_repo.sh")?;
        let current_head_id = repo.head_id()?;
        let message = "a multi\nline message";
        let tag_ref = repo.tag(
            "v1.0.0",
            &current_head_id,
            git_object::Kind::Commit,
            Some(repo.committer_or_default()),
            message,
            git_ref::transaction::PreviousValue::MustNotExist,
        )?;
        assert_eq!(tag_ref.name().as_bstr(), "refs/tags/v1.0.0");
        assert_ne!(tag_ref.id(), current_head_id, "it points to the tag object");
        let tag = tag_ref.id().object()?;
        let tag = tag.try_to_tag_ref()?;
        assert_eq!(tag.name, "v1.0.0");
        assert_eq!(current_head_id, tag.target(), "the tag points to the commit");
        assert_eq!(tag.target_kind, git_object::Kind::Commit);
        assert_eq!(
            tag.tagger.as_ref().expect("tagger").actor(),
            repo.committer_or_default().actor()
        );
        assert_eq!(tag.message, message);
        Ok(())
    }
}

mod commit {
    use git_repository as git;
    use git_testtools::hex_to_id;
    #[test]
    fn parent_in_initial_commit_causes_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = git::init(&tmp).unwrap();
        let empty_tree_id = repo.write_object(&git::objs::Tree::empty()).unwrap().detach();
        let author = git::actor::Signature::empty();
        let err = repo
            .commit(
                "HEAD",
                author.to_ref(),
                author.to_ref(),
                "initial",
                empty_tree_id,
                [empty_tree_id],
            )
            .unwrap_err();
        assert_eq!(
            err.to_string(),
            "Reference 'refs/heads/main' was supposed to exist with value 4b825dc642cb6eb9a060e54bf8d69288fbee4904, but didn't.",
            "cannot provide parent id in initial commit"
        );
    }

    #[test]
    fn single_line_initial_commit_empty_tree_ref_nonexisting() -> crate::Result {
        let tmp = tempfile::tempdir()?;
        let repo = git::init(&tmp)?;
        let empty_tree_id = repo.write_object(&git::objs::Tree::empty())?;
        let author = git::actor::Signature::empty();
        let commit_id = repo.commit(
            "HEAD",
            author.to_ref(),
            author.to_ref(),
            "initial",
            empty_tree_id,
            git::commit::NO_PARENT_IDS,
        )?;
        assert_eq!(
            commit_id,
            hex_to_id("302ea5640358f98ba23cda66c1e664a6f274643f"),
            "the commit id is stable"
        );

        let head = repo.head()?.try_into_referent().expect("born");
        assert_eq!(head.name().as_bstr(), "refs/heads/main", "'main' is the default name");
        assert_eq!(
            head.log_iter()
                .rev()?
                .expect("log present")
                .next()
                .expect("one line")?
                .message,
            "commit (initial): initial"
        );
        Ok(())
    }

    #[test]
    fn multi_line_commit_message_uses_first_line_in_ref_log_ref_nonexisting() -> crate::Result {
        let (repo, _keep) = crate::basic_rw_repo()?;
        let parent = repo.find_reference("HEAD")?.peel_to_id_in_place()?;
        let empty_tree_id = parent.object()?.to_commit_ref_iter().tree_id().expect("tree to be set");
        assert_eq!(
            parent
                .try_object()?
                .expect("present")
                .to_commit_ref_iter()
                .tree_id()
                .expect("tree to be set"),
            empty_tree_id,
            "try and non-try work the same"
        );
        let author = git::actor::Signature::empty();
        let first_commit_id = repo.commit(
            "HEAD",
            author.to_ref(),
            author.to_ref(),
            "hello there \r\n\nthe body",
            empty_tree_id,
            Some(parent),
        )?;
        assert_eq!(
            first_commit_id,
            hex_to_id("1ff7decccf76bfa15bfdb0b66bac0c9144b4b083"),
            "the commit id is stable"
        );

        let head_log_entries: Vec<_> = repo
            .head()?
            .log_iter()
            .rev()?
            .expect("log present")
            .map(Result::unwrap)
            .map(|l| l.message)
            .collect();
        assert_eq!(
            head_log_entries,
            vec!["commit: hello there", "commit: c2", "commit (initial): c1"],
            "we get the actual HEAD log, not the log of some reference"
        );
        let current_commit = repo.head()?.into_fully_peeled_id().expect("born")?;
        assert_eq!(current_commit, first_commit_id, "the commit was set");

        let second_commit_id = repo.commit(
            "refs/heads/new-branch",
            author.to_ref(),
            author.to_ref(),
            "committing into a new branch creates it",
            empty_tree_id,
            Some(first_commit_id),
        )?;

        assert_eq!(
            second_commit_id,
            hex_to_id("b0d041ade77e51d31c79c7147fb769336ccc77b1"),
            "the second commit id is stable"
        );

        let mut branch = repo.find_reference("new-branch")?;
        let current_commit = branch.peel_to_id_in_place()?;
        assert_eq!(current_commit, second_commit_id, "the commit was set");

        let mut log = branch.log_iter();
        let mut log_iter = log.rev()?.expect("log present");
        assert_eq!(
            log_iter.next().expect("one line")?.message,
            "commit: committing into a new branch creates it"
        );
        assert!(
            log_iter.next().is_none(),
            "there is only one log line in the new branch"
        );
        Ok(())
    }
}

fn empty_bare_repo() -> crate::Result<(tempfile::TempDir, git::Repository)> {
    let tmp = tempfile::tempdir()?;
    let repo = git::ThreadSafeRepository::init_opts(
        tmp.path(),
        git::create::Options {
            bare: true,
            fs_capabilities: None,
        },
        git::open::Options::isolated(),
    )?
    .into();
    Ok((tmp, repo))
}