// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import "forge-std/Test.sol";
import "../src/KZGVerifier.sol";
import "../src/Pairing.sol";

contract KZGVerifierTest is Test {
    Verifier public verifier;

    uint256 constant BABYJUB_P =
        21888242871839275222246405745257275088548364400416034343698204186575808495617;

    function setUp() public {
        verifier = new Verifier();
    }

    function testcommit() public view {
        for (uint256 i = 2; i <= 128; i *= 2) {
            verifier.commit(new uint256[](i));
        }
    }

    function testcommit_withFuzzing(uint256[] calldata arr) public view {
        vm.assume(arr.length > 0);
        vm.assume(arr.length < 129);
        verifier.commit(arr);
    }

    function testevalpoly() public view {
        for (uint256 i = 2; i <= 128; i *= 2) {
            verifier.evalPolyAt(new uint256[](i), 0);
        }
    }

    function testevalpoly_withFuzzing(uint256[] calldata arr, uint256 index)
        public
        view
    {
        verifier.evalPolyAt(arr, index);
    }

    function testverify() public view {
        for (uint256 i = 2; i <= 128; i *= 2) {
            uint256[] memory coefficients = new uint256[](i);
            uint256 value = verifier.evalPolyAt(coefficients, i);
            Pairing.G1Point memory commitment = verifier.commit(coefficients);
            // TODO: implement genProof in solidity or use FFI cheatcode to generate a proof
            Pairing.G1Point memory proof = Pairing.G1Point(1, 2);
            verifier.verify(commitment, proof, i, value);
        }
    }

    function testverify_withFuzzing(
        Pairing.G1Point calldata proof,
        uint256 index,
        uint256[] calldata arr
    ) public view {
        vm.assume(index < BABYJUB_P && arr.length > 0);
        uint256 value = verifier.evalPolyAt(arr, index);
        vm.assume(value < BABYJUB_P);
        vm.assume(arr.length > 0 && arr.length < 129);
        Pairing.G1Point memory commitment = verifier.commit(arr);
        // TODO: implement genProof in solidity or use FFI cheatcode to generate a proof
        Pairing.G1Point memory proof = Pairing.G1Point(1, 2);

        verifier.verify(commitment, proof, index, value);
    }
}
