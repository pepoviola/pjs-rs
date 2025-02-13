
const endpoint = arguments[0];
const seed = arguments[1];
const amount = arguments[2];

const api = await ApiPromise.create({ provider: new pjs.api.WsProvider(endpoint) });
await utilCrypto.cryptoWaitReady();
const k = new keyring.Keyring({ type: "sr25519" });
const signer = k.addFromUri(seed);

// Make a transfer from Alice to Bob and listen to system events.
// You need to be connected to a development chain for this example to work.
const ALICE = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const BOB = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';

// Get a random number between 1 and 100000
const randomAmount = amount || Math.floor((Math.random() * 100000) + 1);

// Create a extrinsic, transferring randomAmount units to Bob.
const transferAllowDeath = api.tx.balances.transferAllowDeath(BOB, randomAmount);

return new Promise(async (resolve, _reject) => {
    // Sign and Send the transaction
    const unsub = await transferAllowDeath.signAndSend(signer, { nonce: -1 }, ({ events = [], status }) => {
        if (status.isInBlock) {
            console.log('Successful transfer of ' + randomAmount + ' with hash ' + status.asInBlock.toHex());
            return resolve(randomAmount);
        } else {
            console.log('Status of transfer: ' + status.type);
        }

        events.forEach(({ phase, event: { data, method, section } }) => {
            console.log(phase.toString() + ' : ' + section + '.' + method + ' ' + data.toString());
        });
    });
});
