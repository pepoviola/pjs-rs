
const endpoint = arguments[0];
const seed = arguments[1];

console.log(0);
const api = await pjs.api.ApiPromise.create({ provider: new pjs.api.WsProvider(endpoint) });
await pjs.utilCrypto.cryptoWaitReady();
const keyring = new pjs.keyring.Keyring({ type: "sr25519" });
const signer = keyring.addFromUri(seed);

console.log(1);
// Make a transfer from Alice to Bob and listen to system events.
// You need to be connected to a development chain for this example to work.
const ALICE = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const BOB = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';

// Get a random number between 1 and 100000
const randomAmount = Math.floor((Math.random() * 100000) + 1);

// Create a extrinsic, transferring randomAmount units to Bob.
const transferAllowDeath = api.tx.balances.transferAllowDeath(BOB, randomAmount);

return new Promise(async (resolve, _reject) => {
    console.log(2);
    // Sign and Send the transaction
    const unsub = await transferAllowDeath.signAndSend(signer, ({ events = [], status }) => {
        console.log(3);
        if (status.isInBlock) {
            console.log('Successful transfer of ' + randomAmount + ' with hash ' + status.asInBlock.toHex());
            return resolve();
        } else {
            console.log('Status of transfer: ' + status.type);
        }

        events.forEach(({ phase, event: { data, method, section } }) => {
            console.log(phase.toString() + ' : ' + section + '.' + method + ' ' + data.toString());
        });
    });
});