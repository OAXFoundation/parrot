// Fee Delegation Demo (Free Transfer Demo)
// this is a custom polkadot js api wrapper
const ParrotInterface = require('parrot-client');
// lib to get user input 
const readline = require('readline');

// function to ask question 
function askQuestion(query) {
    const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout,
    });

    return new Promise(resolve => rl.question(query, ans => {
        rl.close();
        resolve(ans);
    }))
}

// sleep time between actions
const SLEEP = 6000;

// sleep blocking
function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function getAliceBobBalStats(parrot, aliceAddress, bobAddress, charlieAddress) {
  const balAlice = await parrot.getFreeBalance(aliceAddress);
  const balBob = await parrot.getFreeBalance(bobAddress);
  const balCharlie = await parrot.getFreeBalance(charlieAddress);
  console.log(` Balance Summary: \n Alice: ${parrot.formatToCurrency(balAlice)} Bob ${parrot.formatToCurrency(balBob)} Charlie ${parrot.formatToCurrency(balCharlie)}`);
  return [balAlice, balBob, balCharlie];
}

async function balanceDifference(parrot, balAlice, balBob, balCharlie,
  balAliceNew, balBobNew, balCharlieNew) {
  const aliceSpent = balAlice.sub(balAliceNew);
  const bobSpent = balBob.sub(balBobNew);
  const charlieReceived = balCharlieNew.sub(balCharlie);
  console.log(`Alice spent ${parrot.formatToCurrency(aliceSpent)} Bob spent ${parrot.formatToCurrency(bobSpent)} Charlie Received: ${parrot.formatToCurrency(charlieReceived)}`);
}


async function feeDelegationDemo() {
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

  let balAlice; let balBob; let
    balCharlie;
  [balAlice, balBob, balCharlie] = await getAliceBobBalStats(parrot, ALICE.address, BOB.address, CHARLIE.address);

  console.log(`Bob wants to send ${parrot.formatToCurrency(AMOUNT)} to Charlie but does not want to pay fees!`)

  // Bob creates a dtd
  const dtd = await parrot.createDelegatedTransferDetails(BOB.address, CHARLIE.address, AMOUNT);
  // Bob signs it
  const signature = await parrot.signDtd(BOB, dtd);
  // Bob creates a SignedDtd
  const signedDtd = await parrot.createSignedDtd(dtd, signature, BOB.address);
  console.log('Bob has created a signedDelegatedTransferDetails that he can share with a fee delegator');

  console.log(` SignedDTD: \n Transfer: ${signedDtd.transfer}\n Signature: ${signedDtd.signature}\n Signer: ${signedDtd.signer}`)

  const ans = await askQuestion('\n \nDo you want to broadcast this manually? Please type Y or N:   ')
  if (ans.toLowerCase() === "n"){
    console.log('Alice acts as the fee delegator and broadcasts this signedDelegatedTransferDetails since she is willing to fee delegate');
    // Now Bob sends this signedDtd ofline to Alice
    // Alice decides to broadcast it since she is willing to do the trade for Bob
    const transferTx = await parrot.api.tx.delegation.delegatedTransfer(signedDtd);
    const hash = await transferTx.signAndSend(ALICE);
    console.log('Delegated transfer sent by Alice with hash', hash.toHex());
    await sleep(SLEEP);

    let balAliceNew; let balBobNew; let
      balCharlieNew;
    [balAliceNew, balBobNew, balCharlieNew] = await getAliceBobBalStats(parrot, ALICE.address, BOB.address, CHARLIE.address);
    await balanceDifference(parrot, balAlice, balBob, balCharlie, balAliceNew, balBobNew, balCharlieNew);
  }
  else if (ans.toLowerCase() === "y"){
    const resp = await askQuestion('\n \nPlease type anything, once you have broadcasted the transaction and it has been mined!:   ')

    let balAliceNew; let balBobNew; let
      balCharlieNew;
    [balAliceNew, balBobNew, balCharlieNew] = await getAliceBobBalStats(parrot, ALICE.address, BOB.address, CHARLIE.address);
    await balanceDifference(parrot, balAlice, balBob, balCharlie, balAliceNew, balBobNew, balCharlieNew);
  }
}


async function main() {
  await feeDelegationDemo();
  process.exit(-1);
}

main().catch(console.error);
