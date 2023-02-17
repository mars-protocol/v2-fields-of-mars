import { taskRunner } from '../base'
import { osmosisTestnetConfig } from './testnet-config'

void (async function () {
  await taskRunner({
    config: {
      ...osmosisTestnetConfig,
      testActions: undefined,
      oracle: { addr: 'TBD' },
      redBank: { addr: 'TBD' },
      multisigAddr: 'osmo14w4x949nwcrqgfe53pxs3k7x53p0gvlrq34l5n',
    },
    label: 'testnet-multisig',
  })
})()
