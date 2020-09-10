# How to use the OAX Polkadot Testnet

## Install Polkadot.js browser extension

For Chrome, here is the Chrome extension for the Polkadot.js which integrates your Polkadot wallets within your browser.

[Chome Polkadot.js extension](https://github.com/OAXFoundation/parrot/blob/master/Testnet.md)

[Firefox Polkadot.js addon](https://addons.mozilla.org/en-US/firefox/addon/polkadot-js-extension/)

Once installed, you need to create or import a wallet.

For detailed instructions on how to use the extension [polkadot.js extension](https://github.com/polkadot-js/extension)

## OAX Testnet UI

Now in your browser, open this link to see the UI for out testnet.

[OAX Polkadot Testnet](https://testnet.oax.org)

A popup screen for the Polkadot.js extension should display, asking you to grant permission to allow our webpage access to the extension, click Agree/Yes.

Nothing should work at first because we need to customize the UI for our chain.

Click on the Settings link on the left side of the screen, then select the Developer tab at the top of the Settings screen.

Copy the following code and replace/paste it into the Developer screen:

```json
{
"Address": "AccountId",
"TokenBalance": "u128",
"TokenId": "u128",
"Public": "AccountId",
"Signature": "MultiSignature",
"Offer": {
"offer_token": "TokenId",
"offer_amount": "TokenBalance",
"requested_token": "TokenId",
"requested_amount": "TokenBalance",
"nonce": "u128"
},
"SignedOffer": {
"offer": "Offer",
"signature": "MultiSignature",
"signer": "AccountId"
},
"TransferDetails": {
"amount": "Balance",
"to": "AccountId"
},
"TokenTransferDetails": {
"amount": "TokenBalance",
"to": "AccountId"
},
"TransferStatus": {
"amount": "TokenBalance",
"to": "AccountId",
"status": "bool"
},
"DelegatedTransferDetails": {
"amount": "Balance",
"to": "AccountId",
"nonce": "u128"
},
"SignedDelegatedTransferDetails": {
"transfer": "DelegatedTransferDetails",
"signature": "MultiSignature",
"signer": "AccountId"
}
}
```

Now click the Save button

After clicking Save, we can go back to the [Explorer](https://testnet.oax.org/#/explorer) page and in a few seconds we should see the testnet mining blocks

If you click on [Acconts](https://testnet.oax.org/#/accounts), you should see your account listed (it will have a type of 'injected'). Clicking on your account display it on the right side of your screen. If you click on the icon at the top of your account, it should copy your wallet address.

## Requesting test tokens

Note that these tokens are for testnet and have no value. Our testnet is for testing the OAX testnet only and have no use elsewhere at this time.

Goto our [Telegram group](http://bit.ly/OAXTGEn) and you can request tokens from our Faucet

For instructions type:
/help

To request tokens type:
/request [paste your address here]

Our Faucet bot should respond with a message. If successful, after a few seconds you should see the tokens in your wallet.
