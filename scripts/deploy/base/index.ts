import { printRed, printYellow } from '../../utils/chalk'
import { DeploymentConfig } from '../../types/config'
import { StargateClient } from '@cosmjs/stargate/build/stargateclient'

export interface TaskRunnerProps {
  config: DeploymentConfig
  label: string
}

export const taskRunner = async ({ config }: TaskRunnerProps) => {
  try {
    const client = await StargateClient.connect(config.chain.rpcEndpoint)

    const logs = await client.searchTx({
      tags: [
        {
          key: 'wasm._contract_address',
          value: 'osmo1f2m24wktq0sw3c0lexlg7fv4kngwyttvzws3a3r3al9ld2s2pvds87jqvf',
        },
        { key: 'wasm.action', value: 'vault/request_unlock' },
      ],
    })

    const mapping: {
      apolloLockupId: string
      osmosisLockupId: string
      vaultAddr: string
      accountId: string
    }[] = []

    logs.forEach((log) => {
      const positionCreatedEvent = log.events.find(
        (e) => e.type === 'wasm-unlocking_position_created',
      )!

      const apolloLockupId = positionCreatedEvent.attributes.find(
        (a) => a.key === 'lockup_id',
      )!.value

      const osmosisLockupId = log.events
        .find((e) => e.type === 'begin_unlock')!
        .attributes.find((a) => a.key === 'period_lock_id')!.value

      const vaultAddr = log.events
        .find((e) => e.type === 'wasm-apollo/vaults/execute_unlock')!
        .attributes.find((a) => a.key === '_contract_address')!.value

      const accountId = log.events
        .find((e) => e.type === 'wasm-position_changed')!
        .attributes.find((a) => a.key === 'account_id')!.value

      mapping.push({
        apolloLockupId,
        osmosisLockupId,
        vaultAddr,
        accountId,
      })
    })

    console.log(JSON.stringify(mapping, null, 2))

    printYellow('COMPLETE')
  } catch (e) {
    printRed(e)
  }
}
