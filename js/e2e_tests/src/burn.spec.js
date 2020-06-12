// # E2E Burn Tests
// this is a custom polkadot js api wrapper
const { BN } = require('bn.js');
const ParrotInterface = require('parrot-client');

// TODO: move away from sleep, and wait for tx to be mined
// sleep time between txs
const SLEEP = 6000;
// sleep blocking
function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}


describe('burn', () => {
  let parrot;
  let ALICE; let BOB; let CHARLIE; let
    DAVE;
  beforeAll(async () => {
    parrot = new ParrotInterface();
    await parrot.initApi();
    // Init keyrings
    await parrot.initKeyRings();
    [ALICE, BOB, CHARLIE, DAVE] = parrot.keyRingPairs;
  });
  test('burn pot recieves funds from fees', async () => {
    const AMOUNT = 100;
    // get burn pot balance
    const burnStartBal = await parrot.getBurnerBalance();
    // make a bunch of transfer s
    const transfer = parrot.api.tx.balances.transfer(DAVE.address, AMOUNT);
    // Sign and send the transaction using senderKeyring
    let hash = await transfer.signAndSend(ALICE);
    hash = await transfer.signAndSend(BOB);
    hash = await transfer.signAndSend(CHARLIE);
    await sleep(SLEEP);
    const burnEndBal = await parrot.getBurnerBalance();
    const dif = burnEndBal - burnStartBal;
    expect(Number(dif.toString())).toBeGreaterThan(0);
  });
  // TODO: More tests, not so sure how to spot a burn event especially if the config is set to 1 Days
});

jest.setTimeout(30000);
