pragma solidity ^0.5.0;

contract BlockHash {
	function blockHash(uint64 blockNum) public view returns (bytes32) {
		return blockhash(blockNum);
	}
}
