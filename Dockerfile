FROM clux/muslrust
WORKDIR /corrs
COPY . .
RUN cargo build --release --all-features --package corrs

FROM alpine:latest
WORKDIR /corrs
COPY --from=0 /corrs/cfg/alpine.toml .
COPY --from=0 /corrs/target/x86_64-unknown-linux-musl/release/corrs .
COPY --from=0 /corrs/index.html ./static
CMD ["./corrs","-c","/corrs/alpine.toml"]
