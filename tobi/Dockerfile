FROM rust:1.94-bookworm AS build

WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config liblzma-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY sample ./sample
RUN cargo build --release

FROM debian:trixie-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates liblzma5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=build /app/target/release/tobi /usr/local/bin/tobi
COPY sample ./sample

ENTRYPOINT ["tobi"]
CMD ["--manifest", "sample/catalog.json", "--mode", "mock"]
