<style>
r { color: Red }
o { color: Pink }
or { color: Orange }
g { color: Green }
</style>

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

> KZG verification in Solidity

## Usage

All smart contract code (solidity), fuzzing tests, and unit tests can be found in `contracts`. 
To interact with these contracts, enter the directory and ensure [foundry](https://book.getfoundry.sh/getting-started/installation) is correctly installed. Once installed run `forge test --gas-report` to collect gas consumption stats for the on-chain KZG verification and proof generation stats ! 

Future work will include a `node` directory which will implement a libp2p node with the following capabilities: 
- ingest data
- generate KZG commitments and proofs over said data 
- issue said proofs against a deployed version of the contracts in `contracts`

## Current Results

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