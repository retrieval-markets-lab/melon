<h1 align="center">
	<br>
	  	🍉🍈
	<br>
	<br>
	Melon
	<br>
	<br>
	<br>
</h1>

> Repo for KZG verification in Solidity and KZG proof generation in Rust. 

## Usage

All smart contract code (solidity), fuzzing tests, and unit tests can be found in `contracts`. 
To interact with these contracts, enter the directory and ensure [foundry](https://book.getfoundry.sh/getting-started/installation) is correctly installed. Once installed run `forge test --gas-report` to collect gas consumption stats for the on-chain KZG verification and proof generation stats ! 

The `node` directory implements a set of naive methods for lagrange interpolation on data and KZG commitments using rust. 
Run `cargo bench` to generate measurements. 

Future work will create utilities within the rust crate for: 
- issuing KZG proofs against a deployed version of the contracts in `contracts`

## Current Results

Here we present the latest results for benchmarks run within the two sub-directories.

### Contract gas costs

Stats on the gas costs for functions on the `src/KZGVerifier.sol:Verifier` contract (over 500 calls).

| src/KZGVerifier.sol:Verifier contract |                 |        |        |         |         |
|---------------------------------------|-----------------|--------|--------|---------|---------|
| Deployment Cost                       | Deployment Size |        |        |         |         |
| 7160083                               | 15328           |        |        |         |         |
| <o>Function Name</o>                  | <g>min</g>      | <or>avg</or>    | <or>median</or>  | <r>max</r>     | # calls |
| commit                                | 8753            | 388326 | 203131 | 1276560 | 24      |
| evalPolyAt                            | 1436            | 14218  | 8323   | 40909   | 16      |
| proofPoly                             | 4206            | 57702  | 36668  | 190353  | 8       |
| submod                                | 623             | 655    | 623    | 699     | 7       |
| verify                                | 134374          | 139374 | 134374 | 154374  | 8       |

### Generating KZG Commitments in Rust

Stats on the execution times for interpolating data, generating commitments from the interpolated polynomial, and generating KZG proofs (average of 100 samples).
- **Machine**: M1 Pro, 32GB RAM


| (num coeffs) --->                     | 16        | 128       | 512       | 2048      | 5096      |
|---------------------------------------|-----------|-----------|-----------|-----------|-----------|
| commit                                | 326.87 µs | 460.30 µs | 694.36 µs | 1.8086 ms | 5.0649 ms |
| create_witness                        | 700.88 µs | 624.53 µs | 2.2374 ms | 4.5927 ms | 13.984 ms |
| interpolation                         | 103.83 µs | 1.6751 ms | 14.935 ms | 149.59 ms | 709.13 ms |

