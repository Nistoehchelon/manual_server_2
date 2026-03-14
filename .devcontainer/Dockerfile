ARG VARIANT="noble"
FROM mcr.microsoft.com/devcontainers/base:${VARIANT}

# Install deps needed for Rust toolchain (curl, build-essential, etc.)
RUN apt-get update \
 && apt-get install -y --no-install-recommends curl build-essential pkg-config libssl-dev libclang-dev clang

# Install rustup and stable toolchain
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup default stable \
 && rustup component add clippy rustfmt

# (Optional) add user or other setup steps as needed