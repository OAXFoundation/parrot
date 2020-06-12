// # E2E multiTransfer Tests
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

describe('mutliTransfer', () => {
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
    test('multiTransfer Works', async () => {
        const AMOUNT = 100 * parrot.DOLLARS;

        const bobBalanceStart = await parrot.getFreeBalance(BOB.address);
        const charlieBalanceStart = await parrot.getFreeBalance(CHARLIE.address);
        const aliceBalanceStart = await parrot.getFreeBalance(ALICE.address);

        // build transfer details
        const td1 = { amount: AMOUNT, to: BOB.address };
        const td2 = { amount: AMOUNT, to: CHARLIE.address };
        // build a multi trafer vec
        const mtvec = [td1, td2];
        // runmultitransfer
        const transfer = await parrot.api.tx.multiTransfer.multiTransfer(mtvec);

        const hash = await transfer.signAndSend(ALICE);
        await sleep(SLEEP);
        // get peoples balances
        const bobBalanceEnd = await parrot.getFreeBalance(BOB.address);
        const charlieBalanceEnd = await parrot.getFreeBalance(CHARLIE.address);
        const aliceBalanceEnd = await parrot.getFreeBalance(ALICE.address);

        const charlieDif = charlieBalanceEnd - (charlieBalanceStart);
        const bobDif = bobBalanceEnd - (bobBalanceStart);
        const aliceDif = aliceBalanceStart - aliceBalanceEnd;

        // since the same user is paying fees, the transfer amount may be slightly less unlike with tokens
        const compareAmount = AMOUNT - 4 * parrot.DOLLARS;
        expect(charlieDif).toBeGreaterThanOrEqual(compareAmount);
        expect(bobDif).toBeGreaterThanOrEqual(compareAmount);
    });
    // TODO: More tests
});

jest.setTimeout(30000);
