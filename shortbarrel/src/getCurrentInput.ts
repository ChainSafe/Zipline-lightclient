import { readFile } from "node:fs/promises";
import { utils, providers, Contract, ContractInterface } from "ethers";

const abi = [
  {
    "inputs": [
      {
        "internalType": "bytes",
        "name": "lightClientUpdate",
        "type": "bytes"
      },
      {
        "internalType": "bytes32",
        "name": "assertedFinalizedBlockRoot",
        "type": "bytes32"
      }
    ],
    "name": "updatePeriod",
    "outputs": [],
    "stateMutability": "nonpayable",
    "type": "function"
  },
  {
    "inputs": [],
    "name": "currentBlockNumber",
    "outputs": [
      {
        "internalType": "uint256",
        "name": "",
        "type": "uint256"
      }
    ],
    "stateMutability": "view",
    "type": "function"
  },
];

async function getInput(
  provider: providers.Provider,
  address: string,
  abi: ContractInterface,
  pending?: boolean
): Promise<Uint8Array> {
  const contract = new Contract(address, abi, provider);
  const updatePeriodSelector = contract.interface.fragments
    .find((fragment) => fragment.name === "updatePeriod")
    .format();

  console.log(updatePeriodSelector);

  const blockNumber = pending ? await contract.pendingBlockNumber() : await contract.currentBlockNumber();
  const block = await provider.getBlockWithTransactions(blockNumber);
  // loop thru block
  const updatePeriodTx = block.transactions.find(
    (transaction) =>
      transaction.to === address &&
      transaction.data.slice(2).startsWith(updatePeriodSelector)
  );
  return Buffer.from(updatePeriodTx.data.slice(10), "hex");
}
