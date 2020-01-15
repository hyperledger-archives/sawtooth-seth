# Copyright 2018 Intel Corporation
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
#   Builds the environment needed to build the Sawtooth Seth docs
#   Running the image will put the Sawtooth Seth docs in
#   sawtooth-seth/docs/build on your local machine.
#
# Build:
#   $ cd sawtooth-seth
#   $ docker build . -f docs/Dockerfile -t seth-build-docs
#
# Run:
#   $ cd sawtooth-seth
#   $ docker run -v $(pwd)/docs:/project/sawtooth-seth/docs seth-build-docs

FROM ubuntu:bionic

RUN apt-get update \
 && apt-get install gnupg -y

ENV DEBIAN_FRONTEND=noninteractive

RUN (apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 44FC67F19B2466EA \
 || apt-key adv --keyserver hkp://p80.pool.sks-keyservers.net:80 --recv-keys 44FC67F19B2466EA) \
 && (apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 308C15A29AD198E9 \
 || apt-key adv --keyserver hkp://p80.pool.sks-keyservers.net:80 --recv-keys 308C15A29AD198E9) \
 && echo 'deb http://repo.sawtooth.me/ubuntu/nightly bionic universe' >> /etc/apt/sources.list \
 && echo 'deb http://ppa.launchpad.net/gophers/archive/ubuntu bionic main' >> /etc/apt/sources.list \
 && apt-get update \
 && apt-get install -y -q \
    curl \
    dvipng \
    gcc \
    git \
    golang-1.11-go \
    libssl-dev \
    libzmq3-dev \
    make \
    openssl \
    python3 \
    python3-grpcio-tools \
    python3-pip \
    python3-sawtooth-cli \
    software-properties-common \
    sudo \
    texlive-fonts-recommended \
    texlive-latex-base \
    texlive-latex-extra \
    texlive-latex-recommended \
    unzip \
    zip \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN pip3 install sphinx==1.5.6 sphinx_rtd_theme

ENV PATH=$PATH:/go/bin:/project/sawtooth-seth/bin:/project/sawtooth-seth/cli-go/bin:/project/sawtooth-seth/common/bin:/project/sawtooth-seth/processor/bin:/project/sawtooth-seth/rpc/bin:/protoc3/bin:/root/.cargo/bin:/usr/lib/go-1.11/bin
ENV GOPATH=/go:/project/sawtooth-seth/common:/project/sawtooth-seth/burrow:/project/sawtooth-seth/cli-go:/project/sawtooth-seth/processor

RUN go get -u \
    github.com/btcsuite/btcd/btcec \
    github.com/golang/mock/gomock \
    github.com/golang/mock/mockgen \
    github.com/golang/protobuf/proto \
    github.com/golang/protobuf/protoc-gen-go \
    github.com/jessevdk/go-flags \
    github.com/pebbe/zmq4 \
    github.com/pelletier/go-toml \
    github.com/satori/go.uuid \
    golang.org/x/crypto/ripemd160 \
    golang.org/x/crypto/ssh/terminal \
    gopkg.in/fatih/set.v0

RUN git clone https://github.com/knkski/burrow.git /go/src/github.com/hyperledger/burrow

RUN go get github.com/hyperledger/sawtooth-sdk-go

ENV GOPATH=/go/src/github.com/hyperledger/sawtooth-sdk-go:$GOPATH

RUN curl -OLsS https://github.com/google/protobuf/releases/download/v3.5.1/protoc-3.5.1-linux-x86_64.zip \
 && unzip protoc-3.5.1-linux-x86_64.zip -d protoc3 \
 && rm protoc-3.5.1-linux-x86_64.zip

ENV CARGO_INCREMENTAL=0

RUN curl https://sh.rustup.rs -sSf > /usr/bin/rustup-init \
 && chmod +x /usr/bin/rustup-init \
 && rustup-init -y

COPY . /project/sawtooth-seth

RUN seth-protogen go

WORKDIR /project/sawtooth-seth/cli-go/src/seth_cli/cli
RUN go build -o /project/sawtooth-seth/cli-go/bin/seth

WORKDIR /project/sawtooth-seth/processor/src/seth_tp
RUN go build -o /project/sawtooth-seth/processor/bin/seth-tp

WORKDIR /project/sawtooth-seth
RUN cargo build && cp ./target/debug/seth-rpc /project/sawtooth-seth/bin/seth-rpc && cargo doc --no-deps

WORKDIR /project/sawtooth-seth/docs
CMD make html latexpdf
