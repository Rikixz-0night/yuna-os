SHELL := /bin/bash
ROOT  := $(CURDIR)

.PHONY: all tools repo iso kernel fetch check test test-fast clean

all: tools repo

## Build the Rust tooling (yuna-fetch + yunainstall)
tools:
	cd rust && cargo build --release

## Show the system summary with the native fetch tool
fetch: tools
	./rust/target/release/yuna-fetch

## Build yuna-tools / yuna-release and the local pacman repository
repo:
	./scripts/build-repo.sh

## ISO build needs root; this target prepares everything and tells you the command
iso: repo
	@echo
	@echo "repository ready. now run:"
	@echo "  sudo ./scripts/build-iso.sh"

## Bare-metal kernel (separate project, in kernel/)
kernel:
	cd kernel && ./run.sh

## Static checks that do not need root
check: tools
	bash -n scripts/*.sh distro/profiledef.sh \
		distro/airootfs/usr/local/bin/yuna-welcome \
		distro/airootfs/usr/local/bin/yuna-mirrors \
		distro/airootfs/root/customize_airootfs.sh \
		distro/airootfs/etc/profile.d/yuna.sh
	./scripts/check-profile.sh
	./rust/target/release/yunainstall --config examples/yuna.conf --print-script | bash -n
	./rust/target/release/yunainstall --dry-run --yes --config examples/yuna.conf > /dev/null
	@echo "all checks passed"

## Full test suite (no root needed)
test:
	./test.sh

## Test suite without the package build and the QEMU kernel boot
test-fast:
	./test.sh --fast

clean:
	rm -rf rust/target out .work .iso-profile .pacman-cache
	rm -f packages/*/*.pkg.tar.*
	rm -rf packages/*/pkg packages/*/src
