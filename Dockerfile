FROM paritytech/ci-linux:staging AS builder
ARG PROFILE=release
WORKDIR /imbue
ADD . .
RUN cargo build --${PROFILE}


FROM parity/polkadot:latest AS polkadot
# to copy polkadot binaries only; no other directives required


FROM node:16
ARG PROFILE=release
ENV DEBIAN_FRONTEND noninteractive
RUN apt-get update -y\
    && apt-get install curl ca-certificates vim awscli -y

RUN yarn global add polkadot-launch

COPY --from=builder /imbue/target/${PROFILE}/imbue-collator /
COPY --from=polkadot /polkadot /
COPY --from=polkadot /subkey /

EXPOSE 30330-30345 9933-9960 8080 3001
