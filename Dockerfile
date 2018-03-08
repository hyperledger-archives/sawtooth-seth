# Copyright 2017 Intel Corporation
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
# ------------------------------------------------------------------------------

# Description:
#   Builds an image to be used when developing for Sawtooth Seth. Should be
#   run from the root of the checked out GitHub repository.
#
# Build:
#   $ docker build . -t sawtooth-dev-seth
#
# Run:
#   $ docker run -v $(pwd):/project/sawtooth-seth sawtooth-dev-seth

FROM ubuntu:xenial

LABEL "install-type"="mounted"

# Install languages and dependencies available as debs
RUN echo "deb http://repo.sawtooth.me/ubuntu/ci xenial universe" >> /etc/apt/sources.list \
 && apt-key adv --keyserver keyserver.ubuntu.com --recv-keys 8AA7AF1F1091A5FD \
 && apt-get update \
 && apt-get install -y -q --allow-downgrades \
    build-essential \
    ca-certificates \
    curl \
    gcc \
    git \
    libssl-dev \
    libzmq3-dev \
    openssl \
    pkg-config \
    python3-grpcio-tools=1.1.3-1 \
    software-properties-common \
 && add-apt-repository ppa:ethereum/ethereum \
 && add-apt-repository ppa:gophers/archive \
 && apt-get update \
 && apt-get install -y -q \
    golang-1.9-go \
    solc \
 && curl -s -S -o /tmp/setup-node.sh https://deb.nodesource.com/setup_6.x \
 && chmod 755 /tmp/setup-node.sh \
 && /tmp/setup-node.sh \
 && apt-get install nodejs -y -q \
 && rm /tmp/setup-node.sh \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

ENV PATH=$PATH:/project/sawtooth-seth/families/seth/bin:/root/.cargo/bin:/go/bin:/usr/lib/go-1.9/bin
ENV GOPATH=/go:/project/sawtooth-seth/families/seth

# Install rust libraries
RUN curl https://sh.rustup.rs -sSf > /usr/bin/rustup-init \
 && chmod +x /usr/bin/rustup-init \
 && rustup-init -y \
 && cargo install protobuf

# Install go libraries
RUN mkdir /go \
 && go get -u \
    github.com/golang/protobuf/proto \
    github.com/golang/protobuf/protoc-gen-go \
    github.com/pebbe/zmq4 \
    github.com/brianolson/cbor_go \
    github.com/satori/go.uuid \
    github.com/btcsuite/btcd/btcec \
    github.com/btcsuite/btcutil/base58 \
    gopkg.in/fatih/set.v0 \
    golang.org/x/crypto/ripemd160 \
    github.com/jessevdk/go-flags \
    github.com/pelletier/go-toml \
    golang.org/x/crypto/ssh/terminal

# Install javascript stuff
RUN npm install \
    ethereumjs-abi \
    web3

RUN mkdir -p /project/sawtooth-seth/ \
 && mkdir -p /var/log/sawtooth \
 && mkdir -p /var/lib/sawtooth \
 && mkdir -p /etc/sawtooth \
 && mkdir -p /etc/sawtooth/keys

WORKDIR /project/sawtooth-seth

EXPOSE 3030/tcp

ENV PATH=$PATH:/project/sawtooth-seth/bin

CMD build_seth
