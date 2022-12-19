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

    function testsubmod() public {
        assertEq(verifier.submod(5, 8, 13), 10);
        assertEq(verifier.submod(0, 99, 13), 5);
        assertEq(verifier.submod(99, 0, 13), 8);
        assertEq(verifier.submod(5, 23, 4), 2);
        assertEq(verifier.submod(99, 2, 78), 19);
        assertEq(verifier.submod(BABYJUB_P, BABYJUB_P, BABYJUB_P), 0);
        assertEq(verifier.submod(BABYJUB_P, BABYJUB_P, BABYJUB_P), 0);
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

    function testevalpoly() public {
        for (uint256 i = 2; i <= 128; i *= 2) {
            uint256[] memory coefficients = new uint256[](i);
            coefficients[i - 1] = 1;
            uint256 eval = verifier.evalPolyAt(coefficients, 2);
            assertEq(eval, 2**(i - 1));
        }
    }

    function testevalpoly_withFuzzing(uint256[] calldata arr, uint256 index)
        public
        view
    {
        verifier.evalPolyAt(arr, index);
    }

    function testverify() public {
        for (uint256 i = 2; i <= 128; i *= 2) {
            uint256[] memory coefficients = new uint256[](i);
            coefficients[i - 1] = 1;
            uint256 value = verifier.evalPolyAt(coefficients, i);
            vm.assume(value < BABYJUB_P);
            Pairing.G1Point memory commitment = verifier.commit(coefficients);
            uint256[] memory proofPoly = verifier.proofPoly(coefficients, i);
            bool res = verifier.verify(
                commitment,
                verifier.commit(proofPoly),
                i,
                value
            );
            assertEq(res, true);
        }
    }

    function testverify_withFuzzing(
        // Pairing.G1Point calldata proof,
        uint256 index,
        uint256[] calldata coefficients
    ) public {
        vm.assume(index < BABYJUB_P);
        vm.assume(
            coefficients.length > 1 &&
                coefficients.length < 129 &&
                coefficients[1] > 0
        );
        uint256 value = verifier.evalPolyAt(coefficients, index);
        vm.assume(value < BABYJUB_P);
        Pairing.G1Point memory commitment = verifier.commit(coefficients);
        uint256[] memory proofPoly = verifier.proofPoly(coefficients, index);
        bool res = verifier.verify(
            commitment,
            verifier.commit(proofPoly),
            index,
            value
        );
        assertEq(res, true);
    }
}
