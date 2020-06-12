// # E2E feeDelegation Tests
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


describe('feeDelegation', () => {
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
  test('fee delegation works', async () => {
    const AMOUNT = 1000 * parrot.DOLLARS;
    // Get everyone's start bal
    const bobBal = await parrot.getFreeBalance(BOB.address);
    const aliceBal = await parrot.getFreeBalance(ALICE.address);
    const charlieBal = await parrot.getFreeBalance(CHARLIE.address);
    // Bob creates a dtd
    const dtd = await parrot.createDelegatedTransferDetails(BOB.address, CHARLIE.address, AMOUNT);
    // Bob signs it
    const signature = await parrot.signDtd(BOB, dtd);
    // Bob creates a SignedDtd
    const signedDtd = await parrot.createSignedDtd(dtd, signature, BOB.address);
    // Now bob gives this signedDtd to fee Delegator Alice to broadcast
    const transferTx = await parrot.api.tx.delegation.delegatedTransfer(signedDtd);
    const hash = await transferTx.signAndSend(ALICE);
    await sleep(SLEEP);
    // Get everyone's end bal
    const bobBalEnd = await parrot.getFreeBalance(BOB.address);
    const aliceBalEnd = await parrot.getFreeBalance(ALICE.address);
    const charlieBalEnd = await parrot.getFreeBalance(CHARLIE.address);

    const difBob = bobBal.sub(bobBalEnd);
    const difAlice = aliceBal.sub(aliceBalEnd);
    const difCharlie = charlieBalEnd.sub(charlieBal);
    console.log(difBob, difAlice, difCharlie);
    const compareAmount = AMOUNT;

    expect(parrot.formatToCurrency(difBob)).toBe(parrot.formatToCurrency(AMOUNT));
    expect(parrot.formatToCurrency(difCharlie)).toBe(parrot.formatToCurrency(AMOUNT));
    expect(parrot.formatToCurrency(difAlice)).toBeGreaterThan(0);
  });
  // TODO: More tests, not so sure how to spot a burn event especially if the config is set to 1 Days
});

jest.setTimeout(30000);
