FROM fedora:34 AS builder
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN dnf install gcc openssl-devel -y
RUN source $HOME/.cargo/env && cargo install --locked trunk && rustup target add wasm32-unknown-unknown
RUN mkdir -p /opt/podcast-player
COPY podcast-player-api /opt/podcast-player/podcast-player-api
COPY podcast-player-common /opt/podcast-player/podcast-player-common
COPY podcast-player-pwa /opt/podcast-player/podcast-player-pwa
COPY Cargo.* /opt/podcast-player/
RUN source $HOME/.cargo/env && cd /opt/podcast-player/podcast-player-pwa && trunk build --release
RUN source $HOME/.cargo/env && cd /opt/podcast-player && cargo build --release -p podcast-player-api

FROM h0h4/pwa-server:v1.1.0 AS pwa
MAINTAINER Hannes Hochreiner <hannes@hochreiner.net>
COPY --from=builder /opt/podcast-player/podcast-player-pwa/dist /opt/podcast-player/dist

FROM fedora:34 AS api
MAINTAINER Hannes Hochreiner <hannes@hochreiner.net>
COPY --from=builder /opt/podcast-player/target/release/rss-json-service /opt/podcast-player-api
CMD ["/opt/podcast-player-api"]

FROM fedora:34 AS updater
MAINTAINER Hannes Hochreiner <hannes@hochreiner.net>
COPY --from=builder /opt/podcast-player/target/release/updater /opt/podcast-player-updater
CMD ["/opt/podcast-player-updater"]
