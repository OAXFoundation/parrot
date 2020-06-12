# Parrot Client  + E2E Tests + Demo Scripts 


## Parrot Client 

This is a simple node package that uses @polkadot/api and implements the custom methods needed to interact with the OAX parachain.

To learn how to use this package, you can look at the demo scripts for examples, or for the simplest connection example look at [this](https://github.com/OAXFoundation/parrot/blob/master/js/parrot-client/src/app.js)

## E2E Tests 

This requires you to have a local node running, please launch the local node before proceeding. 

### Installation 

1) ``` cd e2e_tests ```

2) ```npm install``` 

### E2E tests 

Each test needs to be run individually to avoid one test interfering with another. This is because currently we are not waiting for events and just sleeping. In the future this should be updated. This is done using the ```runInBand``` cli argument of jest which can be seen in this projects `package.json`. 

### Run all tests 


```npm run test``` 


Be patient, this may take several minutes to complete since we have to wait for these transactions to be mined. 

### Run only one test 

To launch a test simply run `jest prc20` , this will find the `prc20.spec.js` file and run the tests. (You may need to have jest installed globally, alternatively do `./node_modules/.bin/jest prc20`)

Current e2e test and names:

- [prc20](https://github.com/OAXFoundation/parrot/blob/master/js/e2e_tests/src/prc20.spec.js) This has tests for atmoicSwap and prc20 
- [burn](https://github.com/OAXFoundation/parrot/blob/master/js/e2e_tests/src/burn.spec.js)
- [multiTransfer](https://github.com/OAXFoundation/parrot/blob/master/js/e2e_tests/src/multiTransfer.spec.js)
- [feeDelegation](https://github.com/OAXFoundation/parrot/blob/master/js/e2e_tests/src/feeDelegation.spec.js)



## Demo Scripts 

The following scripts are simple command line demos of each of the unique parachain features. 
Please cd into the demo/src directory before running the commands below. You can additionally also have the susbtrate UI set-up in order to follow the demos on chain

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
