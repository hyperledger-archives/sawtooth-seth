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
	. "common"
	"fmt"
	"github.com/hyperledger/burrow/acm"
	"github.com/hyperledger/burrow/acm/state"
	"github.com/hyperledger/burrow/binary"
	"github.com/hyperledger/burrow/crypto"
	"github.com/hyperledger/burrow/execution/errors"
	"github.com/hyperledger/burrow/execution/evm"
	"github.com/hyperledger/burrow/permission"
	"github.com/hyperledger/sawtooth-sdk-go/processor"
	. "protobuf/seth_pb2"
)

// -- AppState --

// SawtoothAppState implements the interface used by the Burrow EVM to
// access global state
type SawtoothAppState struct {
	mgr   *StateManager
	error errors.CodedError
}

func NewSawtoothAppState(state *processor.Context) *SawtoothAppState {
	return &SawtoothAppState{
		mgr: NewStateManager(state),
	}
}

// GetAccount retrieves an existing account with the given address. Returns nil
// if the account doesn't exist.
func (s *SawtoothAppState) GetAccount(addr crypto.Address) (*acm.Account, error) {
	addrBytes := addr.Bytes()
	vmAddress, err := NewEvmAddrFromBytes(addrBytes)
	if err != nil {
		return nil, err
	}
	logger.Debugf("GetAccount(%v)", vmAddress)

	entry, err := s.mgr.GetEntry(vmAddress)
	if err != nil {
		return nil, err
	}
	if entry == nil {
		return nil, nil
	}

	return toVmAccount(entry.GetAccount()), nil
}

// UpdateAccount updates the account in state. Creates the account if it doesn't
// exist yet.
func (s *SawtoothAppState) UpdateAccount(acct *acm.Account) error {
	addrBytes := acct.Address.Bytes()
	vmAddress, err := NewEvmAddrFromBytes(addrBytes)
	if err != nil {
		return err
	}

	logger.Debugf("UpdateAccount(%v)", vmAddress)

	entry, err := s.mgr.GetEntry(vmAddress)
	if err != nil {
		return err
	}

	if entry == nil {
		entry, err = s.mgr.NewEntry(vmAddress)
		if err != nil {
			return err
		}
	}

	entry.Account = toStateAccount(acct)

	s.mgr.MustSetEntry(vmAddress, entry)

	return nil
}

// RemoveAccount removes the account and associated storage from global state
// and panics if it doesn't exist.
func (s *SawtoothAppState) RemoveAccount(acct crypto.Address) {
	addrBytes := acct.Bytes()
	vmAddress, err := NewEvmAddrFromBytes(addrBytes)
	if err != nil {
		panic(fmt.Sprintf("Error while removing account: %v", err))
	}
	logger.Debugf("RemoveAccount(%v)", vmAddress)

	err = s.mgr.DelEntry(vmAddress)
	if err != nil {
		panic(fmt.Sprintf(
			"Tried to DelEntry(%v) but nothing exists there", vmAddress,
		))
	}
}

// CreateAccount creates a new Contract Account using the given existing
// account to generate a new address. panics if the given account doesn't exist
// or the address of the newly created account conflicts with an existing
// account.
func (s *SawtoothAppState) CreateAccount(address crypto.Address) {
	addrBytes := address.Bytes()
	vmAddress, err := NewEvmAddrFromBytes(addrBytes)
	if err != nil {
		panic(fmt.Sprintf("Error while creating account: %v", err))
	}
	logger.Debugf("CreateAccount(%v)", vmAddress)

	// If it already exists, something has gone wrong
	_, err = s.mgr.NewEntry(vmAddress)
	if err != nil {
		panic(fmt.Sprintf(
			"Failed to NewEntry(%v): %v", address, err.Error(),
		))
	}
}

// GetStorage gets the 256 bit value stored with the given key in the given
// account, returns zero if the key does not exist.
func (s *SawtoothAppState) GetStorage(address crypto.Address, key binary.Word256) binary.Word256 {
	addrBytes := address.Bytes()
	vmAddress, err := NewEvmAddrFromBytes(addrBytes)
	if err != nil {
		panic(fmt.Sprintf("Error while getting storage: %v", err))
	}
	logger.Debugf("GetStorage(%v, %v)", vmAddress, key.Bytes())

	// Load the entry from global state
	entry := s.mgr.MustGetEntry(vmAddress)

	storage := entry.GetStorage()

	for _, pair := range storage {
		k := binary.LeftPadWord256(pair.GetKey())
		if k.Compare(key) == 0 {
			return binary.LeftPadWord256(pair.GetValue())
		}
	}

	logger.Debugf("Key %v not set for account %v", key.Bytes(), vmAddress)
	return binary.Zero256
}

func (s *SawtoothAppState) SetStorage(address crypto.Address, key, value binary.Word256) {
	addrBytes := address.Bytes()
	vmAddress, err := NewEvmAddrFromBytes(addrBytes)
	if err != nil {
		panic(fmt.Sprintf("Error while setting storage: %v", err))
	}
	logger.Debugf("SetStorage(%v, %v, %v)", vmAddress, key.Bytes(), value.Bytes())

	entry := s.mgr.MustGetEntry(vmAddress)

	storage := entry.GetStorage()

	// Make sure we update the entry after changing it
	defer func() {
		entry.Storage = storage
		logger.Debugf("Storage")
		for _, pair := range entry.GetStorage() {
			logger.Debugf("%v -> %v", pair.GetKey(), pair.GetValue())
		}
		s.mgr.MustSetEntry(vmAddress, entry)
	}()

	for _, pair := range storage {
		k := binary.LeftPadWord256(pair.GetKey())

		// If the key has already been set, overwrite it
		if k.Compare(key) == 0 {
			pair.Value = value.Bytes()
		}
	}

	// If the key is new, append it
	storage = append(storage, &EvmStorage{
		Key:   key.Bytes(),
		Value: value.Bytes(),
	})
}

func (s *SawtoothAppState) GetBlockHash(blockNumber int64) (binary.Word256, error) {
	blockInfo, err := getBlockInfo(s.mgr.state, blockNumber)
	if err != nil {
		return binary.Zero256, fmt.Errorf("Failed to get block info: %v", err.Error())
	}

	hash, err := StringToWord256(blockInfo.GetHeaderSignature())
	if err != nil {
		return binary.Zero256, fmt.Errorf("Failed to get block info: %v", err.Error())
	}

	return hash, nil
}

func (s *SawtoothAppState) AddRole(address crypto.Address, foo string) bool {
	panic("AddRole not supported!")
}

func (s *SawtoothAppState) RemoveRole(address crypto.Address, role string) bool {
	panic("RemoveRole not supported!")
}

func (s *SawtoothAppState) AddToBalance(address crypto.Address, foo uint64) {
	panic("AddToBalance not supported!")
}

func (s *SawtoothAppState) SubtractFromBalance(address crypto.Address, amount uint64) {
	panic("SubtractFromBalance not supported!")
}

func (s *SawtoothAppState) GetBalance(address crypto.Address) uint64 {
	panic("GetBalance not supported!")
}

func (s *SawtoothAppState) GetCode(address crypto.Address) acm.Bytecode {
	panic("GetCode not supported!")
}

func (s *SawtoothAppState) InitCode(address crypto.Address, code []byte) {
	panic("InitCode not supported!")
}

func (s *SawtoothAppState) GetSequence(address crypto.Address) uint64 {
	panic("GetSequence not supported!")
}

func (s *SawtoothAppState) Exists(address crypto.Address) bool {
	panic("AddToBalance not supported!")
}

func (s *SawtoothAppState) GetPermissions(address crypto.Address) permission.AccountPermissions {
	acct, err := s.GetAccount(address)
	if err != nil {
		return permission.AccountPermissions{}
	}

	if acct == nil {
		return permission.AccountPermissions{}
	}

	return acct.Permissions
}

func (s *SawtoothAppState) SetPermission(address crypto.Address, permFlag permission.PermFlag, value bool) {
	panic("SetPermission not supported!")
}

func (s *SawtoothAppState) UnsetPermission(address crypto.Address, permFlag permission.PermFlag) {
	panic("UnsetPermission not supported!")
}

func (s *SawtoothAppState) NewCache(cacheOptions ...state.CacheOption) evm.Interface {
	panic("NewCache not supported!")
}

func (s *SawtoothAppState) Sync() errors.CodedError {
	panic("Sync not supported!")
}

func (s *SawtoothAppState) PushError(err error) {
	if s.error == nil {
		// Make sure we are not wrapping a known nil value
		ex := errors.AsException(err)
		if ex != nil {
			ex.Exception = fmt.Sprintf("%s", ex.Exception)
			s.error = ex
		}
	}
}

func (s *SawtoothAppState) Error() errors.CodedError {
	if s.error == nil {
		return nil
	}
	return s.error
}

// -- Utilities --

func toStateAccount(acct *acm.Account) *EvmStateAccount {
	if acct == nil {
		return nil
	}
	return &EvmStateAccount{
		Address:     acct.Address.Bytes(),
		Balance:     int64(acct.Balance),
		Code:        acct.Code,
		Nonce:       acct.Sequence,
		Permissions: toStatePermissions(acct.Permissions),
	}
}

func toVmAccount(sa *EvmStateAccount) *acm.Account {
	if sa == nil {
		return nil
	}
	return &acm.Account{
		Address:     crypto.MustAddressFromBytes(sa.Address),
		Balance:     uint64(sa.Balance),
		Code:        sa.Code,
		Sequence:    sa.Nonce,
		Permissions: toVmPermissions(sa.Permissions),
	}
}

func toStatePermissions(aPerm permission.AccountPermissions) *EvmPermissions {
	return &EvmPermissions{
		Perms:  uint64(aPerm.Base.Perms),
		SetBit: uint64(aPerm.Base.SetBit),
	}
}

func toVmPermissions(ePerm *EvmPermissions) permission.AccountPermissions {
	return permission.AccountPermissions{
		Base: permission.BasePermissions{
			Perms:  permission.PermFlag(ePerm.Perms),
			SetBit: permission.PermFlag(ePerm.SetBit),
		},
	}
}
