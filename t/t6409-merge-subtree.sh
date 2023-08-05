#!/bin/sh

test_description='subtree merge strategy'

GIT_TEST_DEFAULT_INITIAL_BRANCH_NAME=main
export GIT_TEST_DEFAULT_INITIAL_BRANCH_NAME

. ./test-lib.sh

test_expect_success setup '

	s="1 2 3 4 5 6 7 8" &&
	test_write_lines $s >hello &&
	gnostr-git add hello &&
	gnostr-git commit -m initial &&
	gnostr-git checkout -b side &&
	echo >>hello world &&
	gnostr-git add hello &&
	gnostr-git commit -m second &&
	gnostr-git checkout main &&
	test_write_lines mundo $s >hello &&
	gnostr-git add hello &&
	gnostr-git commit -m main

'

test_expect_success 'subtree available and works like recursive' '

	gnostr-git merge -s subtree side &&
	test_write_lines mundo $s world >expect &&
	test_cmp expect hello

'

test_expect_success 'setup branch sub' '
	gnostr-git checkout --orphan sub &&
	gnostr-git rm -rf . &&
	test_commit foo
'

test_expect_success 'setup topic branch' '
	gnostr-git checkout -b topic main &&
	gnostr-git merge -s ours --no-commit --allow-unrelated-histories sub &&
	gnostr-git read-tree --prefix=dir/ -u sub &&
	gnostr-git commit -m "initial merge of sub into topic" &&
	test_path_is_file dir/foo.t &&
	test_path_is_file hello
'

test_expect_success 'update branch sub' '
	gnostr-git checkout sub &&
	test_commit bar
'

test_expect_success 'update topic branch' '
	gnostr-git checkout topic &&
	gnostr-git merge -s subtree sub -m "second merge of sub into topic" &&
	test_path_is_file dir/bar.t &&
	test_path_is_file dir/foo.t &&
	test_path_is_file hello
'

test_expect_success 'setup' '
	mkdir gnostr-git-gui &&
	cd gnostr-git-gui &&
	gnostr-git init &&
	echo gnostr-git-gui > gnostr-git-gui.sh &&
	o1=$(gnostr-git hash-object gnostr-git-gui.sh) &&
	gnostr-git add gnostr-git-gui.sh &&
	gnostr-git commit -m "initial gnostr-git-gui" &&
	cd .. &&
	mkdir gnostr-git &&
	cd gnostr-git &&
	gnostr-git init &&
	echo gnostr-git >gnostr-git.c &&
	o2=$(gnostr-git hash-object gnostr-git.c) &&
	gnostr-git add gnostr-git.c &&
	gnostr-git commit -m "initial gnostr-git"
'

test_expect_success 'initial merge' '
	gnostr-git remote add -f gui ../gnostr-git-gui &&
	gnostr-git merge -s ours --no-commit --allow-unrelated-histories gui/main &&
	gnostr-git read-tree --prefix=gnostr-git-gui/ -u gui/main &&
	gnostr-git commit -m "Merge gnostr-git-gui as our subdirectory" &&
	gnostr-git checkout -b work &&
	gnostr-git ls-files -s >actual &&
	(
		echo "100644 $o1 0	gnostr-git-gui/gnostr-git-gui.sh" &&
		echo "100644 $o2 0	gnostr-git.c"
	) >expected &&
	test_cmp expected actual
'

test_expect_success 'merge update' '
	cd ../gnostr-git-gui &&
	echo gnostr-git-gui2 > gnostr-git-gui.sh &&
	o3=$(gnostr-git hash-object gnostr-git-gui.sh) &&
	gnostr-git add gnostr-git-gui.sh &&
	gnostr-git checkout -b topic_2 &&
	gnostr-git commit -m "update gnostr-git-gui" &&
	cd ../gnostr-git &&
	gnostr-git pull --no-rebase -s subtree gui topic_2 &&
	gnostr-git ls-files -s >actual &&
	(
		echo "100644 $o3 0	gnostr-git-gui/gnostr-git-gui.sh" &&
		echo "100644 $o2 0	gnostr-git.c"
	) >expected &&
	test_cmp expected actual
'

test_expect_success 'initial ambiguous subtree' '
	cd ../gnostr-git &&
	gnostr-git reset --hard main &&
	gnostr-git checkout -b topic_2 &&
	gnostr-git merge -s ours --no-commit gui/main &&
	gnostr-git read-tree --prefix=gnostr-git-gui2/ -u gui/main &&
	gnostr-git commit -m "Merge gnostr-git-gui2 as our subdirectory" &&
	gnostr-git checkout -b work2 &&
	gnostr-git ls-files -s >actual &&
	(
		echo "100644 $o1 0	gnostr-git-gui/gnostr-git-gui.sh" &&
		echo "100644 $o1 0	gnostr-git-gui2/gnostr-git-gui.sh" &&
		echo "100644 $o2 0	gnostr-git.c"
	) >expected &&
	test_cmp expected actual
'

test_expect_success 'merge using explicit' '
	cd ../gnostr-git &&
	gnostr-git reset --hard topic_2 &&
	gnostr-git pull --no-rebase -Xsubtree=gnostr-git-gui gui topic_2 &&
	gnostr-git ls-files -s >actual &&
	(
		echo "100644 $o3 0	gnostr-git-gui/gnostr-git-gui.sh" &&
		echo "100644 $o1 0	gnostr-git-gui2/gnostr-git-gui.sh" &&
		echo "100644 $o2 0	gnostr-git.c"
	) >expected &&
	test_cmp expected actual
'

test_expect_success 'merge2 using explicit' '
	cd ../gnostr-git &&
	gnostr-git reset --hard topic_2 &&
	gnostr-git pull --no-rebase -Xsubtree=gnostr-git-gui2 gui topic_2 &&
	gnostr-git ls-files -s >actual &&
	(
		echo "100644 $o1 0	gnostr-git-gui/gnostr-git-gui.sh" &&
		echo "100644 $o3 0	gnostr-git-gui2/gnostr-git-gui.sh" &&
		echo "100644 $o2 0	gnostr-git.c"
	) >expected &&
	test_cmp expected actual
'

test_done
