# e2e-tests

This crate contains e2e test scenarios for the aleph-node.

## Running

The most basic way to run (assuming a local node is listening on 9944) is:

```bash
$ NODE_URL=ws://127.0.0.1:9944 cargo test name_of_one_test
```

Note that the particular test cases might require different numbers of launched nodes, validators, or a particular
configuration of the launched nodes, see the documentation for a particular test case for details.

Additional options are passed to the tests via env variables. See `src/config.rs` for docs on available options.

## Running on devnet (or other-net)

You can also run the tests on some other network. For example, to run the contract test for the `adder` contract on
devnet:

1. Prepare an account with some money, note the seed of the account.
2. Deploy the contract to devnet:

```bash
contracts/adder$ NODE_URL=wss://ws.dev.azero.dev AUTHORITY="$THE_SEED" ./deploy.sh
```

3. Run the tests:

```bash
e2e-tests$ RUST_BACKTRACE=1 SUDO_SEED="$THE_SEED" NODE_URL=wss://ws.dev.azero.dev:443 \
  ADDER=$DEPLOY_ADDRESS ADDER_METADATA=../contracts/adder/target/ink/metadata.json cargo test adder -- --nocapture
```
