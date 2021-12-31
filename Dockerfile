FROM paritytech/ci-linux:staging AS builder
LABEL maintainer="imbue-dev"
WORKDIR /builds/imbue

ADD libs libs
ADD node node
ADD pallets pallets
ADD runtime runtime
ADD Cargo.* ./

RUN cargo build --release
RUN cp target/release/imbue-collator /


FROM parity/polkadot:v0.9.13 AS polkadot
FROM parity/subkey:latest AS subkey
# to copy polkadot binaries only; no other directives required


FROM alpine:latest

RUN apk update && apk add ca-certificates

COPY --from=builder /imbue-collator /
COPY --from=polkadot /usr/bin/polkadot /
COPY --from=subkey /usr/local/bin/subkey /

EXPOSE 30330-30345 9933-9960 8080 3001
