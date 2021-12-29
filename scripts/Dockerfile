FROM samelamin/polkadot:0.9.15 as builder

LABEL maintainer="imbue-dev"
ARG PROFILE=release
ARG IMBUE_GIT_REPO="https://github.com/ImbueNetwork/imbue.git"
ARG GIT_BRANCH="main"
ARG GIT_CLONE_DEPTH="--depth 1"
WORKDIR /builds/

#Build imbue-collator
RUN git clone --recursive ${IMBUE_GIT_REPO} ${GIT_CLONE_DEPTH}

WORKDIR /builds/imbue
RUN git checkout ${GIT_BRANCH}
RUN cargo build --${PROFILE}
RUN cp target/${PROFILE}/imbue-collator /imbue-collator

FROM ubuntu
ENV DEBIAN_FRONTEND noninteractive
RUN apt update && apt install git gnupg2 curl ca-certificates vim npm nodejs wget awscli -y
RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add - && echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list

WORKDIR /builds/
RUN apt update && apt install yarn -y
RUN apt purge nodejs npm -y

RUN apt-get install -y \
    software-properties-common \
    npm
RUN npm install npm@latest -g && \
    npm install n -g && \
    n stable

ARG POLKADOT_LAUNCH_REPO="https://github.com/ImbueNetwork/polkadot-launch.git"
RUN git clone --recursive ${POLKADOT_LAUNCH_REPO} ${GIT_CLONE_DEPTH}
WORKDIR /builds/polkadot-launch
RUN git checkout dcfb2295
RUN yarn
RUN yarn install
RUN yarn global add polkadot-launch@1.9.0
# RUN npm i polkadot-launch -g

WORKDIR /
COPY --from=builder /imbue-collator /
COPY --from=builder /polkadot /
COPY --from=builder /subkey /

EXPOSE 30330-30345 9933-9960 8080 3001