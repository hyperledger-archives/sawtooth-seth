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

FROM ubuntu:xenial

ENV PATH=$PATH:/go/bin:/project/sawtooth-seth/bin:/project/sawtooth-seth/cli/bin:/project/sawtooth-seth/common/bin:/project/sawtooth-seth/processor/bin:/project/sawtooth-seth/rpc/bin:/protoc3/bin:/root/.cargo/bin:/usr/lib/go-1.9/bin

RUN (apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 8AA7AF1F1091A5FD \
 || apt-key adv --keyserver hkp://p80.pool.sks-keyservers.net:80 --recv-keys 8AA7AF1F1091A5FD) \
 && echo 'deb http://repo.sawtooth.me/ubuntu/1.0/stable xenial universe' >> /etc/apt/sources.list \
 && apt-get update \
 && apt-get install -y -q \
    curl \
    dvipng \
    gcc \
    git \
    golang-1.9-go \
    libssl-dev \
    libzmq3-dev \
    make \
    openssl \
    python3 \
    python3-grpcio-tools=1.1.3-1 \
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
 && rm -rf /var/lib/apt/lists/* \
 && pip3 install \
    sphinx==1.5.6 \
    sphinxcontrib-httpdomain \
    sphinxcontrib-openapi \
    sphinx_rtd_theme

ENV GOPATH=/go:/project/sawtooth-seth/common:/project/sawtooth-seth/burrow:/project/sawtooth-seth/cli:/project/sawtooth-seth/processor

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

RUN go get github.com/hyperledger/sawtooth-sdk-go \
 && cd /go/src/github.com/hyperledger/sawtooth-sdk-go \
 && go generate

RUN curl -OLsS https://github.com/google/protobuf/releases/download/v3.5.1/protoc-3.5.1-linux-x86_64.zip \
 && unzip protoc-3.5.1-linux-x86_64.zip -d protoc3 \
 && rm protoc-3.5.1-linux-x86_64.zip

ENV CARGO_INCREMENTAL=0

RUN curl https://sh.rustup.rs -sSf > /usr/bin/rustup-init \
 && chmod +x /usr/bin/rustup-init \
 && rustup-init -y

COPY . /project/sawtooth-seth

RUN seth-protogen go

WORKDIR /project/sawtooth-seth/cli/src/seth_cli/cli
RUN go build -o /project/sawtooth-seth/cli/bin/seth

WORKDIR /project/sawtooth-seth/processor/src/seth_tp
RUN go build -o /project/sawtooth-seth/processor/bin/seth-tp

WORKDIR /project/sawtooth-seth/rpc
RUN mkdir /project/sawtooth-seth/rpc/bin
RUN cargo build && cp ./target/debug/rpc /project/sawtooth-seth/rpc/bin/seth-rpc

WORKDIR /project/sawtooth-seth/docs
CMD make html latexpdf
