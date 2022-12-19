// SPDX-License-Identifier: UNLICENSED
// Modified from https://github.com/appliedzkp/semaphore/blob/master/contracts/sol/verifier.sol
pragma experimental ABIEncoderV2;
pragma solidity ^0.8.13;

import "./Pairing.sol";
import {Constants} from "./Constants.sol";

contract Verifier is Constants {
    using Pairing for *;

    uint256 public number;

    // The G1 generator
    Pairing.G1Point SRS_G1_0 =
        Pairing.G1Point({X: Constants.SRS_G1_X[0], Y: Constants.SRS_G1_Y[0]});

    // The G2 generator
    Pairing.G2Point g2Generator =
        Pairing.G2Point({
            X: [Constants.SRS_G2_X_0[0], Constants.SRS_G2_X_1[0]],
            Y: [Constants.SRS_G2_Y_0[0], Constants.SRS_G2_Y_1[0]]
        });

    Pairing.G2Point SRS_G2_1 =
        Pairing.G2Point({
            X: [Constants.SRS_G2_X_0[1], Constants.SRS_G2_X_1[1]],
            Y: [Constants.SRS_G2_Y_0[1], Constants.SRS_G2_Y_1[1]]
        });

    /*
     * Verifies a single-point evaluation of a polynominal using the KZG
     * commitment scheme.
     * Returns true if and only if the following holds, and returns false
     * otherwise:
     *     e(commitment - commit([_value]), G2.g) == e(proof, commit([0, 1]) - zCommit)
     * @param _commitment The KZG polynomial commitment.
     * @param _proof The proof.
     * @param _index The x-value at which to evaluate the polynomial.
     * @param _value The result of the polynomial evaluation.
     */
    function verify(
        Pairing.G1Point memory _commitment,
        Pairing.G1Point memory _proof,
        uint256 _index,
        uint256 _value
    ) public view returns (bool) {
        // Make sure each parameter is less than the prime q
        require(
            _commitment.X < BABYJUB_P,
            "Verifier.verifyKZG: _commitment.X is out of range"
        );
        require(
            _commitment.Y < BABYJUB_P,
            "Verifier.verifyKZG: _commitment.Y is out of range"
        );
        require(
            _proof.X < BABYJUB_P,
            "Verifier.verifyKZG: _proof.X is out of range"
        );
        require(
            _proof.Y < BABYJUB_P,
            "Verifier.verifyKZG: _proof.Y is out of range"
        );
        require(
            _index < BABYJUB_P,
            "Verifier.verifyKZG: _index is out of range"
        );
        require(
            _value < BABYJUB_P,
            "Verifier.verifyKZG: _value is out of range"
        );

        // Compute commitment - aCommitment
        Pairing.G1Point memory commitmentMinusA = Pairing.plus(
            _commitment,
            Pairing.negate(Pairing.mulScalar(SRS_G1_0, _value))
        );

        // Negate the proof
        Pairing.G1Point memory negProof = Pairing.negate(_proof);

        // Compute index * proof
        Pairing.G1Point memory indexMulProof = Pairing.mulScalar(
            _proof,
            _index
        );

        // Returns true if and only if
        // e((index * proof) + (commitment - aCommitment), G2.g) * e(-proof, xCommit) == 1
        return
            Pairing.pairing(
                Pairing.plus(indexMulProof, commitmentMinusA),
                g2Generator,
                negProof,
                SRS_G2_1
            );
    }

    /*
     * @return A KZG commitment to a polynominal
     * @param coefficients The coefficients of the polynomial to which to
     *                     commit.
     */
    function commit(uint256[] memory coefficients)
        public
        view
        returns (Pairing.G1Point memory)
    {
        Pairing.G1Point memory result = Pairing.G1Point(0, 0);

        for (uint256 i = 0; i < coefficients.length; i++) {
            result = Pairing.plus(
                result,
                Pairing.mulScalar(
                    Pairing.G1Point({
                        X: Constants.SRS_G1_X[i],
                        Y: Constants.SRS_G1_Y[i]
                    }),
                    coefficients[i]
                )
            );
        }
        return result;
    }

    /*
     * @return The polynominal evaluation of a polynominal with the specified
     *         coefficients at the given index.
     */
    function evalPolyAt(uint256[] memory _coefficients, uint256 _index)
        public
        pure
        returns (uint256)
    {
        uint256 m = Constants.BABYJUB_P;
        uint256 result = 0;
        uint256 powerOfX = 1;

        for (uint256 i = 0; i < _coefficients.length; i++) {
            uint256 coeff = _coefficients[i];
            assembly {
                result := addmod(result, mulmod(powerOfX, coeff, m), m)
                powerOfX := mulmod(powerOfX, _index, m)
            }
        }
        return result;
    }

    /*
     * @return A KZG commitment proof of evaluation at a single point.
     * @param coefficients The coefficients of the polynomial associated with the
     *                     KZG commitment.
     * @param index The x-value for the polynomial evaluation proof.
     * @param p The field size. Defaults to the BabyJub field size.
     */
    //  TODO: finish implementing this
    function proofPoly(uint256[] memory _coefficients, uint256 _index)
        public
        pure
        returns (uint256[] memory)
    {
        // first we generate the quotient polynomial
        uint256 m = Constants.BABYJUB_P;
        uint256 yval = evalPolyAt(_coefficients, _index);
        uint256[] memory polya = _coefficients;
        polya[0] = submod(polya[0], yval, m);
        uint256 polyb = submod(0, _index, m);

        uint256 apos = lastNonZeroIndex(polya);
        // bpos =  lastNonZeroIndex(polyb) which will always be 1 in this case
        // as such diff = (apos - bpos) is just (apos - 1)
        require(apos >= 1, "Cannot divide by polynomial of higher order");
        uint256[] memory divpoly = new uint256[](apos);
        // adapted from https://github.com/GuildOfWeavers/galois/blob/f3d9cfbf2fe7857f3840bdba3406e2ba9ea548c7/lib/PrimeField.ts#L660
        for (uint256 i = apos; i > 0; i--) {
            divpoly[i - 1] = polya[i];
            // simplify quot = div(polya[apos], polyb[bpos]) to polya[apos], as polyb[pos] is always 1
            polya[i - 1] = submod(polya[i - 1], mulmod(polyb, polya[i], m), m);
        }
        return divpoly;
    }

    /* @dev Modular subtraction of two numbers (mod p).
     * @param _a first number
     * @param _b second number
     * @param _pp The modulus
     * @return q such (_a-_b)(mod _pp)
     */
    function submod(
        uint256 _a,
        uint256 _b,
        uint256 _pp
    ) public pure returns (uint256) {
        uint256 a = _a % _pp;
        uint256 b = _b % _pp;
        if (a >= b) {
            return (a - b) % _pp;
        } else {
            return _pp - ((b - a) % _pp);
        }
    }

    function lastNonZeroIndex(uint256[] memory values)
        internal
        pure
        returns (uint256)
    {
        for (uint256 i = values.length - 1; i >= 0; i--) {
            if (values[i] != 0) return i;
        }
        // works solely for this particular use-case ! do not attempt to utilize in other contexts
        return 0;
    }
}
