/**
 * Copyright 2017 Intel Corporation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ------------------------------------------------------------------------------
 */

package handler

import (
	"encoding/hex"
	"fmt"
	"github.com/hyperledger/burrow/execution/errors"
	"github.com/hyperledger/burrow/execution/exec"
	"github.com/hyperledger/sawtooth-sdk-go/processor"
)

type SawtoothEventFireable struct {
	context *processor.Context
}

func NewSawtoothEventFireable(context *processor.Context) *SawtoothEventFireable {
	return &SawtoothEventFireable{
		context: context,
	}
}

func (evc *SawtoothEventFireable) Call(call *exec.CallEvent, exception *errors.Exception) error {
	return nil
}

func (evc *SawtoothEventFireable) Log(log *exec.LogEvent) error {
	attributes := []processor.Attribute{
		{
			Key:   "address",
			Value: hex.EncodeToString(log.Address.Bytes()),
		},
		{
			Key:   "eventID",
			Value: log.Address.String(),
		},
	}
	for i, topic := range log.Topics {
		attributes = append(attributes, processor.Attribute{
			Key:   fmt.Sprintf("topic%v", i+1),
			Value: hex.EncodeToString(topic.Bytes()),
		})
	}
	evc.context.AddEvent("seth_log_event", attributes, log.Data)

	return nil
}
