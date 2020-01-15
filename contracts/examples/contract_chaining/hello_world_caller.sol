pragma solidity ^0.5.0;

contract HelloWorld {
	function helloWorld() public pure returns(bytes32);
}
contract HelloWorldCaller {
	function callHelloWorld(address helloWorldAddr) public pure returns(bytes32) {
		HelloWorld hello = HelloWorld(helloWorldAddr);
		return hello.helloWorld();
	}
}
