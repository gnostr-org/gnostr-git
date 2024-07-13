-include Makefile

cargo-dist-build:
	cargo dist build \
	--artifacts=global \
	--tag=$(shell git for-each-ref refs/tags --sort=-taggerdate --format="%(refname:short)" \
	--count=1 \
	--points-at=HEAD)

# vim: set noexpandtab:
# vim: set setfiletype make
