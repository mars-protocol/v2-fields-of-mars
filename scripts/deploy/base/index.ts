import { setupDeployer } from './setupDeployer'
import { printRed, printYellow } from '../../utils/chalk'
import { DeploymentConfig } from '../../types/config'
import { wasmFile } from '../../utils/environment'

export interface TaskRunnerProps {
  config: DeploymentConfig
  label: string
}

export const taskRunner = async ({ config, label }: TaskRunnerProps) => {
  const deployer = await setupDeployer(config, label)
  try {
    await deployer.upload('accountNft', wasmFile('mars_account_nft'))
    await deployer.upload('swapper', wasmFile(config.swapperContractName))
    await deployer.upload('zapper', wasmFile(config.zapperContractName))
    await deployer.upload('creditManager', wasmFile('mars_credit_manager'))

    if (config.testActions) {
      await deployer.upload('mockVault', wasmFile('mars_mock_vault'))
      await deployer.instantiateMockVault()
    }

    await deployer.instantiateSwapper()
    await deployer.instantiateZapper()
    await deployer.instantiateCreditManager()
    await deployer.instantiateNftContract()
    await deployer.transferNftContractOwnership()
    await deployer.saveDeploymentAddrsToFile(label)

    // Test basic user flows
    if (config.testActions) {
      await deployer.grantCreditLines()
      await deployer.setupOraclePrices()
      await deployer.setupRedBankMarkets()

      const rover = await deployer.newUserRoverClient(config.testActions)
      await rover.createCreditAccount()
      await rover.deposit()
      await rover.borrow()
      await rover.swap()
      await rover.repay()
      await rover.withdraw()

      const vaultConfig = config.vaults[0]
      const info = await rover.getVaultInfo(vaultConfig)
      await rover.zap(info.tokens.base_token)
      await rover.vaultDeposit(vaultConfig, info)
      if (info.lockup) {
        await rover.vaultRequestUnlock(vaultConfig, info)
      } else {
        await rover.vaultWithdraw(vaultConfig, info)
        await rover.unzap(info.tokens.base_token)
      }
      await rover.refundAllBalances()
    }

    // If multisig is set, transfer ownership from deployer to multisig
    if (config.multisigAddr) {
      await deployer.updateCreditManagerOwner()
      await deployer.updateSwapperOwner()
    }

    printYellow('COMPLETE')
  } catch (e) {
    printRed(e)
  } finally {
    await deployer.saveStorage()
  }
}
