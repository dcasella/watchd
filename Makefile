NAME := $(shell awk '/name/ {gsub(/["]/, "", $$3); print $$3}' Cargo.toml)
RPMDIR := .rpm

release:
	@cargo build --release

deb-docker:
	@docker build -t watchd-debian .build/debian
	@docker run --rm -v ${PWD}:/source watchd-debian \
		make deb

rpm-docker:
	@docker build -t watchd-rhel .build/rhel
	@docker run --rm -v ${PWD}:/source watchd-rhel \
		make rpm

deb:
	@cargo deb -v
	@mkdir -p releases
	@cp target/debian/*.deb releases

rpm: ${RPMDIR}/${NAME}.spec rpmprep
	@mkdir -p target/release
	@cargo rpm build -v
	@rm -rf ${RPMDIR}/etc ${RPMDIR}/systemd
	@mkdir -p releases
	@cp target/release/rpmbuild/RPMS/x86_64/*.rpm releases

rpmprep: package/*
	@cp -r $^ ${RPMDIR}
