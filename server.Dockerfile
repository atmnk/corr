FROM rustlang/rust:nightly as builder

RUN USER=root cargo new --bin corr
WORKDIR ./corr
RUN rm src/*.rs
ADD . ./
RUN cargo build --release


FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 8765

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /corr/target/release/corrs ${APP}/corrs
COPY --from=builder /corr/cfg/docker.toml /usr/local/etc/corrs.toml
COPY --from=builder /corr/index.html /usr/local/etc/index.html

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./corrs"]