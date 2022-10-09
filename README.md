# Zipline - A Trustless Eth2 Bridge Protocol
![](https://i.imgur.com/44VWVOd.png)
## Relevant Repository Layout
----
<pre>
<a href="./zipline">zipline</a>
├── <a href="./zipline/light-client-rs">light-client-rs</a>: Ethereum Light Client protocol verification functions in platform-agnostic Rust.  Modified code from by <a href="https://github.com/Snowfork/snowbridge">SnowBridge</a> team to remove all Substrate related functionality and add statelessness.
├── <a href="./zipline/cannon">cannon</a>: Forked from Optimism's <a href="https://github.com/ethereum-optimism/cannon">Cannon</a> repo.
|   |── <a href="./zipline/cannon/contracts">contracts</a>
|   |   |── <a href="./zipline/cannon/contracts/Challenge.sol">Challenge.sol</a>: Forked contract that is used to arbitrate Interactive Verification Game. Modified it to track Ethereum Chain. 
|   |── <a href="./zipline/cannon/mipsevm">mipsevm</a>: Forked to run Rust MIPS code instead of minigeth.  
|   |── <a href="./zipline/cannon/contracts">scripts</a>: Added scripts to support interaction with Zipline.
|   |── <a href="./zipline/cannon/contracts">test</a>: Test functionality of Zipline contracts.
├── <a href="./zipline/light-client-verification-mips">light-client-verification-mips</a>: Adapted from <a href="https://github.com/pepyakin/rusty-cannon">rusty-cannon</a>. A build system for making Cannon compatible Rust programs.
├── <a href="./zipline/chain-fetcher-cli">chain-fetcher-cli</a>: Scrapes an ETH2 chain to create light client updates for Zipline.
├── <a href="./zipline/light-client-verification-cli">light-client-verification-cli</a>: A Rust CLI that runs light-client-rs natively to verify light client updates.
</pre>

## Background

### Ethereum Light Client Protocol

Since the Altair upgrade Ethereum has a light client protocol that allows a node to follow the network by receiving a single signed update message every ~27 hours. Each sync period (~27 hours) a sync committee of 512 validators is selected. They attest all finalized blocks for their duration and at the end of the sync period a new committee is deterministically selected.

A light client node receives *SyncUpdate* messages which contain:

- Aggregte BLS signature and participation bitfield
- Header of the attested block
- List of pubkeys of next committee
- Proof committee rotation

Starting from a trusted finalized block and sync committee (could be genesis) a light client that receives a *SyncUpdate* can be given a new block root and committee and verify that the committee is correct and that the block is finalized in the canonical chain. A *SyncUpdate* contains everything needed to verify this transition.

Executing this light client on-chain would allow for a trustless bridge where anyone can relay updates and the chain can verify the correctness. Unfortunately the verification process is too expensive to run on-chain. The signature verification alone requires 512 elliptic curve additions on BLS12–381 plus a pairing check which far exceeds most chains gas limit. 

A solution to this is to perform the execution of the light-client off-chain and then verify the execution on-chain. A number of teams are working on implementing this with SNARKS [^1][^2] or executes the verification in a blockchain where execution can be cheap (e.g. Substrate) [^3]. Zipline instead uses fault proofs to verify the execution of the light client on EVM chains.
### Fault Proofs

Fault proofs allow any observer to prove that another actor has **not** performed execution of a program correctly. Under an honest, active minority assumption any execution can be assumed to be correct after a sufficient time period has elapsed with no challenge.

### Optimism Cannon

Optimism developed an EVM based fault proof stack to verify the execution of the state transition of their rollup. It can prove execution of any program that can be compiled to the MIPS CPU architecture.

It works through a Solidity contract that is able to execute a single MIPS instruction given the memory and registers it requires. A two-party bisection protocol is used between the submitter of the proof and the challenger to identify the first instruction where both parties agree on the state of the CPU memory before hand but disagree on it after. The contract can then execute this single instruction and is the final arbitrator on which party, the challenger or submitter is correct.

This has amazing implications. It means the execution of an arbitrarily large program can be verified false by executing only a single instruction on-chain!

Through a clever construction called the [pre-image oracle]() allows the program being proven to access any data that is available on the host chain (including calldata). This is how the state transition program for Optimism has access to the rollup transactions and how Zipline receives the `SyncUpdate` messages.

## Zipline

Zipline is comprised of two main components. 

#### Verification Program

The first is a Rust implementation of the `validateUpdate` function of the Ethereum light client protocol. It accepts as input the hash of the prior SyncUpdate and a current SyncUpdate and outputs a boolean of if the transition between is valid. 

It uses the input hashes to retrieve the full `SyncUpdate` payloads via the pre-image oracle, deserializes them from SSZ, validates the signatures from the current committee, and calculates the next committee. 

The code for this component is adapted from the [Snowbridge Ethereum to Substrate bridge](https://github.com/Snowfork/snowbridge). It was refactored to have a much smaller footprint and to compile to baremetal MIPS (e.g. Rust no_std).

#### Contract

The second component is the on-chain component that is deployed to the destination EVM chain. This was adapted from Optimism Cannon but redesigned to implement a bridge rather than a rollup.

It keeps track of two update submissions. One is the most recent finalized submission (e.g. no successful fault proof challenge was submitted during the challenge period) and the other is the current pending submission which can be challenged. 

```solidity
struct UpdateSubmission {
    // merkle root of the beacon block header
    bytes32 blockRoot;
    // keccak of the light client update
    bytes32 updateHash;
    // block number of the submission
    uint256 blockNumber;
}
```

These are updated with every submission so the chain always has access to the most recent validated, finalized block root. This can be used by contracts on the destination EVM chain to prove the inclusion of any piece of data in the Eth2 chain state. This in turn can be used to develop token bridges (if a bridge in the other direction can be established) or other cross-chain applications.

### Flow

The bridge must start from a trusted initial block root and committee.

An untrusted relayer can submit new `SyncUpdate` messages as they become available along with a bond. Once the bridge contract receives a new update it stores it as pending.

#### Happy Case

In the ideal case the relayer submitted a valid `SyncUpdate` and no further action is required. After the duration of the challenge period the block referenced by the SyncUpdate can be assumed to be finalized, both on the source chain and on the destination. The relayer can reclaim their bond and may be rewarded depending on the implementation.

#### Unhappy Case (Relayer Fraud)

In the unhappy case the relayer submitted an invalid `SyncUpdate`. Either the signature verification or the new committee rotation are not valid.

A watcher notices this is invalid by executing the `validateUpdate` code off-chain. Then then run the validation code again within a MIPS emulator to obtain the final state of the MIPS machine memory. This includes the output showing the validation evaluates to false and the number of instructions executed to obtain the result. This snapshot Merklized to obtain the final memory snapshot root.

The watcher calls the `initiateChallenge` function on the bridge contract with the final snapshot root and number of instruction steps. They also submit a bond. This begins the two party bisection game.

The relayer must respond with their proposed snapshot root at the same number of instruction steps. If these disagree another instruction index is selected midway between. The challenger then submits what they think the snapshot root is at this point and if they think the relayer is at fault in the left or right half of the execution trace. 

If either participant stops responding on their turn the other participant wins by default.

This continues until the participants find two adjacent snapshot roots where they agree on the first and disagree on the second. The contract `MIPS.sol` can then execute instruction that transitions between and determine that the relayer is at fault. The watcher receives both their bond back and the relayers bond as a reward.

See the [Cannon specification](https://github.com/ethereum-optimism/cannon/wiki/Cannon-Overview) for details on how the participants must make the memory required by the instruction available on-chain along with proofs of inclusion in the snapshot.

The `SyncUpdate` will be removed from pending.

#### Unhappy Case (Watcher Fraud)

A watcher might also challenge a valid `SyncUpdate`. This case is almost identical to before except the on-chain instruction execution finds the watchers proposed snapshot to be invalid. In this case the `SyncUpdate` remains pending and the relayer receives the watchers bond.

## Epoch Updates

The protocol as described so far only relays a single block per sync period (8192 slots). Proving a transaction was accepted in another prior block in the worst case requires submitting up to 8191 block header hashes as part of the proof.

Fortunately because the committee also attests to every block in a sync period it is possible to verify these signatures to jump forward to other finalized blocks. This uses the same verification program and fault proof logic as described above but without the proof of committee cycling.

It does not make sense for every block to be relayed that way (too expensive) but a block from each epoch or so could be relayed and the signatures verified with a fault proof. This can reduce the number of hashes required for a proof of transaction to a manageable level.

## Future Work

The next step for this project is implementing a contract that consumes the finalized block headers and uses them as a root to verify proofs of inclusion for pieces of state. This could allow for state synchronization between the origin and target chains.

[^1]: https://rdi.berkeley.edu/zkp/zkBridge/zkBridge.html
[^2]: https://hackmd.io/@umaroy/SkB26pAFc
[^3]: https://github.com/Snowfork/snowbridge
## Demo Instructions
------

### Testing the zipline contract

- Build chain-fetcher-cli
    ```shell
    cd chain-fetcher-cli
    yarn && yarn build
    ```
- Compile zipline contracts
    ```shell
    cd cannon
    npm i
    npx hardhat compile
    ```
- Run hardhat test
    ```shell
    cd cannon
    npx hardhat test
    ```

### Running light-client-verification-cli

```shell
cd light-client-verification-cli
# <hash1> and <hash2> would be known by chain-fetcher-cli
cargo run -q -- <hash1> <hash2>
```

### Running mipsevm

- build light-client-verification-mips
    ```shell
    cd light-client-verification-mips
    make
    ```
- build mipsevm
    ```shell
    cd cannon
    make build
    ```
- run mipsevm
    ```shell
    cd cannon/mipsevm
    # <hash1> and <hash2> would be known by chain-fetcher-cli
    go run . <hash1> <hash2>
    ```

### Running a relayer in testnet

- make sure Zipline contracts are compiled
- make sure mipsevm has been built
- 
```shell
# in one window
cd cannon
./demo/forked_node.sh

# in another window
cd cannon
# deploy contract and seed with an initial light client update
npx hardhat run scripts/deploy.js
PRIOR_PERIOD=2 npx hardhat run scripts/submitUpdate.js
# run this once per sync committee period
npx hardhat run scripts/submitUpdate.js
```
