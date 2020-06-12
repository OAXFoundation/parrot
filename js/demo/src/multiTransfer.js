// Multi Transfer Demo (this is cheaper than normal transfer)
// this is a custom polkadot js api wrapper
const ParrotInterface = require('parrot-client');

// number of transfers to run
const RUNS = 5;
// sleep time between actions
const SLEEP = 6000;


function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
async function getSenderReceiverBalance(parrot, account1, account2) {
  // new version of node template
  const bal1 = await parrot.api.query.system.account(account1);
  const bal2 = await parrot.api.query.system.account(account2);
  return [bal1.data.free, bal2.data.free];
}

// This function runs a test transfer
async function transferXTimes(parrot, amount, senderKeyring, receiverAddress, count) {
  console.log(`Running normal Transfer ${count} times!`);
  // get sender and receiver balance and log
  let senderBalance; let
    receiverBalance;
  [senderBalance, receiverBalance] = await getSenderReceiverBalance(
    parrot,
    senderKeyring.address,
    receiverAddress,
  );
  console.log(
    `Initial Balances ... \nSender: ${parrot.formatToCurrency(senderBalance)} \nReceiver: ${
      parrot.formatToCurrency(receiverBalance)}`,
  );

  for (let i = 0; i < count; i++) {
    // Try to transfer
    console.log(`Attempting transfer of value ${parrot.formatToCurrency(amount)}!`);
    const transfer = parrot.api.tx.balances.transfer(receiverAddress, amount);
    // Sign and send the transaction using senderKeyring
    const hash = await transfer.signAndSend(senderKeyring);
    console.log('Transfer sent with hash', hash.toHex());
    // sleep
    await sleep(SLEEP);
  }

  // get sender and receiver balance again
  let senderBalanceNew; let
    receiverBalanceNew;
  [senderBalanceNew, receiverBalanceNew] = await getSenderReceiverBalance(
    parrot,
    senderKeyring.address,
    receiverAddress,
  );
  console.log(
    `Final Balances ... \nSender: ${parrot.formatToCurrency(senderBalanceNew)} \nReceiver: ${
      parrot.formatToCurrency(receiverBalanceNew)}`,
  );
  const feesPaid = senderBalance - senderBalanceNew - count * amount;

  console.log(`Fees paid by sender = ${parrot.formatToCurrency(feesPaid)}`);

  return feesPaid;
}

async function multiTransferX(parrot, amount, senderKeyring, receiverAddress, count) {
  console.log(`Running MultiTransfer ${count} times!`);
  let senderBalance; let
    receiverBalance;
  [senderBalance, receiverBalance] = await getSenderReceiverBalance(
    parrot,
    senderKeyring.address,
    receiverAddress,
  );
  console.log(
    `Initial Balances ... \nSender: ${parrot.formatToCurrency(senderBalance)} \nReceiver: ${
      parrot.formatToCurrency(receiverBalance)}`,
  );
  // Try to transfer
  console.log(
    `Attempting transfer of value ${parrot.formatToCurrency(amount)} ${count} times using a single multiTransfer!`,
  );

  const td1 = { amount, to: receiverAddress };
  const mtvec = [];
  for (let i = 0; i < count; i++) {
    mtvec.push(td1);
  }

  const transfer = await parrot.api.tx.multiTransfer.multiTransfer(mtvec);

  // Sign and send the transaction using senderKeyring
  const hash = await transfer.signAndSend(senderKeyring);
  console.log('Transfer sent with hash', hash.toHex());
  // sleep
  await sleep(SLEEP);
  // get sender and receiver balance again
  let senderBalanceNew; let
    receiverBalanceNew;
  [senderBalanceNew, receiverBalanceNew] = await getSenderReceiverBalance(
    parrot,
    senderKeyring.address,
    receiverAddress,
  );
  console.log(
    `Final Balances ... \nSender: ${parrot.formatToCurrency(senderBalanceNew)} \nReceiver: ${
      parrot.formatToCurrency(receiverBalanceNew)}`,
  );

  const feesPaid = senderBalance - senderBalanceNew - count * amount;

  console.log(`Fees paid by sender = ${parrot.formatToCurrency(feesPaid)}`);

  return feesPaid;
}


async function multiTransferDemo() {
  // Get a new instance of the interface
  const parrot = new ParrotInterface();
  // Init api
  await parrot.initApi();
  // Init keyrings
  await parrot.initKeyRings();
  // get keyrings
  let ALICE; let BOB; let CHARLIE; let
    DAVE;
  [ALICE, BOB, CHARLIE, DAVE] = parrot.keyRingPairs;

  // amount to transfer in each transfer
  const AMOUNT = 1000 * parrot.DOLLARS;

  // This script will attempt to do a normal transfer X times, followed by a multiTransfer, then compare the fees
  console.log(`This script will try to do a transfer ${RUNS} times. 
	It will then attempt to recreate a similar transfer using MultiTransfer 
	but in a single transaction. It will finally compare the fees charged!`);

  console.log('Normal transfer!');
  const fees = await transferXTimes(parrot, AMOUNT, BOB, DAVE.address, RUNS);
  console.log('Multi transfer!');
  const cheaperFees = await multiTransferX(parrot, AMOUNT, BOB, DAVE.address, RUNS);
  const difrnc = fees - cheaperFees;
  console.log(
    `Transfer cost ${parrot.formatToCurrency(fees)}, Multitransfer cost: ${parrot.formatToCurrency(cheaperFees)} MultiTransfer is cheaper by: ${parrot.formatToCurrency(difrnc)} for ${RUNS} transfers`,
  );
  process.exit(-1);
}

async function main() {
  await multiTransferDemo();
  process.exit(-1);
}

main().catch(console.error);
