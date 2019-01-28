NAME := $(shell awk '/name/ {gsub(/["]/, "", $$3); print $$3}' Cargo.toml)
RPMDIR := .rpm

release:
	@cargo build --release

rpmbuild: ${RPMDIR}/${NAME}.spec rpmprep
	@mkdir -p target/release
	@cargo rpm build -v
	@rm -rf ${RPMDIR}/etc ${RPMDIR}/systemd

rpmprep: package/*
	@cp -r $^ ${RPMDIR}

.PHONY: clean
