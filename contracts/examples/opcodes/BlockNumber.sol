pragma solidity ^0.5.0;

contract BlockNumber {
	function blockNumber(uint number) public view returns (uint) {
		return block.number - number;
	}
}
