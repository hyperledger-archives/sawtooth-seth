pragma solidity ^0.5.0;

contract Timestamp {
	function timestamp(bool test) public view returns (uint) {
		if (test) {
			return block.timestamp;
		} else {
			return 0;
		}
	}
}
