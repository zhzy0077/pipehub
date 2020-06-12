# FROM node:lts AS web-builder
# ADD ./web /root/web
# WORKDIR /root/web
# RUN yarn && yarn build

FROM ekidd/rust-musl-builder:latest AS server-builder
ARG DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/postgres
ENV DATABASE_URL ${DATABASE_URL}
ADD --chown=rust:rust ./server /home/rust/server
WORKDIR /home/rust/server
RUN cargo build

FROM alpine:latest
RUN apk --no-cache add ca-certificates
WORKDIR /
COPY --from=server-builder \
    /home/rust/server/target/x86_64-unknown-linux-musl/release/server \
    /usr/local/bin/
COPY --from=web-builder \
    /root/web/build \
    /static
CMD /usr/local/bin/server