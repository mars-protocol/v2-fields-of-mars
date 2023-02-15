import { taskRunner } from '../base'
import { DeploymentConfig } from '../../types/config'

const uosmo = 'uosmo'
const uatom = 'ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2'
const axlUSDC = 'ibc/D189335C6E4A68B513C10AB227BF1C1D38C746766278BA3EEB4FB14124F1D858'
const gammPool1 = 'gamm/pool/1'
const gammPool678 = 'gamm/pool/678'

// Latest from: https://stats.apollo.farm/api/vaults/v1/all
const vaultOsmoAtom1 = 'TBD'
const vaultOsmoAtom7 = 'TBD'
const vaultOsmoAtom14 = 'TBD'
const atomOsmoConfig = {
  config: {
    deposit_cap: { denom: axlUSDC, amount: '2000000000000' }, // $2M
    max_ltv: '0.63',
    liquidation_threshold: '0.65',
    whitelisted: true,
  },
}

const vaultUsdcOsmo1 = 'TBD'
const vaultUsdcOsmo7 = 'TBD'
const vaultUsdcOsmo14 = 'TBD'
const usdcOsmoConfig = {
  config: {
    deposit_cap: { denom: axlUSDC, amount: '750000000000' }, // $750k
    max_ltv: '0.65',
    liquidation_threshold: '0.66',
    whitelisted: true,
  },
}

export const osmosisMainnetConfig: DeploymentConfig = {
  multisigAddr: 'osmo14w4x949nwcrqgfe53pxs3k7x53p0gvlrq34l5n',
  allowedCoins: [uosmo, uatom, axlUSDC, gammPool1, gammPool678],
  chain: {
    baseDenom: uosmo,
    defaultGasPrice: 0.1,
    id: 'osmosis-1',
    prefix: 'osmo',
    rpcEndpoint: 'https://rpc.osmosis.zone',
  },
  deployerMnemonic: 'TO BE INSERTED AT TIME OF DEPLOYMENT',
  maxCloseFactor: '0.5',
  maxUnlockingPositions: '1',
  maxValueForBurn: '10000',
  oracle: { addr: 'osmo1mhznfr60vjdp2gejhyv2gax9nvyyzhd3z0qcwseyetkfustjauzqycsy2g' },
  redBank: { addr: 'osmo1c3ljch9dfw5kf52nfwpxd2zmj2ese7agnx0p9tenkrryasrle5sqf3ftpg' },
  swapRoutes: [
    { denomIn: uosmo, denomOut: uatom, route: [{ token_out_denom: uatom, pool_id: '1' }] },
    { denomIn: uatom, denomOut: uosmo, route: [{ token_out_denom: uosmo, pool_id: '1' }] },
    { denomIn: uosmo, denomOut: axlUSDC, route: [{ token_out_denom: axlUSDC, pool_id: '678' }] },
    { denomIn: axlUSDC, denomOut: uosmo, route: [{ token_out_denom: uosmo, pool_id: '678' }] },
  ],
  vaults: [
    {
      vault: { address: vaultOsmoAtom1 },
      ...atomOsmoConfig,
    },
    {
      vault: { address: vaultOsmoAtom7 },
      ...atomOsmoConfig,
    },
    {
      vault: { address: vaultOsmoAtom14 },
      ...atomOsmoConfig,
    },
    {
      vault: { address: vaultUsdcOsmo1 },
      ...usdcOsmoConfig,
    },
    {
      vault: { address: vaultUsdcOsmo7 },
      ...usdcOsmoConfig,
    },
    {
      vault: { address: vaultUsdcOsmo14 },
      ...usdcOsmoConfig,
    },
  ],
  swapperContractName: 'mars_swapper_osmosis',
  zapperContractName: 'mars_zapper_osmosis',
}

void (async function () {
  await taskRunner({
    config: osmosisMainnetConfig,
    label: 'mainnet',
  })
})()
