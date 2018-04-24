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

RUN echo "deb apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 8AA7AF1F1091A5FD" \
 && echo 'deb http://repo.sawtooth.me/ubuntu/1.0/stable xenial universe' >> /etc/apt/sources.list \
 && apt-get update \
 && apt-get install -y -q --allow-unauthenticated \
    python3-grpcio-tools=1.1.3-1 \
    python3-nose2 \
    python3-protobuf \
    python3-requests \
    python3-sawtooth-sdk \
    python3-zmq \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*

COPY . /project/sawtooth-seth

ENV PATH=$PATH:/project/sawtooth-seth/bin

RUN seth-protogen
