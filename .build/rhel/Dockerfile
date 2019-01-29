FROM centos:7

ENV PATH "/home/builder/.cargo/bin:${PATH}"

RUN yum update -y \
    && yum install -y gcc make rpm-build \
    && yum clean all

RUN adduser builder
USER builder
WORKDIR /source

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
RUN cargo install cargo-rpm
