// ATOMIC SWAP DEMO for PRC20 TOKENS
// this is a custom polkadot js api wrapper
const ParrotInterface = require('./parrot/interface');

// demo for an atomic swap of tokens (this demo creates two new tokens)
async function testConnect() {
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
}

async function main() {
    await testConnect();
    process.exit(-1);
}

main().catch(console.error);
