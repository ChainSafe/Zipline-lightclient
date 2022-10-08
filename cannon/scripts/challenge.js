const fs = require("fs")
const { execSync } = require("node:child_process");
const { deployed, getTrieNodesForCall } = require("../scripts/lib")


async function main() {
  let [c, m, mm] = await deployed()

  console.log(c.address, m.address, mm.address)

  const output = execSync("node ../shortbarrel/dist/createUpdate.js")

  const finalSystemState = output.slice(0, 32);
  const step = ethers.BigNumber.from(output.slice(32, 64));
  const finalizedHash = output.slice(64, 96);
  const update = output.slice(96);

  let args = [finalSystemState, step]
  let cdat = c.interface.encodeFunctionData("initiateChallenge", args)
  let nodes = await getTrieNodesForCall(c, c.address, cdat, preimages)

  // run "on chain"
  for (n of nodes) {
    await mm.AddTrieNode(n)
  }
// TODO: Setting the gas limit explicitly here shouldn't be necessary, for some
//    weird reason (to be investigated), it is for L2.
//  let ret = await c.initiateChallenge(...args)
  let ret = await c.initiateChallenge(...args, { gasLimit: 10_000_000 })
  let receipt = await ret.wait()
  // ChallengeCreated event
  let challengeId = receipt.events[0].args['challengeId'].toNumber()
  console.log("new challenge with id", challengeId)
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
