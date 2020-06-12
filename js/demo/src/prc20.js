// PRC20  Demo (create,transfer, approve, transferFrom)
// this is a custom polkadot js api wrapper
const ParrotInterface = require('parrot-client');

// sleep time
const SLEEP = 6000;

// this prints token stats for 2 addresses and 2 different tokens, useful in making sure that a swap has sucessfully occured (only for visual feedback purposes)
async function printAliceBobTokenStats(parrot, aliceAddress, bobAddress, aliceTokenId) {
  // Alice Bal
  const bal1 = await parrot.getTokenBalance(aliceAddress, aliceTokenId);
  const bal2 = await parrot.getTokenBalance(bobAddress, aliceTokenId);
  // Print stats
  console.log(` Token Balance Summary: \n TokenId: ${aliceTokenId} Alice: ${bal1} Bob: ${bal2} \n`);
}

// sleep blocking
function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// demo for an atomic swap of tokens (this demo creates two new tokens)
async function prc20Demo() {
  // total supply of Alice Token
  const TOTALSUPPLY = 1000;
  // transfer Amount
  const AMOUNT = 10;
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

  console.log('This is a simple demo of creating, transfering, approving and using transfer_from on prc20 tokens!');

  console.log(`Alice is creating a new PRC20 token with supply ${TOTALSUPPLY}!`);
  // Alice creates a token
  const tokenIdAlice = await parrot.createToken(ALICE, TOTALSUPPLY);
  await sleep(SLEEP);
  console.log(`Alice has created Alice Token with tokenId: ${tokenIdAlice}`);
  // print Alice and Bob token stats
  await printAliceBobTokenStats(parrot, ALICE.address, BOB.address, tokenIdAlice);
  console.log(`Now alice will transfer ${AMOUNT} tokens to BOB`);
  // Now Alice will transfer transferAmount tokens to Bobn
  await parrot.transferToken(ALICE, BOB.address, tokenIdAlice, AMOUNT);
  await sleep(SLEEP);
  // print Alice and Bob token stats
  await printAliceBobTokenStats(parrot, ALICE.address, BOB.address, tokenIdAlice);

  // Now Bob will approve Alice to spend his 10 tokens.
  console.log('Now Bob will approve Alice to spend his 10 AliceTokens');
  await parrot.approveToken(BOB, ALICE.address, tokenIdAlice, AMOUNT);
  await sleep(SLEEP);

  // Now Alice will try to send the 10 tokens back from Bobs wallet to her own wallet
  console.log('Now Alice will use transferFrom to take her approved tokens from Bobs wallet back to her own wallet ');
  await parrot.tokenTransferFrom(ALICE, BOB.address, ALICE.address, tokenIdAlice, AMOUNT);
  await sleep(SLEEP);
  // print Alice and Bob token stats
  await printAliceBobTokenStats(parrot, ALICE.address, BOB.address, tokenIdAlice);
}


async function main() {
  await prc20Demo();
  process.exit(-1);
}

main().catch(console.error);
