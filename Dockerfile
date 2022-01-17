FROM rust:1.58 as builder

WORKDIR /tgalertbot

COPY . .

RUN cargo build --release
RUN ls -a

FROM debian:buster-slim
ARG APP=/var/tgalertbot

RUN apt-get update \
    && apt-get install -y libssl-dev pkg-config build-essential ca-certificates

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /tgalertbot/target/release/povorot-tgalertbot ${APP}

RUN chown -R $APP_USER:$APP_USER ${APP}
USER $APP_USER
WORKDIR ${APP}
CMD [ "./povorot-tgalertbot" ]