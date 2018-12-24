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

FROM ubuntu:xenial

ENV GOPATH=/project/sawtooth-seth/cli-go
ENV PATH=$PATH:/project/sawtooth-seth/cli-go/bin:/project/sawtooth-seth/bin:/usr/lib/go-1.10/bin

RUN \
 if [ ! -z $HTTP_PROXY ] && [ -z $http_proxy ]; then \
  http_proxy=$HTTP_PROXY; \
 fi; \
 if [ ! -z $HTTPS_PROXY ] && [ -z $https_proxy ]; then \
  https_proxy=$HTTPS_PROXY; \
 fi

RUN (apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 8AA7AF1F1091A5FD 9AD198E9 \
 || apt-key adv --keyserver hkp://p80.pool.sks-keyservers.net:80 --recv-keys 8AA7AF1F1091A5FD 9AD198E9) \
 && echo 'deb http://repo.sawtooth.me/ubuntu/bumper/stable xenial universe' >> /etc/apt/sources.list \
 && echo 'deb http://ppa.launchpad.net/gophers/archive/ubuntu xenial main' >> /etc/apt/sources.list \
 && apt-get update \
 && apt-get install -y -q \
    curl \
    git \
    golang-1.10-go \
    libssl-dev \
    libzmq3-dev \
    openssl \
    python3 \
    python3-grpcio-tools=1.1.3-1 \
    python3-sawtooth-cli \
    software-properties-common \
 && add-apt-repository -k hkp://p80.pool.sks-keyservers.net:80 ppa:ethereum/ethereum \
 && apt-get update \
 && apt-get install -y -q \
    solc \
 && curl -s -S -o /tmp/setup-node.sh https://deb.nodesource.com/setup_6.x \
 && chmod 755 /tmp/setup-node.sh \
 && /tmp/setup-node.sh \
 && apt-get install nodejs -y -q \
 && rm /tmp/setup-node.sh \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

RUN \
 if [ ! -z $http_proxy ]; then \
  npm config set proxy $http_proxy; \
  git config --global http.proxy $http_proxy; \
 fi; \
 if [ ! -z $https_proxy ]; then \
  npm config set https-proxy $https_proxy; \
  git config --global https.proxy $https_proxy; \
 fi

RUN git config --global url."https://".insteadOf git://

######################################################################
RUN ls
RUN git clone -v --progress https://github.com/hyperledger/burrow.git -b master /burrow-master
RUN go get -u -v github.com/hyperledger/burrow/binary
RUN cp -r /burrow-master/execution/evm/sha3 $GOPATH/src/github.com/hyperledger/burrow/execution/evm/sha3
RUN ls
######################################################################

RUN go get -u -v\
    github.com/btcsuite/btcd/btcec \
    github.com/golang/mock/gomock \
    github.com/golang/mock/mockgen \
    github.com/golang/protobuf/proto \
    github.com/golang/protobuf/protoc-gen-go \
    github.com/hyperledger/burrow/binary \
	github.com/hyperledger/burrow/execution/evm/sha3 \
    github.com/jessevdk/go-flags \
    github.com/pebbe/zmq4 \
    github.com/pelletier/go-toml \
    github.com/satori/go.uuid \
    golang.org/x/crypto/ssh/terminal

RUN go get -v github.com/hyperledger/sawtooth-sdk-go \
 && cd $GOPATH/src/github.com/hyperledger/sawtooth-sdk-go \
 && go generate

RUN npm install \
    ethereumjs-abi \
    web3

COPY . /project/sawtooth-seth

RUN seth-protogen go

WORKDIR $GOPATH/src/seth_cli/cli
ENV GOPATH=$GOPATH:/project/sawtooth-seth/burrow:/project/sawtooth-seth/common
RUN go build -o /project/sawtooth-seth/cli-go/bin/seth
