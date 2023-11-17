pjs.api.ApiPromise.create({ provider: new pjs.api.WsProvider('wss://rococo-rpc.polkadot.io') }).then(api => {
    return api.consts.babe.epochDuration.toJSON()
});
