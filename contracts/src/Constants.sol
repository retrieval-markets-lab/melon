// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "./Pairing.sol";

/*
 * The values in this file come from a dummy ceremony where the secret is 1
 */

contract Constants {
    using Pairing for *;

    uint256 constant PRIME_Q =
        21888242871839275222246405745257275088696311157297823662689037894645226208583;
    uint256 constant BABYJUB_P =
        21888242871839275222246405745257275088548364400416034343698204186575808495617;

    uint256[] SRS_G1_X = [
        uint256(
            0x0000000000000000000000000000000000000000000000000000000000000001
        )
    ];

    uint256[] SRS_G1_Y = [
        uint256(
            0x0000000000000000000000000000000000000000000000000000000000000002
        )
    ];

    uint256[] SRS_G2_X_0 = [
        uint256(
            0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2
        )
    ];

    uint256[] SRS_G2_X_1 = [
        uint256(
            0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed
        )
    ];

    uint256[] SRS_G2_Y_0 = [
        uint256(
            0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b
        )
    ];

    uint256[] SRS_G2_Y_1 = [
        uint256(
            0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa
        )
    ];
}
