# Stage 1: Build tycho-indexer
FROM rust:1.89 as tycho-indexer-builder

WORKDIR /build
RUN apt-get update && apt-get install -y git

RUN git clone --depth 1 https://github.com/propeller-heads/tycho-indexer.git
WORKDIR /build/tycho-indexer
RUN cargo build --release --bin tycho-indexer

# Stage 2: Build tycho-protocol-sdk and protocol-testing
FROM rust:1.89 as protocol-sdk-builder

WORKDIR /build
RUN apt-get update && apt-get install -y git

RUN git clone --depth 1 https://github.com/propeller-heads/tycho-protocol-sdk.git
RUN git clone --depth 1 https://github.com/propeller-heads/tycho-simulation.git
WORKDIR /build/tycho-protocol-sdk/substreams

# Copy and run the build script for all crates
RUN chmod +x build_all.sh && ./build_all.sh

WORKDIR /build/tycho-protocol-sdk/protocol-testing
RUN cargo build --release

# Stage 3: Final image
FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates

# Copy binaries from previous stages
COPY --from=tycho-indexer-builder /build/tycho-indexer/target/release/tycho-indexer /usr/local/bin/tycho-indexer
COPY --from=protocol-sdk-builder /build/tycho-protocol-sdk/protocol-testing/target/release/tycho-protocol-sdk /usr/local/bin/tycho-protocol-sdk

WORKDIR /app

# Entrypoint script to run tests
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
