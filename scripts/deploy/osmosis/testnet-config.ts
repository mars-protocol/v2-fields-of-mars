import { DeploymentConfig, VaultType } from '../../types/config'

const uosmo = 'uosmo'
const uatom = 'ibc/A8C2D23A1E6F95DA4E48BA349667E322BD7A6C996D8A4AAE8BA72E190F3D1477'
const axlUSDC = 'ibc/6F34E1BD664C36CE49ACC28E60D62559A5F96C4F9A6CCE4FC5A67B2852E24CFE'

const usdcOsmoPoolTest = 'gamm/pool/5'

const vaultOsmoUsdc1 = 'osmo1q40xvrzpldwq5he4ftsf7zm2jf80tj373qaven38yqrvhex8r9rs8n94kv'
const vaultOsmoUsdc7 = 'osmo14lu7m4ganxs20258dazafrjfaulmfxruq9n0r0th90gs46jk3tuqwfkqwn'
const vaultOsmoUsdc14 = 'osmo1fmq9hw224fgz8lk48wyd0gfg028kvvzggt6c3zvnaqkw23x68cws5nd5em'
const osmoUsdcConfig = {
  config: {
    deposit_cap: { denom: axlUSDC, amount: '1000000000' }, // 1000 atom
    max_ltv: '0.63',
    liquidation_threshold: '0.65',
    whitelisted: true,
  },
}

// REPLACE WITH ATOM/OSMO once pool is created and vaults are deployed
// const vaultJunoOsmo1 = 'osmo1r6h0pafu3wq0kf6yv09qhc8qvuku2d6fua0rpwwv46h7hd8u586scxspjf'
// const vaultJunoOsmo7 = 'osmo1gr5epxn67q6202l3hy0mcnu7qc039v22pa6x2tsk23zwg235n9jsq6pmes'
// const vaultJunoOsmo14 = 'osmo1d6knwkelyr9eklewnn9htkess4ttpxpf2cze9ec0xfw7e3fj0ggssqzfpp'
// const junoOsmoConfig = {
//   config: {
//     deposit_cap: { denom: uatom, amount: '500000000' }, // 500 atom
//     max_ltv: '0.65',
//     liquidation_threshold: '0.66',
//     whitelisted: true,
//   },
// }

export const osmosisTestnetConfig: DeploymentConfig = {
  allowedCoins: [uosmo, uatom, axlUSDC, usdcOsmoPoolTest],
  chain: {
    baseDenom: uosmo,
    defaultGasPrice: 0.1,
    id: 'osmo-test-5',
    prefix: 'osmo',
    rpcEndpoint: 'https://rpc.osmotest5.osmosis.zone',
  },
  deployerMnemonic:
    'rely wonder join knock during sudden slow plate segment state agree also arrest mandate grief ordinary lonely lawsuit hurt super banana rule velvet cart',
  maxCloseFactor: '0.6',
  maxUnlockingPositions: '10',
  maxValueForBurn: '1000000',
  // Latest from: https://github.com/mars-protocol/outposts/blob/master/scripts/deploy/addresses/osmo-test-5.json
  oracle: { addr: 'osmo1khe29uw3t85nmmp3mtr8dls7v2qwsfk3tndu5h4w5g2r5tzlz5qqarq2e2' },
  redBank: { addr: 'osmo1dl4rylasnd7mtfzlkdqn2gr0ss4gvyykpvr6d7t5ylzf6z535n9s5jjt8u' },
  swapRoutes: [
    { denomIn: uosmo, denomOut: axlUSDC, route: [{ token_out_denom: axlUSDC, pool_id: '5' }] },
    { denomIn: axlUSDC, denomOut: uosmo, route: [{ token_out_denom: uosmo, pool_id: '5' }] },
  ],
  // Latest from: https://stats.apollo.farm/api/vaults/v1/all
  vaults: [
    {
      vault: { address: vaultOsmoUsdc1 },
      ...osmoUsdcConfig,
    },
    {
      vault: { address: vaultOsmoUsdc7 },
      ...osmoUsdcConfig,
    },
    {
      vault: { address: vaultOsmoUsdc14 },
      ...osmoUsdcConfig,
    },
  ],
  swapperContractName: 'mars_swapper_osmosis',
  zapperContractName: 'mars_zapper_osmosis',
  testActions: {
    allowedCoinsConfig: [
      { denom: uosmo, priceSource: { fixed: { price: '1' } }, grantCreditLine: true },
      // POOL NEEDS TO BE CREATED
      // {
      //   denom: uatom,
      //   priceSource: { geometric_twap: { pool_id: 1, window_size: 1800 } },
      //   grantCreditLine: true,
      // },
      {
        denom: axlUSDC,
        priceSource: { geometric_twap: { pool_id: 5, window_size: 1800 } },
        grantCreditLine: true,
      },
      {
        denom: usdcOsmoPoolTest,
        priceSource: { xyk_liquidity_token: { pool_id: 5 } },
        grantCreditLine: false,
      },
    ],
    vault: {
      depositAmount: '1000000',
      withdrawAmount: '1000000',
      mock: {
        config: {
          deposit_cap: { denom: uosmo, amount: '100000000' }, // 100 osmo
          liquidation_threshold: '0.585',
          max_ltv: '0.569',
          whitelisted: true,
        },
        vaultTokenDenom: axlUSDC,
        type: VaultType.LOCKED,
        lockup: { time: 900 }, // 15 mins
        baseToken: usdcOsmoPoolTest,
      },
    },
    outpostsDeployerMnemonic:
      'elevator august inherit simple buddy giggle zone despair marine rich swim danger blur people hundred faint ladder wet toe strong blade utility trial process',
    borrowAmount: '10',
    repayAmount: '11',
    defaultCreditLine: '100000000000',
    depositAmount: '100',
    secondaryDenom: uatom,
    startingAmountForTestUser: '2500000',
    swap: {
      slippage: '0.4',
      amount: '40',
      route: [
        {
          token_out_denom: uatom,
          pool_id: '1',
        },
      ],
    },
    unzapAmount: '1000000',
    withdrawAmount: '12',
    zap: {
      coinsIn: [
        {
          denom: uatom,
          amount: '1',
        },
        { denom: uosmo, amount: '3' },
      ],
      denomOut: usdcOsmoPoolTest,
    },
  },
}
