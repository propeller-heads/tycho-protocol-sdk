
## Token Pairs & Routes

### USDS <-> USDC
- Route: [USDS LitePSMWrapper-USDC @ Ethereum](https://developers.sky.money/modules/litepsm#usds-litepsmwrapper-usdc-ethereum)
- [Codebase](https://github.com/makerdao/usds-wrappers/blob/dev/src/UsdsPsmWrapper.sol)
- USDS Token: 0xdC035D45d973E3EC169d2276DDab16f1e407384F
- USDS Implementation: 0x1923DfeE706A8E78157416C29cBCCFDe7cdF4102
- USDS Join: 0x3C0f895007CA717Aa01c8693e59DF1e8C3777FEB
- USDS LitePSMWrapper-USDC: 0xA188EEC8F81263234dA3622A406892F3D630f98c


### DAI <-> USDC
- Route [DAI LitePSM-USDC @ Ethereum](https://developers.sky.money/modules/litepsm#dai-litepsm-usdc-ethereum)
- [Codebase](https://github.com/makerdao/dss-lite-psm)
- DAI Token: 0x6b175474e89094c44da98b954eedeac495271d0f
- DAI Join: 0x9759a6ac90977b93b58547b4a71c78317f391a28
- DAI LitePSM-USDC: 0xf6e72db5454dd049d0788e411b06cfaf16853042

### DAI <-> USDS
> Details<br>
> Converts DAI to USDS at a fixed ratio of 1:1 and vice versa.<br>
> No fees assessed. Fees cannot be enabled on this route in the future.

- Route [DAI-USDS Converter @ Ethereum](https://developers.sky.money/modules/usds#dai-usds-converter-ethereum)
- [Codebase](https://github.com/makerdao/usds/blob/dev/src/DaiUsds.sol)
- DAI Token: 0x6b175474e89094c44da98b954eedeac495271d0f
- USDS Token: 0xdC035D45d973E3EC169d2276DDab16f1e407384F
- DAI-USDS Converter: 0x3225737a9Bbb6473CB4a45b7244ACa2BeFdB276A

### USDS <-> sUSDS
