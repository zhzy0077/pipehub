ARG BASE_IMAGE=ekidd/rust-musl-builder:latest

FROM ${BASE_IMAGE} AS builder

RUN sudo apt update && sudo apt install -y nodejs npm
RUN sudo npm install -g yarn

ADD --chown=rust:rust . /home/rust
WORKDIR /home/rust/web
RUN yarn && yarn build
WORKDIR /home/rust/server
RUN cargo build --release

FROM alpine:latest
RUN apk --no-cache add ca-certificates
WORKDIR /
COPY --from=builder \
    /home/rust/server/target/x86_64-unknown-linux-musl/release/server \
    /usr/local/bin/
COPY --from=builder \
    /home/rust/web/build \
    /static
CMD /usr/local/bin/server