FROM samelamin/imbue AS imbue

LABEL maintainer="imbue-dev"

FROM node:16-slim
ARG APT_PACKAGES
ENV DEBIAN_FRONTEND noninteractive
RUN apt-get update
RUN apt-get install -y\
        gnupg2 ca-certificates\
        ${APT_PACKAGES}

COPY --from=imbue /imbue-collator /
COPY --from=imbue /polkadot /
COPY --from=imbue /subkey /

RUN yarn global add polkadot-launch

EXPOSE 30330-30345 9933-9960 8080 3001
