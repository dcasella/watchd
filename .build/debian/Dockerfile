FROM debian:stretch

ENV PATH "/home/builder/.cargo/bin:${PATH}"

RUN apt-get update \
    && apt-get install -y curl gcc make \
    && apt-get clean

RUN adduser builder
USER builder
WORKDIR /source

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN cargo install cargo-deb
