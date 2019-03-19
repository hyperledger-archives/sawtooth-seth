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

FROM ubuntu:bionic

RUN apt-get update \
 && apt-get install curl gnupg -y

ENV GOPATH=/project/sawtooth-seth/processor
ENV PATH=$PATH:/project/sawtooth-seth/processor/bin:/project/sawtooth-seth/bin:/usr/lib/go-1.10/bin
WORKDIR $GOPATH/src/seth_tp

RUN (curl -sSL 'http://keyserver.ubuntu.com/pks/lookup?op=get&search=0x44FC67F19B2466EA' | apt-key add - \
 || curl -sSL 'http://p80.pool.sks-keyservers.net/pks/lookup?op=get&search=0x44FC67F19B2466EA' | apt-key add - ) \
 && (curl -sSL 'http://keyserver.ubuntu.com/pks/lookup?op=get&search=0x308C15A29AD198E9' | apt-key add - \
 || curl -sSL 'http://p80.pool.sks-keyservers.net/pks/lookup?op=get&search=0x308C15A29AD198E9' | apt-key add - ) \
 && echo 'deb http://repo.sawtooth.me/ubuntu/nightly bionic universe' >> /etc/apt/sources.list \
 && echo 'deb http://ppa.launchpad.net/gophers/archive/ubuntu bionic main' >> /etc/apt/sources.list \
 && apt-get update \
 && apt-get install -y -q \
    git \
    golang-1.10-go \
    libssl-dev \
    libzmq3-dev \
    openssl \
    python3 \
    python3-grpcio-tools \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN git config --global url."https://".insteadOf git://

RUN go get -u \
    github.com/btcsuite/btcd/btcec \
    github.com/golang/mock/gomock \
    github.com/golang/mock/mockgen \
    github.com/golang/protobuf/proto \
    github.com/golang/protobuf/protoc-gen-go \
    github.com/jessevdk/go-flags \
    github.com/pebbe/zmq4 \
    github.com/satori/go.uuid \
    golang.org/x/crypto/ripemd160 \
    gopkg.in/fatih/set.v0

RUN git clone https://github.com/knkski/burrow.git $GOPATH/src/github.com/hyperledger/burrow

RUN go get github.com/hyperledger/sawtooth-sdk-go \
 && cd $GOPATH/src/github.com/hyperledger/sawtooth-sdk-go \
 && go generate

COPY . /project/sawtooth-seth

RUN seth-protogen go

ENV GOPATH=$GOPATH:/project/sawtooth-seth/burrow:/project/sawtooth-seth/common
RUN go build -o /project/sawtooth-seth/processor/bin/seth-tp
