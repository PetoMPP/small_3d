FROM rust:1.76

RUN rustup update
RUN rustup target add x86_64-pc-windows-msvc
RUN cargo install xwin
RUN xwin --accept-license splat --output /var/lib/xwin
RUN apt-get update && apt-get install lld -yy

ENV CARGO_FEATURE_PURE=1 

WORKDIR /app