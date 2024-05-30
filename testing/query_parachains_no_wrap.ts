(async () => {
    try {
        const api = await pjs.api.ApiPromise.create({ provider: new pjs.api.WsProvider('wss://polkadot.api.onfinality.io/public-ws') });
        const parachains: number[] = (await api.query.paras.parachains()) || [];
        console.log("parachain ids in polkadot:", parachains);

        return parachains.toJSON();
    } catch(e) {
        console.log("err:", e.toString());
    };
})();