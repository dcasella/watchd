FROM debian:stretch

ENV PATH "/home/builder/.cargo/bin:${PATH}"

RUN apt-get update \
    && apt-get install -y curl gcc make \
    && apt-get clean

RUN adduser builder
USER builder
WORKDIR /source

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
RUN cargo install cargo-deb
