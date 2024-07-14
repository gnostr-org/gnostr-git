-include Makefile
-include cargo.mk

cargo-dist-build-tag:
	cargo dist build \
	--artifacts=global \
	--tag=$(shell git for-each-ref refs/tags --sort=-taggerdate --format="%(refname:short)" \
	--count=1 \
	--points-at=HEAD)

fix-releases:
	git tag -f fix-releases-$(shell date +%s)-$(git log -1 --pretty=format:%h) && git push -f --tags

# vim: set noexpandtab:
# vim: set setfiletype make
