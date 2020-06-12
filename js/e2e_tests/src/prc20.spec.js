// # E2E PRC20 Tests
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

describe('prc20', () => {
    let parrot;
    let ALICE; let BOB; let CHARLIE; let
        DAVE;
    let tokenIdAlice; let
        tokenIdBob;
    let SUPPLY; let
        AMOUNT;
    beforeAll(async () => {
        parrot = new ParrotInterface();
        await parrot.initApi();
        // Init keyrings
        await parrot.initKeyRings();
        [ALICE, BOB, CHARLIE, DAVE] = parrot.keyRingPairs;
        SUPPLY = 100;
        AMOUNT = 10;
        // Create Two tokens during setup
        tokenIdAlice = await parrot.createToken(ALICE, SUPPLY);
        await sleep(SLEEP);
        tokenIdBob = await parrot.createToken(BOB, SUPPLY);
        await sleep(SLEEP);
    });
    test('createToken in setup worked correctly and new token and mints to user', async () => {
        // get balance of tokens created at setup!
        const aliceTokenBalance = await parrot.getTokenBalance(ALICE.address, tokenIdAlice);
        const bobTokenBalance = await parrot.getTokenBalance(BOB.address, tokenIdBob);
        // male sure alice has all the Supply
        expect(aliceTokenBalance.toNumber()).toBe(SUPPLY);
        expect(bobTokenBalance.toNumber()).toBe(SUPPLY);
    });

    test('transferToken works', async () => {
        const bobTokenBalanceInit = await parrot.getTokenBalance(BOB.address, tokenIdAlice);
        // try token transfer
        await parrot.transferToken(ALICE, BOB.address, tokenIdAlice, AMOUNT);
        await sleep(SLEEP);
        // make sure balances updated
        const bobTokenBalance = await parrot.getTokenBalance(BOB.address, tokenIdAlice);

        expect(bobTokenBalance.sub(bobTokenBalanceInit).toNumber()).toBe(AMOUNT);
    });

    test('tokenApprove works', async () => {
        // approve token
        await parrot.approveToken(ALICE, BOB.address, tokenIdAlice, AMOUNT);
        await sleep(SLEEP);
        // make sure allowance updated
        const tokenApprovedAmount = await parrot.getAllowanceOf(ALICE.address, BOB.address, tokenIdAlice);
        expect(tokenApprovedAmount.toNumber()).toBe(AMOUNT);
    });

    test('transferFrom works', async () => {
        // bob bal init
        const tokenBalBobInit = await parrot.getTokenBalance(BOB.address, tokenIdAlice);
        // approve Bob to spend 10 aliceToken from Alice
        await parrot.approveToken(ALICE, BOB.address, tokenIdAlice, AMOUNT);
        await sleep(SLEEP);
        // Bob tries running transfer from
        await parrot.tokenTransferFrom(BOB, ALICE.address, BOB.address, tokenIdAlice, AMOUNT);
        await sleep(SLEEP);
        // check bob bal
        const tokenBalBob = await parrot.getTokenBalance(BOB.address, tokenIdAlice);
        expect((tokenBalBob.sub(tokenBalBobInit).toNumber())).toBe(AMOUNT);
    });
    test('swap works', async () => {
        const aliceBalanceAliceTokenInit = await parrot.getTokenBalance(ALICE.address, tokenIdAlice);
        const aliceBalanceBobTokenInit = await parrot.getTokenBalance(ALICE.address, tokenIdBob);
        const bobBalanceAliceTokenInit = await parrot.getTokenBalance(BOB.address, tokenIdAlice);
        const bobBalanceBobTokenInit = await parrot.getTokenBalance(BOB.address, tokenIdBob);

        const offer = await parrot.createOffer(BOB.address, tokenIdBob, AMOUNT, tokenIdAlice, AMOUNT);
        // Bob creates a signature for the offer
        const signature = await parrot.signOffer(BOB, offer);
        // Bob creates a signedOffer
        const signedOffer = await parrot.createSignedOffer(offer, signature, BOB.address);
        // Alice broadcasts swap
        await parrot.swap(ALICE, signedOffer);
        await sleep(SLEEP);
        const aliceBalanceAliceToken = await parrot.getTokenBalance(ALICE.address, tokenIdAlice);
        const aliceBalanceBobToken = await parrot.getTokenBalance(ALICE.address, tokenIdBob);
        const bobBalanceAliceToken = await parrot.getTokenBalance(BOB.address, tokenIdAlice);
        const bobBalanceBobToken = await parrot.getTokenBalance(BOB.address, tokenIdBob);

        // Bob shld have 10 less tokenIdBob
        expect(bobBalanceBobTokenInit.sub(bobBalanceBobToken).toNumber()).toBe(AMOUNT);
        // Bob shld have 10 more of tokenIdAlice
        expect(bobBalanceAliceToken.sub(bobBalanceAliceTokenInit).toNumber()).toBe(AMOUNT);
        // Alice shld have 10 less alice token
        expect(aliceBalanceAliceTokenInit.sub(aliceBalanceAliceToken).toNumber()).toBe(AMOUNT);
        expect(aliceBalanceBobToken.sub(aliceBalanceBobTokenInit).toNumber()).toBe(AMOUNT);
    });
    test('swap fails not enuf bal', async () => {
        const aliceBalanceAliceTokenInit = await parrot.getTokenBalance(ALICE.address, tokenIdAlice);
        const aliceBalanceBobTokenInit = await parrot.getTokenBalance(ALICE.address, tokenIdBob);
        const bobBalanceAliceTokenInit = await parrot.getTokenBalance(BOB.address, tokenIdAlice);
        const bobBalanceBobTokenInit = await parrot.getTokenBalance(BOB.address, tokenIdBob);
        // wrong offer , since bob dsnt have any alicetokens to offer
        const offer = await parrot.createOffer(BOB.address, tokenIdAlice, 3 * AMOUNT, tokenIdBob, 3 * AMOUNT);
        // Bob creates a signature for the offer
        const signature = await parrot.signOffer(BOB, offer);
        // Bob creates a signedOffer
        const signedOffer = await parrot.createSignedOffer(offer, signature, BOB.address);
        // Alice broadcasts swap
        await parrot.swap(ALICE, signedOffer);
        await sleep(SLEEP);
        const aliceBalanceAliceToken = await parrot.getTokenBalance(ALICE.address, tokenIdAlice);
        const aliceBalanceBobToken = await parrot.getTokenBalance(ALICE.address, tokenIdBob);
        const bobBalanceAliceToken = await parrot.getTokenBalance(BOB.address, tokenIdAlice);
        const bobBalanceBobToken = await parrot.getTokenBalance(BOB.address, tokenIdBob);

        // should be no change in balances
        expect(bobBalanceBobTokenInit.sub(bobBalanceBobToken).toNumber()).toBe(0);
        expect(bobBalanceAliceToken.sub(bobBalanceAliceTokenInit).toNumber()).toBe(0);
        expect(aliceBalanceAliceTokenInit.sub(aliceBalanceAliceToken).toNumber()).toBe(0);
        expect(aliceBalanceBobToken.sub(aliceBalanceBobTokenInit).toNumber()).toBe(0);
    });
    test('swap fails wrong nonce', async () => {
        const aliceBalanceAliceTokenInit = await parrot.getTokenBalance(ALICE.address, tokenIdAlice);
        const aliceBalanceBobTokenInit = await parrot.getTokenBalance(ALICE.address, tokenIdBob);
        const bobBalanceAliceTokenInit = await parrot.getTokenBalance(BOB.address, tokenIdAlice);
        const bobBalanceBobTokenInit = await parrot.getTokenBalance(BOB.address, tokenIdBob);
        // Creeate a wrong offer type with wrong nonce
        let senderNonce = await parrot.getNonce(BOB.address);
        senderNonce += 2;
        const offer = await parrot.api.createType('Offer', {
            offer_token: tokenIdBob, offer_amount: AMOUNT, requested_token: tokenIdAlice, requested_amount: AMOUNT, nonce: senderNonce,
        });
        // Bob creates a signature for the offer
        const signature = await parrot.signOffer(BOB, offer);
        // Bob creates a signedOffer
        const signedOffer = await parrot.createSignedOffer(offer, signature, BOB.address);
        // Alice broadcasts swap
        await parrot.swap(ALICE, signedOffer);
        await sleep(SLEEP);
        const aliceBalanceAliceToken = await parrot.getTokenBalance(ALICE.address, tokenIdAlice);
        const aliceBalanceBobToken = await parrot.getTokenBalance(ALICE.address, tokenIdBob);
        const bobBalanceAliceToken = await parrot.getTokenBalance(BOB.address, tokenIdAlice);
        const bobBalanceBobToken = await parrot.getTokenBalance(BOB.address, tokenIdBob);
        // should be no change in balances
        expect(bobBalanceBobTokenInit.sub(bobBalanceBobToken).toNumber()).toBe(0);
        expect(bobBalanceAliceToken.sub(bobBalanceAliceTokenInit).toNumber()).toBe(0);
        expect(aliceBalanceAliceTokenInit.sub(aliceBalanceAliceToken).toNumber()).toBe(0);
        expect(aliceBalanceBobToken.sub(aliceBalanceBobTokenInit).toNumber()).toBe(0);
    });
    test('multiTransfer for Token Works', async () => {
        // get balances
        const aliceBalanceInit = await parrot.getTokenBalance(ALICE.address, tokenIdAlice);
        const bobBalanceInit = await parrot.getTokenBalance(BOB.address, tokenIdAlice);
        const charlieBalanceInit = await parrot.getTokenBalance(CHARLIE.address, tokenIdAlice);

        // Alice does multitransfer to Bob and Charlie
        const td1 = { amount: AMOUNT, to: BOB.address };
        const td2 = { amount: AMOUNT, to: CHARLIE.address };
        const mtvec = [td1, td2];
        const transfer = await parrot.api.tx.prc20.multiTransfer(tokenIdAlice, mtvec);
        const hash = await transfer.signAndSend(ALICE);
        await sleep(SLEEP);
        // get balances
        const aliceBalance = await parrot.getTokenBalance(ALICE.address, tokenIdAlice);
        const bobBalance = await parrot.getTokenBalance(BOB.address, tokenIdAlice);
        const charlieBalance = await parrot.getTokenBalance(CHARLIE.address, tokenIdAlice);
        // make sure balances changed
        expect(aliceBalanceInit.sub(aliceBalance).toNumber()).toBe(AMOUNT * 2);
        expect(bobBalance.sub(bobBalanceInit).toNumber()).toBe(AMOUNT);
        expect(charlieBalance.sub(charlieBalanceInit).toNumber()).toBe(AMOUNT);
    });
});

jest.setTimeout(30000);
