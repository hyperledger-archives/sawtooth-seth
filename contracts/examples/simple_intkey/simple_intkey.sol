pragma solidity ^0.4.0;

contract intkey {
  mapping (uint => uint) intmap;

  event Set(uint key, uint value);

  function set(uint key, uint value) public {
    intmap[key] = value;
    emit Set(key, value);
  }

  function inc(uint key) public {
    intmap[key] = intmap[key] + 1;
  }

  function dec(uint key) public {
    intmap[key] = intmap[key] - 1;
  }

  function get(uint key) public constant returns (uint retVal) {
    return intmap[key];
  }
}
