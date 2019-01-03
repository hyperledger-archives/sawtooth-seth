/**
 * Copyright 2017 Intel Corporation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *		 http://www.apache.org/licenses/LICENSE-2.0
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
	. "common"
	"encoding/hex"
	"fmt"
	"github.com/hyperledger/burrow/acm"
	"github.com/hyperledger/burrow/crypto"
	"github.com/hyperledger/burrow/execution/evm"
	"github.com/hyperledger/burrow/permission"
	"github.com/hyperledger/sawtooth-sdk-go/processor"
	. "protobuf/seth_pb2"
	"strings"
)

var TxnHandlers = map[SethTransaction_TransactionType]TransactionHandler{
	SethTransaction_CREATE_EXTERNAL_ACCOUNT: CreateExternalAccount,
	SethTransaction_CREATE_CONTRACT_ACCOUNT: CreateContractAccount,
	SethTransaction_MESSAGE_CALL:            MessageCall,
	SethTransaction_SET_PERMISSIONS:         SetPermissions,
}

func CreateExternalAccount(wrapper *SethTransaction, sender *EvmAddr, sapps *SawtoothAppState) HandlerResult {
	txn := wrapper.GetCreateExternalAccount()
	var newAcct *acm.Account

	// Sender is creating a separate external account, this is only possible
	// when gas is free and the sender has permission to create accounts
	if txn.GetTo() != nil {
		// The creating account must exist and have permission to create accounts
		senderAcct, err := sapps.GetAccount(crypto.AddressFromWord256(sender.ToWord256()))
		if senderAcct == nil {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
					"Creating account must already exist for it to be able to create other accounts: %v",
					sender,
				)},
			}
		}
		if !evm.HasPermission(sapps, senderAcct.Address, permission.CreateAccount) {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
					"Sender account does not have permission to create external accounts: %v",
					sender,
				)},
			}
		}
		// Check that the nonce in the transaction matches the nonce in state
		if txn.GetNonce() != senderAcct.Sequence {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
					"Nonces do not match: Transaction (%v), State (%v)",
					txn.GetNonce(), senderAcct.Sequence,
				)},
			}
		}

		// Get the address of the account to create
		newAcctAddr, err := NewEvmAddrFromBytes(txn.GetTo())
		if err != nil {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
					"Failed to construct address for new EOA: %v", txn.GetTo(),
				)},
			}
		}

		logger.Debugf("Creating new EOA on behalf of %v", newAcctAddr)

		// The new account must not already exist
		existingAcct, err := sapps.GetAccount(crypto.AddressFromWord256(newAcctAddr.ToWord256()))
		if existingAcct != nil {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
					"Account already exists at address %v", newAcctAddr,
				)},
			}
		}
		if err != nil {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
					"Error while checking for account at address %v", newAcctAddr,
				)},
			}
		}

		// If no permissions were passed by the transaction, inherit them from
		// sender. Otherwise, set them from transaction.
		var newPerms permission.AccountPermissions
		if txn.GetPermissions() == nil {
			newPerms = senderAcct.Permissions
			newPerms.Base.Set(permission.Root, false)

		} else {
			if !evm.HasPermission(sapps, senderAcct.Address, permission.Root) {
				return HandlerResult{
					Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
						"Creating account does not have permission to set permissions: %v",
						sender,
					)},
				}
			}
			newPerms = toVmPermissions(txn.GetPermissions())
		}

		// Create new account
		newAcct = &acm.Account{
			Address:     crypto.AddressFromWord256(newAcctAddr.ToWord256()),
			Sequence:    1,
			Permissions: newPerms,
		}

		senderAcct.Sequence += 1

		// Update accounts in state
		err = sapps.UpdateAccount(senderAcct)
		if err != nil {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: err.Error()},
			}
		}
		err = sapps.UpdateAccount(newAcct)
		if err != nil {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: err.Error()},
			}
		}

		// Sender is new and is creating this account for the first time
	} else {
		logger.Debugf("Creating new EOA at sender address: %v", sender)

		// The new account must not already exist
		senderAcct, err := sapps.GetAccount(crypto.AddressFromWord256(sender.ToWord256()))
		if err != nil {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
					"Failed to get sender account: %s", err,
				)},
			}
		}
		if senderAcct != nil {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
					"Account already exists at address %v", sender,
				)},
			}
		}

		// Check global permissions to decide if the account can be created
		global, err := sapps.GetAccount(acm.GlobalPermissionsAddress)
		if err != nil {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
					"Failed to get global permissions address: %s", err,
				)},
			}
		}

		if !evm.HasPermission(sapps, global.Address, permission.CreateAccount) {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
					"New account creation is disabled, couldn't create account: %v",
					sender,
				)},
			}
		}

		newAcct = &acm.Account{
			Address:     crypto.AddressFromWord256(sender.ToWord256()),
			Sequence:    1,
			Permissions: global.Permissions,
		}

		err = sapps.UpdateAccount(newAcct)
		if err != nil {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
					"Failed to update account: %s", err,
				)},
			}
		}
	}

	return HandlerResult{
		NewAccount: newAcct,
	}
}

func CreateContractAccount(wrapper *SethTransaction, sender *EvmAddr, sapps *SawtoothAppState) HandlerResult {
	txn := wrapper.GetCreateContractAccount()

	// The creating account must already exist
	senderAcct, err := sapps.GetAccount(crypto.AddressFromWord256(sender.ToWord256()))
	if err != nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Couldn't get sender account: %s", err,
			)},
		}
	}
	if senderAcct == nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Creating account must already exist to create contract account: %v",
				sender,
			)},
		}
	}

	// Verify this account has permission to create contract accounts
	if !evm.HasPermission(sapps, senderAcct.Address, permission.CreateContract) {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Sender account does not have permission to create contracts: %v",
				sender,
			)},
		}
	}

	// Check that the nonce in the transaction matches the nonce in state
	if txn.GetNonce() != senderAcct.Sequence {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Nonces do not match: Transaction (%v), State (%v)",
				txn.GetNonce(), senderAcct.Sequence,
			)},
		}
	}

	var newPerms permission.AccountPermissions
	if txn.GetPermissions() == nil {
		newPerms = senderAcct.Permissions
		newPerms.Base.Set(permission.Root, false)

	} else {
		if !evm.HasPermission(sapps, senderAcct.Address, permission.Root) {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
					"Creating account does not have permission to set permissions: %v",
					sender,
				)},
			}
		}
		newPerms = toVmPermissions(txn.GetPermissions())
	}

	// Create the new account
	// NOTE: The senderAcct's nonce will be incremented
	addrBytes := senderAcct.Address.Bytes()
	creatorAddress, err := NewEvmAddrFromBytes(addrBytes)
	if err != nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Could not convert evm address: %v",
				err,
			)},
		}
	}
	logger.Debugf("CreateAccount(%v)", creatorAddress)

	// Get address of new account
	newAddress := creatorAddress.Derive(uint64(senderAcct.Sequence))

	// Increment nonce
	senderAcct.Sequence += 1
	sapps.CreateAccount(crypto.MustAddressFromBytes(newAddress.Bytes()))
	newAcct, err := sapps.GetAccount(senderAcct.Address)
	if err != nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Could not get account: %v",
				err,
			)},
		}
	}

	// Initialize the new account
	out, gasUsed, err := callVm(sapps, newAcct, nil, txn.GetInit(), nil, txn.GetGasLimit())
	if err != nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Error while calling VM: %v",
				sender,
			)},
		}
	}

	newAcct.Sequence += 1
	newAcct.Code = out
	newAcct.Permissions = newPerms

	// Update accounts in state
	err = sapps.UpdateAccount(senderAcct)
	if err != nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Error Updating sender account: %v",
				sender,
			)},
		}
	}
	err = sapps.UpdateAccount(newAcct)
	if err != nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Error updating new account: %v",
				sender,
			)},
		}
	}

	return HandlerResult{
		GasUsed:     gasUsed,
		ReturnValue: out,
		NewAccount:  newAcct,
	}
}

func MessageCall(wrapper *SethTransaction, sender *EvmAddr, sapps *SawtoothAppState) HandlerResult {
	txn := wrapper.GetMessageCall()

	// The sender account must already exist
	senderAcct, err := sapps.GetAccount(crypto.AddressFromWord256(sender.ToWord256()))
	if senderAcct == nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Sender account must already exist to message call: %v", sender,
			)},
		}
	}

	// Verify this account has permission to make message calls
	if !evm.HasPermission(sapps, senderAcct.Address, permission.Call) {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Sender account does not have permission to make message calls: %v",
				sender,
			)},
		}
	}

	// Check that the nonce in the transaction matches the nonce in state
	if txn.GetNonce() != senderAcct.Sequence {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Nonces do not match: Transaction (%v), State (%v)",
				txn.GetNonce(), senderAcct.Sequence,
			)},
		}
	}

	receiver, err := NewEvmAddrFromBytes(txn.GetTo())
	if err != nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Failed to construct receiver address for message call: %v", txn.GetTo(),
			)},
		}
	}

	receiverAcct, err := sapps.GetAccount(crypto.AddressFromWord256(receiver.ToWord256()))
	if err != nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Error while retrieving receiver account: %v", err,
			)},
		}
	}

	// Receiving account must exist to call it
	if receiverAcct == nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Receiver account must already exist to call it: %v", receiver,
			)},
		}
	}

	// Execute the contract
	out, gasUsed, err := callVm(
		sapps,
		senderAcct,
		receiverAcct,
		receiverAcct.Code.Bytes(),
		txn.GetData(),
		txn.GetGasLimit(),
	)

	if err != nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: err.Error()},
		}
	}
	logger.Debug("Gas Used: ", gasUsed)
	logger.Debug("EVM Output: ", strings.ToLower(hex.EncodeToString(out)))

	senderAcct.Sequence += 1

	sapps.UpdateAccount(senderAcct)
	sapps.UpdateAccount(receiverAcct)

	return HandlerResult{
		ReturnValue: out,
		GasUsed:     gasUsed,
	}
}

func SetPermissions(wrapper *SethTransaction, sender *EvmAddr, sapps *SawtoothAppState) HandlerResult {
	txn := wrapper.GetSetPermissions()

	if txn.GetPermissions() == nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{
				Msg: "Permissions field cannot be blank in UpdatePermissions transaction",
			},
		}
	}
	newPerms := toVmPermissions(txn.GetPermissions())

	// Get the account that is trying to update permissions
	senderAcct, err := sapps.GetAccount(crypto.AddressFromWord256(sender.ToWord256()))
	if senderAcct == nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Sender account must already exist for updating permissions: %v", sender,
			)},
		}
	}

	// Verify this account has permission to update permissions
	if !evm.HasPermission(sapps, senderAcct.Address, permission.Root) {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Sender account does not have permission to change permissions: %v",
				sender,
			)},
		}
	}

	// Check that the nonce in the transaction matches the nonce in state
	if txn.GetNonce() != senderAcct.Sequence {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Nonces do not match: Transaction (%v), State (%v)",
				txn.GetNonce(), senderAcct.Sequence,
			)},
		}
	}

	receiver, err := NewEvmAddrFromBytes(txn.GetTo())
	if err != nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Failed to construct receiver address for permission change: %v",
				txn.GetTo(),
			)},
		}
	}

	logger.Debugf(
		"SetPermissions(%v): Perms(%v), SetBit(%v)\n", receiver,
		newPerms.Base.Perms, newPerms.Base.SetBit,
	)

	receiverWord256 := crypto.AddressFromWord256(receiver.ToWord256())
	receiverAcct, err := sapps.GetAccount(receiverWord256)
	if err != nil {
		return HandlerResult{
			Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
				"Error while retrieving receiver account: %v", err,
			)},
		}
	}
	if receiverAcct == nil {
		if receiverWord256 == acm.GlobalPermissionsAddress {
			receiverAcct = &acm.Account{
				Address:  receiverWord256,
				Sequence: 1,
			}
		} else {
			return HandlerResult{
				Error: &processor.InvalidTransactionError{Msg: fmt.Sprintf(
					"Receiver account must already exist to change its permissions: %v",
					receiver,
				)},
			}
		}
	}

	// Update accounts
	senderAcct.Sequence += 1
	receiverAcct.Permissions = newPerms

	sapps.UpdateAccount(senderAcct)
	sapps.UpdateAccount(receiverAcct)

	return HandlerResult{}
}
