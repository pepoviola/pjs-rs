const api = await ApiPromise.create({ provider: new WsProvider('wss://rpc.polkadot.io') });
const parachains: number[] = (await api.query.paras.parachains()) || [];
console.log("parachain ids in polkadot:", parachains);

return parachains.toJSON();
