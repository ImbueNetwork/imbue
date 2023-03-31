# This file is sourced from https://github.com/paritytech/polkadot/blob/master/scripts/ci/dockerfiles/polkadot/polkadot_builder.Dockerfile
FROM docker.io/paritytech/ci-linux:production as builder

RUN git clone -b briefs-pallet -v https://github.com/imbuenetwork/imbue
WORKDIR /builds/imbue

RUN cargo build --release

FROM debian:buster-slim as collator
RUN apt-get update && apt-get install jq curl bash -y && \
    curl -sSo /wait-for-it.sh https://raw.githubusercontent.com/vishnubob/wait-for-it/master/wait-for-it.sh && \
    chmod +x /wait-for-it.sh && \
    curl -sL https://deb.nodesource.com/setup_12.x | bash - && \
    apt-get install -y nodejs && \
    npm install --global yarn && \
    yarn global add @polkadot/api-cli

COPY --from=builder \
    /builds/imbue/target/release/imbue /usr/bin

ENTRYPOINT ["/usr/bin/imbue"]
