ifeq ($(project),)
PROJECT_NAME                            := $(notdir $(PWD))
else
PROJECT_NAME                            := $(project)
endif
export PROJECT_NAME

##	:
## doc.rust-lang.org/cargo/reference/profiles.html#custom-profiles
ifeq ($(profile),)
## make profile=release
PROFILE=release
else
## make profile=release-with-debug
PROFILE=release-with-debug
endif

OS                                      :=$(shell uname -s)
export OS
OS_VERSION                              :=$(shell uname -r)
export OS_VERSION
ARCH                                    :=$(shell uname -m)
export ARCH
ifeq ($(ARCH),x86_64)
TRIPLET                                 :=x86_64-linux-gnu
export TRIPLET
endif
ifeq ($(ARCH),arm64)
TRIPLET                                 :=aarch64-linux-gnu
export TRIPLET
endif
ifeq ($(ARCH),arm64)
TRIPLET                                 :=aarch64-linux-gnu
export TRIPLET
endif

ifeq ($(reuse),true)
REUSE                                   :=-r
else
REUSE                                   :=
endif
export REUSE
ifeq ($(bind),true)
BIND                                   :=-b
else
BIND                                   :=
endif
export BIND

ifeq ($(token),)
GITHUB_TOKEN                            :=$(shell touch ~/GITHUB_TOKEN.txt && cat ~/GITHUB_TOKEN.txt || echo "0")
else
GITHUB_TOKEN                            :=$(shell echo $(token))
endif
export GITHUB_TOKEN

export $(cat ~/GITHUB_TOKEN) && make act

PYTHON                                  := $(shell which python)
export PYTHON
PYTHON2                                 := $(shell which python2)
export PYTHON2
PYTHON3                                 := $(shell which python3)
export PYTHON3

PIP                                     := $(shell which pip)
export PIP
PIP2                                    := $(shell which pip2)
export PIP2
PIP3                                    := $(shell which pip3)
export PIP3

PYTHON_VENV                             := $(shell python -c "import sys; sys.stdout.write('1') if hasattr(sys, 'base_prefix') else sys.stdout.write('0')")
PYTHON3_VENV                            := $(shell python3 -c "import sys; sys.stdout.write('1') if hasattr(sys, 'real_prefix') else sys.stdout.write('0')")

python_version_full := $(wordlist 2,4,$(subst ., ,$(shell python3 --version 2>&1)))
python_version_major := $(word 1,${python_version_full})
python_version_minor := $(word 2,${python_version_full})
python_version_patch := $(word 3,${python_version_full})

my_cmd.python.3 := $(PYTHON3) some_script.py3
my_cmd := ${my_cmd.python.${python_version_major}}

PYTHON_VERSION                         := ${python_version_major}.${python_version_minor}.${python_version_patch}
PYTHON_VERSION_MAJOR                   := ${python_version_major}
PYTHON_VERSION_MINOR                   := ${python_version_minor}

export python_version_major
export python_version_minor
export python_version_patch
export PYTHON_VERSION

CARGO:=$(shell which cargo)
export CARGO
RUSTC:=$(shell which rustc)
export RUSTC
RUSTUP:=$(shell which rustup)
export RUSTUP
BREW:=$(shell which brew)
export BREW
APT_GET:=$(shell which apt-get)
export APT_GET

HELP2MAN:=$(shell which help2man)
export HELP2MAN
-:
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?##/ {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)
help:## 	help
	@sed -n 's/^##//p' ${MAKEFILE_LIST} | column -t -s ':' |  sed -e 's/^/ /'
##	:
## RUSTUP

rustup-install:rustup-install-stable## 	rustup-install
rustup-install-stable:## 	rustup-install-stable
	[ -x "$(shell command -v $(APT_GET))" ] && sudo $(APT_GET) -y install musl-tools || \
    [ -x "$(shell command -v $(BREW))" ] && $(BREW) install filosottile/musl-cross/musl-cross || true
	[ -x "$(shell command -v $(RUSTUP))" ] || \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain stable --profile default || \
    . "$(HOME)/.cargo/env" || true
	[ -x "$(shell command -v $(RUSTUP))" ] && $(RUSTUP) default stable

rustup-install-nightly:## 	rustup-install-nightly
	[ -x "$(shell command -v $(APT_GET))" ] && sudo $(APT_GET) -y install musl-tools || \
    [ -x "$(shell command -v $(BREW))" ] && $(BREW) install filosottile/musl-cross/musl-cross || true
	[ -x "$(shell command -v $(RUSTUP))" ] || \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path --default-toolchain nightly --profile default || \
    . "$(HOME)/.cargo/env" || true
	[ -x "$(shell command -v $(RUSTUP))" ] && $(RUSTUP) default nightly

rustup-target-add:## 	rustup-target-add
## rustup target add x86_64-unknown-linux-musl
	rustup target add x86_64-unknown-linux-musl
## rustup target add aarch64-unknown-linux-gnu
	rustup target add aarch64-unknown-linux-gnu
	rustup target add aarch64-unknown-linux-gnu
## ARM64 Linux (kernel 4.1, glibc 2.17+) 1
	rustup target add i686-pc-windows-gnu
## 32-bit MinGW (Windows 7+) 2 3
	rustup target add i686-pc-windows-msvc
## 32-bit MSVC (Windows 7+) 2 3
	rustup target add i686-unknown-linux-gnu
## 32-bit Linux (kernel 3.2+, glibc 2.17+) 3
	rustup target add x86_64-apple-darwin
## 64-bit macOS (10.12+, Sierra+)
	rustup target add x86_64-pc-windows-gnu
## 64-bit MinGW (Windows 7+) 2
	rustup target add x86_64-pc-windows-msvc
## 64-bit MSVC (Windows 7+) 2
	rustup target add x86_64-unknown-linux-gnu
## 64-bit Linux (kernel 3.2+, glibc 2.17+)
##	:

cargo-b:## 	cargo-b
	[ -x "$(shell command -v $(RUSTUP))" ] || $(MAKE) rustup-install-stable
	[ -x "$(shell command -v $(CARGO))" ] && $(CARGO) build
cargo-br: cargo-b-release ## 	cargo-br
cargo-b-release:## 	cargo-b-release
	[ -x "$(shell command -v $(RUSTUP))" ] || $(MAKE) rustup-install-stable
	[ -x "$(shell command -v $(CARGO))" ] && $(CARGO) build --release
cargo-c:## 	cargo-c
	[ -x "$(shell command -v $(RUSTC))" ] || $(MAKE) rustup-install-stable
	[ -x "$(shell command -v $(CARGO))" ] && $(CARGO) c
install:cargo-install## 	install
cargo-i:## 	cargo-i
	[ -x "$(shell command -v $(RUSTC))" ] || $(MAKE) rustup-install-stable
	[ -x "$(shell command -v $(CARGO))" ] && $(CARGO) install --path .

.PHONY:man ## :)
man:
	mkdir -p man
	[ -x "$(shell command -v $(HELP2MAN))" ] && $(HELP2MAN) ./target/release/gnostr-tui > man/gnostr-tui.1 || cat man/gnostr-tui.1

-include Makefile
-include cargo.mk
-include docker.mk
-include act.mk
