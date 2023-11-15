const endpoint = arguments[0];
const api = await pjs.api.ApiPromise.create({ provider: new pjs.api.WsProvider(endpoint) });

// We need to wrap the code in an outer promise to keep the event loop of the runtime
// running until the promise is resolved.
return new Promise(async (resolve, _reject) => {
  // subscribe to new headers, printing the full info for 5 Blocks
  let count = 0;
  const unsub = await api.rpc.chain.subscribeNewHeads((header) => {
    console.log(`#${header.number}:`, header);

    if (++count === 5) {
      console.log('5 headers retrieved, unsubscribing');
      unsub();
      return resolve(count);
    }
  });
});
