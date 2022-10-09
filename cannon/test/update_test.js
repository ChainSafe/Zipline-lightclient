const { expect } = require("chai")
const fs = require("fs")
const {execSync} = require("child_process")
const { deploy, getTrieNodesForCall } = require("../scripts/lib")
const hre = require("hardhat")
const { network, ethers } = require("hardhat")
const { mine, time } = require("@nomicfoundation/hardhat-network-helpers");

const arrayify = ethers.utils.arrayify
const hexlify = ethers.utils.hexlify

// This test needs preimages to run correctly.
// It is skipped when running `make test_contracts`, but can be run with `make test_challenge`.
describe("Challenge contract", async function () {
  const {ssz} = await import("@lodestar/types");
  const goldenRoot = execSync("cd mipsevm && go run .", {encoding: "utf8"});

  beforeEach(async function () {
    [c, m, mm] = await deploy(goldenRoot)
  })
  it("challenge contract deploys", async function() {
    console.log("Challenge deployed at", c.address)
  })
  it.only("submit a bootstrap update", async function() {
    const output = execSync("node ../shortbarrel/dist/createUpdate.js", {stdio: "pipe"})

    const finalizedRoot = output.slice(64, 96);
    const update = output.slice(96);

    // submit a single update
    await c.updatePeriod(update, finalizedRoot, {gasLimit: 30000000});

    const blockNumber = await time.latestBlock()

    // assert that the bootstrap update sets the finalized submission slot with the right values
    const finalizedSubmission = await c.finalizedSubmission()

    expect(finalizedSubmission.blockNumber.toNumber()).to.equal(blockNumber)
    expect(finalizedSubmission.blockRoot).to.equal(hexlify(finalizedRoot))
  })
  it("submit a pending update", async function() {
    // assert that the bootstrap update sets the pending submission slot with the right values
  })
  it("pending update becomes finalized after the challenge period", async function() {
    hre.network.provider.request({
      method: "evm_increaseTime",
      params: []
    })

  })
  it("pending update can be challenged during challenge period", async function() {
    // TODO: is there a better way to get the "HardhatNetworkProvider"?
    const hardhat = network.provider._wrapped._wrapped._wrapped._wrapped._wrapped
    const blockchain = hardhat._node._blockchain

    // get data
    const blockNumberN = (await ethers.provider.getBlockNumber())-2
    const blockNp1 = blockchain._data._blocksByNumber.get(blockNumberN+1)
    const blockNp1Rlp = blockNp1.header.serialize()

    const assertionRoot = "0x9e0261efe4509912b8862f3d45a0cb8404b99b239247df9c55871bd3844cebbd"
    let startTrie = JSON.parse(fs.readFileSync("/tmp/cannon/golden.json"))
    let finalTrie = JSON.parse(fs.readFileSync("/tmp/cannon/0_13284469/checkpoint_final.json"))
    let preimages = Object.assign({}, startTrie['preimages'], finalTrie['preimages']);
    const finalSystemState = finalTrie['root']

    let args = [blockNumberN, blockNp1Rlp, assertionRoot, finalSystemState, finalTrie['step']]
    let cdat = c.interface.encodeFunctionData("initiateChallenge", args)
    let nodes = await getTrieNodesForCall(c, c.address, cdat, preimages)

    // run "on chain"
    for (n of nodes) {
      await mm.AddTrieNode(n)
    }
    let ret = await c.initiateChallenge(...args)
    let receipt = await ret.wait()
    // ChallengeCreated event
    let challengeId = receipt.events[0].args['challengeId'].toNumber()
    console.log("new challenge with id", challengeId)

    // the real issue here is from step 0->1 when we write the input hash
    // TODO: prove the challenger wrong?
  }).timeout(200_000)
})
