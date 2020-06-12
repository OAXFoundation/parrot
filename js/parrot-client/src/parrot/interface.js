const { ApiPromise, Keyring } = require('@polkadot/api');
const { BN } = require('bn.js');
const Util = require('@polkadot/util');
const ADDITIONAL_TYPES = require('../types/types.json');

class ParrotInterface {
    constructor(types = ADDITIONAL_TYPES) {
        this.types = types;
        this.api = undefined;
        this.keyRingPairs = [];
        this.DOLLARS = new BN('1000000000000');
        this.burnerId = 'modlpy/burns';
    }

    // This initializes api
    async initApi() {
        // Instantiate the API
        this.api = await ApiPromise.create({ types: this.types });
        // Retrieve the chain & node information information via rpc calls
        const [chain, nodeName, nodeVersion] = await Promise.all([
            this.api.rpc.system.chain(),
            this.api.rpc.system.name(),
            this.api.rpc.system.version(),
        ]);
        // Log these stats
        console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`);
    }

    // Function that loads alice, bob, charlie, and dave Keyring
    async initKeyRings() {
        // Construct the keying after the API (crypto has an async init)
        const keyring = new Keyring({ type: 'sr25519' });
        const ALICE = keyring.addFromUri('//Alice');
        const BOB = keyring.addFromUri('//Bob');
        const CHARLIE = keyring.addFromUri('//Charlie');
        const DAVE = keyring.addFromUri('//Dave');
        this.keyRingPairs = [ALICE, BOB, CHARLIE, DAVE];
    }

    formatToCurrency(value) {
        return value / this.DOLLARS;
    }

    // this returns bal, is wrapped in a function due to the possibility of this call changing
    async getFreeBalance(address) {
        const bal = await this.api.query.system.account(address);
        return bal.data.free;
    }

    async getNonce(address) {
        const stats = await this.api.query.system.account(address);
        return stats.nonce;
    }

    // returns total issuance of the system after querrying
    async getTotalIssuance() {
        const totalIssuance = await this.api.query.balances.totalIssuance();
        return totalIssuance;
    }

    // gets the balance of the burn pot
    async getBurnerBalance() {
        const PADDED_SEED = Util.stringToU8a(this.burnerId.padEnd(32, '\0'));
        const burnerBalanceStats = await this.api.query.system.account(PADDED_SEED);
        return burnerBalanceStats.data.free;
    }

    // Creates a PRC20 Token, returns tokenID
    // TODO: should be improved to use event to see if token is sucesfully created
    async createToken(keyringPair, totalSupply) {
        const tokenCount = await this.api.query.prc20.tokenCount();
        // console.log(`Current token count ${tokenCount}`);
        const tx = this.api.tx.prc20.createToken(totalSupply);
        // Sign and send the transaction using keyring
        const hash = await tx.signAndSend(keyringPair);
        console.log('CreateToken sent with hash', hash.toHex());
        // console.log(`Your tokenId is ${tokenCount}`);
        return tokenCount;
    }

    // Transfer a token
    async transferToken(keyringPair, to, tokenId, amount) {
        const prc20TransferTx = await this.api.tx.prc20.transfer(to, tokenId, amount);
        const hash = await prc20TransferTx.signAndSend(keyringPair);
        console.log('PRC20 Transfer sent with hash', hash.toHex());
    }

    // approves token
    async approveToken(keyingPair, who, tokenId, amount) {
        const approveTx = await this.api.tx.prc20.approve(who, tokenId, amount);
        const hash = await approveTx.signAndSend(keyingPair);
        console.log(`Approve sent with hash ${hash}`);
    }

    async getAllowanceOf(wallet, who, tokenId) {
        const bal = await this.api.query.prc20.allowance([tokenId, wallet, who]);
        return bal;
    }

    // token transfer from
    async tokenTransferFrom(keyringPair, from, to, tokenId, amount) {
        const transferFromTx = await this.api.tx.prc20.transferFrom(from, to, tokenId, amount);
        const hash = await transferFromTx.signAndSend(keyringPair);
        console.log(`TransferFrom sent with hash ${hash}`);
    }

    // returns the token balance
    async getTokenBalance(address, tokenId) {
        const bal = await this.api.query.prc20.balances([tokenId, address]);
        // console.log(`Token ${tokenId} Wallet: ${address} Balance: ${bal}`)
        return bal;
    }

    // creates an offer struct
    async createOffer(address, offerToken, offerAmount, requestedToken, requestedAmount) {
        const senderNonce = await this.getNonce(address);
        const offer = await this.api.createType(
            'Offer', {
                offer_token: offerToken,
                offer_amount: offerAmount,
                requested_token: requestedToken,
                requested_amount: requestedAmount,
                nonce: senderNonce,
            },
        );
        return offer;
    }

    // takes an offer and returns a signature
    async signOffer(keyRingPair, offer) {
        const encodedOffer = offer.toU8a();
        const signature = keyRingPair.sign(encodedOffer, { withType: true });
        return signature;
    }

    // creates signed offer struct
    async createSignedOffer(offer, signature, signer) {
        const signedOffer = await this.api.createType('SignedOffer', { offer, signature, signer });
        return signedOffer;
    }

    // runs swap
    async swap(keyRingPair, signedOffer) {
        const swapTx = this.api.tx.prc20.swap(signedOffer);
        const hash = await swapTx.signAndSend(keyRingPair);
        console.log('Swap sent by Alice with hash', hash.toHex());
    }

    // create a Delegated Transfer Details struct
    async createDelegatedTransferDetails(senderAddress, receiverAddress, amount) {
        const nonce = await this.getNonce(senderAddress);
        const dtd = await this.api.createType('DelegatedTransferDetails', { amount, to: receiverAddress, nonce });
        return dtd;
    }

    async signDtd(keyRingPair, dtd) {
        const encodedDtd = dtd.toU8a();
        const signature = keyRingPair.sign(encodedDtd, { withType: true });
        return signature;
    }

    async createSignedDtd(dtd, signature, signer) {
        const signedDtd = await this.api.createType('SignedDelegatedTransferDetails', { transfer: dtd, signature, signer });
        return signedDtd;
    }
}
module.exports = ParrotInterface;
