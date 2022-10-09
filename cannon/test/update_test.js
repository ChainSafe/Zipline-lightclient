const { expect } = require("chai")
const fs = require("fs")
const {execSync} = require("child_process")
const { deploy, getTrieNodesForCall } = require("../scripts/lib")
const hre = require("hardhat")
const { network, ethers } = require("hardhat")
const { mine, time } = require("@nomicfoundation/hardhat-network-helpers");

const arrayify = ethers.utils.arrayify
const hexlify = ethers.utils.hexlify
const keccak256 = ethers.utils.keccak256

const createUpdate = (priorPeriod=1) => {
  const output = execSync(`PRIOR_PERIOD=${priorPeriod} node ../chain-fetcher-cli/dist/createUpdate.js`, {stdio: "pipe"});
  const finalizedRoot = output.slice(64, 96);
  const update = output.slice(96);
  return {finalizedRoot, update};
}

// This test needs preimages to run correctly.
// It is skipped when running `make test_contracts`, but can be run with `make test_challenge`.
describe("Challenge contract", async function () {
  const goldenRoot = execSync("cd mipsevm && go run .", {encoding: "utf8"});

  beforeEach(async function () {
    [c, m, mm] = await deploy(goldenRoot)
  })
  it("challenge contract deploys", async function() {
    console.log("Challenge deployed at", c.address)
  })
  it.only("test happy case", async function() {
    const update0 = createUpdate(3)
    const update1 = createUpdate(2)
    const update2 = createUpdate(1)

    console.log("submitting an update for period N-3");
    // submit a bootstrap update
    await c.updatePeriod(update0.update, update0.finalizedRoot, {gasLimit: 30000000});

    const blockNumber0 = await time.latestBlock()

    console.log("update N-3 is treated as a trusted bootstrap");
    // assert that the bootstrap update sets the finalized submission with the right values
    const finalizedSubmission0 = await c.finalizedSubmission()

    expect(finalizedSubmission0.blockNumber.toNumber()).to.equal(blockNumber0)
    expect(finalizedSubmission0.blockRoot).to.equal(hexlify(update0.finalizedRoot))
    expect(finalizedSubmission0.updateHash).to.equal(keccak256(update0.update))

    console.log("submitting an update for period N-2");
    // submit another update, this one should be pending
    await c.updatePeriod(update1.update, update1.finalizedRoot, {gasLimit: 30000000});

    const blockNumber1 = await time.latestBlock()

    console.log("update N-2 is now set as pending");
    // assert that the second update sets the pending submission with the right values
    const pendingSubmission0 = await c.pendingSubmission()

    expect(pendingSubmission0.blockNumber.toNumber()).to.equal(blockNumber1)
    expect(pendingSubmission0.blockRoot).to.equal(hexlify(update1.finalizedRoot))
    expect(pendingSubmission0.updateHash).to.equal(keccak256(update1.update))

    console.log("submitting update for period N-1 fails before the challenge period is over");
    // try to submit another update before the challenge period is over
    try {
      await c.updatePeriod(update2.update, update2.finalizedRoot, {gasLimit: 30000000});
      expect.fail("update shouldn't succeed during current challenge period")
    } catch (e) {}

    // dial time forward so challenge period is over
    const challengeFinishTimestamp = await c.challengeFinishTimestamp()
    await time.increaseTo(challengeFinishTimestamp)

    console.log("once the challenge period is over, submitting update for period N-1 succeeds");
    // submitting the update should now succeed
    await c.updatePeriod(update2.update, update2.finalizedRoot, {gasLimit: 30000000});

    const blockNumber2 = await time.latestBlock()

    console.log("update N-2 is now set as finalized");
    // assert that the finalized submission has been properly updated
    const finalizedSubmission1 = await c.finalizedSubmission()

    expect(finalizedSubmission1.blockNumber.toNumber()).to.equal(blockNumber1)
    expect(finalizedSubmission1.blockRoot).to.equal(hexlify(update1.finalizedRoot))
    expect(finalizedSubmission1.updateHash).to.equal(keccak256(update1.update))

    console.log("update N-1 is now set as pending");
    // assert that the pending submission has been properly updated
    const pendingSubmission1 = await c.pendingSubmission()

    expect(pendingSubmission1.blockNumber.toNumber()).to.equal(blockNumber2)
    expect(pendingSubmission1.blockRoot).to.equal(hexlify(update2.finalizedRoot))
    expect(pendingSubmission1.updateHash).to.equal(keccak256(update2.update))
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
