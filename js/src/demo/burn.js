// BURN DEMO
// this is a custom polkadot js api wrapper
const { BN } = require('bn.js');
const ParrotInterface = require('../parrot/interface');
// This imports the json types.json (used to define custom types)
const ADDITIONAL_TYPES = require('../types/types.json');

// number of transfers to run
const RUNS = 50;
// sleep time between actions
const SLEEP = 6000;


// prints the burner account balance and the total issuance of the chain
async function printBurnerStats(parrot) {
  const burnerBalance = await parrot.getBurnerBalance();
  console.log(`Burner Balance: ${parrot.formatToCurrency(burnerBalance)}`);
  const totalIssuance = await parrot.getTotalIssuance();
  console.log(`Total Issuance: ${parrot.formatToCurrency(totalIssuance)}`);
}
// sleep blocking
function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function burnDemo() {
  // Get a new instance of the interface
  const parrot = new ParrotInterface(ADDITIONAL_TYPES);
  // Init api
  await parrot.initApi();
  // Init keyrings
  await parrot.initKeyRings();
  // get keyrings
  let ALICE; let BOB; let CHARLIE; let
    DAVE;
  [ALICE, BOB, CHARLIE, DAVE] = parrot.keyRingPairs;


  // amount to transfer in each transfer
  const AMOUNT = parrot.DOLLARS.mul(new BN('1000'));

  // Run a bunch of blokchain operations so the burn account can recieve the fees
  console.log(`		This script will run a bunch of transfers from Alice to Dave ${RUNS} times,
		this simulates the burner pot accumulating funds from fees
		You should be able to track the Burner Pot Increasing and Decreasing in funds
		as it accumulates more fees, and burns them every BurnPeriod. You should also
		see the TotalIssuance of the system reducing whenever a burn happens`);
  for (let i = 0; i < RUNS; i++) {
    // Try to transfer;
    const transfer = parrot.api.tx.balances.transfer(DAVE.address, AMOUNT);
    // Sign and send the transaction using senderKeyring
    const hash = await transfer.signAndSend(ALICE);
    console.log(`Transfer sent with hash: ${hash.toHex()}, amount: ${parrot.formatToCurrency(AMOUNT)}`);
    await printBurnerStats(parrot);
    // sleep
    await sleep(SLEEP);
  }
}

async function main() {
  await burnDemo();
  process.exit(-1);
}

main().catch(console.error);
