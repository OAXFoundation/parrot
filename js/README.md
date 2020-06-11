# Demo Scripts & E2E Tests

These scripts can be used for using additional methods on the OAX Parachain with @polkadot-js/api

All these tests and script require you to have a local node running. Please follow instructions in the previous page to run a local instance of this chain before proceeding. 

## Installation 

```npm install``` in the current dir 

## E2E tests 

Each test needs to be run individually to avoid one test interfering with another. This is because currently we are not waiting for events and just sleeping. In the future this should be updated. This is done using the ```runInBand``` cli arguement of jest which can be seen in this projects `package.json`. 

## Run all tests 

`npm run test`

Be patient, this may take several minutes to complete since we have to wait for these transactions to be mined. 

## Run only one test 

To launch a test simply run `jest prc20` , this will find the `prc20.spec.js` file and run the tests. (You may need to have jest installed globally, alternatively do `./node_modules/.bin/jest prc20`)

Current e2e test and names:

- [prc20](https://github.com/OAXFoundation/parrot/blob/master/js/src/e2e_tests/prc20.spec.js) This has tests for atmoicSwap and prc20 
- [burn](https://github.com/OAXFoundation/parrot/blob/master/js/src/e2e_tests/burn.spec.js)
- [multiTransfer](https://github.com/OAXFoundation/parrot/blob/master/js/src/e2e_tests/multiTransfer.spec.js)
- [feeDelegation](https://github.com/OAXFoundation/parrot/blob/master/js/src/e2e_tests/feeDelegation.spec.js)



## Demo Scripts 

The following scripts are simple command line demos of each of the unique parachain features. 
Please cd into the src/demo directory before running the commands below. You can additionally also have the susbtrate UI set-up in order to follow the demos on chain

### PRC20 Token 

```node prc20.js```

### Atomic Swap  

```node atomicSwap.js```

### Automatic Fee Burn  

```node burn.js```

### MultiTransfer  

``` node multiTransfer.js```

### Fee Delegation  

```node feeDelegation.js```


## Additional FAQ

All these demos use @polkadot/api.
A simple api wrapper is used to make things easier to read. The code for this can be found in the `src/parrot/interfaces.js`. Custom types can also be provided by using a different types.json file. 
