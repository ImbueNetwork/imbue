FROM arm64v8/node:16-slim AS builder

LABEL maintainer="imbue-dev"
ARG PROFILE=release
ARG IMBUE_GIT_REPO="https://github.com/ImbueNetwork/imbue.git"
ARG GIT_BRANCH="main"
ARG GIT_CLONE_DEPTH="--depth 1"
ENV DEBIAN_FRONTEND noninteractive

WORKDIR /builds
RUN apt-get upgrade
RUN apt-get update -y \
        && apt-get install -y --no-install-recommends ca-certificates wget curl git clang curl libssl-dev llvm libudev-dev bash make npm vim g++ python3.7

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup default stable \
        && rustup update \
        && rustup update nightly \
        && rustup target add wasm32-unknown-unknown --toolchain nightly

# #Build imbue-collator
RUN git clone --recursive ${IMBUE_GIT_REPO} ${GIT_CLONE_DEPTH}
WORKDIR /builds/imbue
RUN cargo build --${PROFILE}
RUN cp target/${PROFILE}/imbue-collator /

WORKDIR /builds/
RUN git clone --recursive https://github.com/paritytech/polkadot
WORKDIR /builds/polkadot
ARG POLKADOT_GIT_BRANCH="release-v0.9.13"
RUN git checkout $POLKADOT_GIT_BRANCH
RUN cargo build --${PROFILE}
RUN cp target/${PROFILE}/polkadot /


FROM arm64v8/node:16-slim
ENV DEBIAN_FRONTEND noninteractive
RUN apt-get update
RUN apt-get install -y \
        --no-install-recommends ca-certificates python3 wget git curl bash make vim g++  

RUN npm i polkadot-launch -g

COPY --from=builder /imbue-collator /
COPY --from=builder /polkadot /

EXPOSE 30330-30345 9933-9960 8080 3001