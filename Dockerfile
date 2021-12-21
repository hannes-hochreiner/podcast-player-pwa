FROM fedora:34 AS builder
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN source $HOME/.cargo/env && cargo install --locked trunk && rustup target add wasm32-unknown-unknown
RUN mkdir -p /opt/podcast-player
COPY src /opt/podcast-player/src
COPY public /opt/podcast-player/public
COPY index.html /opt/podcast-player/index.html
COPY Cargo.* /opt/podcast-player/
RUN source $HOME/.cargo/env && cd /opt/podcast-player && trunk build --release

FROM h0h4/pwa-server:v1.0.1
MAINTAINER Hannes Hochreiner <hannes@hochreiner.net>
COPY --from=builder /opt/podcast-player/dist /opt/podcast-player/dist
ENV ROOT_DIR=/opt/podcast-player/dist
ENV ROCKET_ADDRESS=0.0.0.0