FROM ubuntu:22.04

ARG RUST_VERSION=1.94.1

ENV DEBIAN_FRONTEND=noninteractive
ENV PATH=/opt/cargo/bin:$PATH
ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        build-essential \
        pkg-config \
        liblzma-dev \
        git \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --no-modify-path --profile minimal --default-toolchain "${RUST_VERSION}"

WORKDIR /workspace

CMD ["cargo", "--version"]
