
const api = await pjs.api.ApiPromise.create({ provider: new pjs.api.WsProvider('wss://rpc.polkadot.io') });
const parachains = (await api.query.paras.parachains()) || [];
console.log("parachain ids in polkadot:", parachains);

return parachains.toJSON();
