FROM centos:7

ENV PATH "/root/.cargo/bin:${PATH}"

RUN yum update -y \
    && yum install -y gcc make rpm-build \
    && yum clean all

WORKDIR /source

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN cargo install cargo-rpm
