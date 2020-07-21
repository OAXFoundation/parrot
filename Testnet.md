# OAX Testnet 

[Blockchain Portal](https://polkadotnet.duckdns.org/#/explorer)
[WSS URL](wss://polkadotnet.duckdns.org/wss)


## Requesting Tokens 

Please send an email to wayland@oax.org to request for OAX Testnet tokens. The email should be in the following format. 

`Subject`: Requesting OAX Testnet Tokens 
`Body`: Address: 5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY

## Interacting with the OAX Testnet 

There are two ways in which you can interact with the OAX chain. The first method is using the Blokchain Portal. The second method is using the oax client. 


### Using the UI 

This is the generic Polkadot/Substrate portal, and you can interact with OAX's custom runtime's by going to the Extrinsic's and Chain State tabs. 


### Using OAX Client

Please refer to  [this](https://github.com/OAXFoundation/parrot/blob/master/js/README.md) for a detailed overview of 
using the client. 

The only difference you need to take note of, is while connecting to the OAX testnet, you have to initialize the client
as follows, passing the new WSS url for the API. 

```
// Get a new instance of the interface
const parrot = new ParrotInterface('wss://polkadotnet.duckdns.org/wss');
```

If you do not pass anything, it will default to using a local testnet at `ws://localhost:9944`