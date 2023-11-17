// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract Fibonacci {
    function fib(uint256 n) external pure returns(uint256 a, uint256 b) { 
    if (n == 0) {
        return (n, 0);
    }
    uint h = n / 2; 
    uint mask = 1;
    // find highest set bit in n
    while(mask <= h) {
        mask <<= 1;
    }
    mask >>= 1;
    a = 1;
    uint b = 1;
    uint c;
    while(mask > 0) {
        c = a * a+b * b;          
        if (n & mask > 0) {
            b = b * (b + 2 * a);  
            a = c;                
        } else {
            a = a * (2 * b - a);  
            b = c;                
        }
        mask >>= 1;
    }
    return (n, a);
}
}