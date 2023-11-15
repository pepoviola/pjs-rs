// Get chain state information
// Make our basic chain state / storage queries, all in one go

const endpoint = arguments[0];
const api = await pjs.api.ApiPromise.create({ provider: new pjs.api.WsProvider(endpoint) });

const [now, validators] = await Promise.all([
    api.query.timestamp.now(),
    api.query.session.validators()
  ]);

  console.log('The current date is: ' + now);

  if (validators && validators.length > 0) {
    // Retrieve the balances for all validators
    console.log('Validators');

    const validatorBalances = await Promise.all(
      validators.map((authorityId) => api.query.system.account(authorityId))
    );

    validators.forEach((authorityId, index) => {
      console.log('Validator: ' + authorityId.toString() )
      console.log('AccountData: ' + JSON.stringify(validatorBalances[index].toHuman()) );
    });
  }
