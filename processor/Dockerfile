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

FROM ubuntu:xenial

COPY . /project/sawtooth-seth
ENV GOPATH=/project/sawtooth-seth/processor
ENV PATH=$PATH:/project/sawtooth-seth/processor/bin:/project/sawtooth-seth/bin:/usr/lib/go-1.9/bin
WORKDIR $GOPATH/src/seth_tp

RUN apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 8AA7AF1F1091A5FD \
 && echo 'deb http://repo.sawtooth.me/ubuntu/1.0/stable xenial universe' >> /etc/apt/sources.list \
 && apt-get update \
 && apt-get install -y -q \
    git \
    golang-1.9-go \
    libssl-dev \
    libzmq3-dev \
    openssl \
    python3 \
    python3-grpcio-tools=1.1.3-1 \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN go get -u \
    github.com/golang/protobuf/proto \
    github.com/golang/protobuf/protoc-gen-go \
    github.com/btcsuite/btcd/btcec \
    github.com/grkvlt/sawtooth-go-sdk/... \
    gopkg.in/fatih/set.v0 \
    golang.org/x/crypto/ripemd160 \
    github.com/jessevdk/go-flags

RUN seth-protogen go

ENV GOPATH=$GOPATH:/project/sawtooth-seth/burrow:/project/sawtooth-seth/common
RUN go build -o /project/sawtooth-seth/processor/bin/seth-tp
