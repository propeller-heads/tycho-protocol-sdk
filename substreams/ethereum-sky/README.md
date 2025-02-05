# ethereum_sky Substreams modules

This package was initialized via `substreams init`, using the `evm-events-calls` template.

## Usage

```bash
substreams build
substreams auth
substreams gui       			  # Get streaming!
substreams registry login         # Login to substreams.dev
substreams registry publish       # Publish your Substreams to substreams.dev
```

## Modules

All of these modules produce data filtered by these contracts:
- _sdai_ at **0x83f20f44975d03b1b09e64809b757c47f942beea**
- _dai_usds_converter_ at **0x3225737a9bbb6473cb4a45b7244aca2befdb276a**
- _dai_lite_psm_ at **0xf6e72db5454dd049d0788e411b06cfaf16853042**
- _usds_psm_wrapper_ at **0xa188eec8f81263234da3622a406892f3d630f98c**
- _susds_ at **0xa3931d71877c0e7a3148cb7eb4463524fec27fbd**
- _mkr_sky_converter_ at **0xbdcfca946b6cdd965f99a839e4435bcdc1bc470b**
- _usdc_ at **0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48**
### `map_events_calls`

This module gets you events _and_ calls


### `map_events`

This module gets you only events that matched.



### `map_calls`

This module gets you only calls that matched.

## Sky Substreams Events broke down by contract

- sdai
- dai_usds_converter
- dai_lite_psm
- usds_psm_wrapper
- susds
- mkr_sky_converter

# Contracts and their deployment details

- Latest Block tx: 20,791,763
- First Creation tx (excluding SDai): 20,283,666
- Block Difference: 508,097

# sdai
> This contract DOES NOT contain DAI or SDAI
- address: 0x83F20F44975D03b1b09e64809B757c47f942BEeA
- initial_block_number: 16,428,133
- creation_tx: 0xa2f51048265f2fe9ffaf69b94cb5a2a4113be49bdecd2040d530dd6f68facc42
- first_tx_block_number: 16,932,340
- first_tx (Deposit): 0x0a3a532684b1778eef3ed18cea1134bd4f5d1cb9c05779684da2d05219097b4d
- difference_creation_tx_first_tx: 504,207

# dai_usds_converter
> This contract DOES NOT contain DAI or USDS
- address: 0x3225737a9Bbb6473CB4a45b7244ACa2BeFdB276A
- initial_block_number: 20,663,734
- creation_tx: 0xb63d6f4cfb9945130ab32d914aaaafbad956be3718176771467b4154f9afab61
- first_tx_block_number: 20,770,195
- first_tx (Deposit): 0x371f1f6e26604915c10a88368ebcf3d6132ec2b2d38a7d33bd196f224d9c2efb
- difference_creation_tx_first_tx: 106,461

# dai_lite_psm
> This contract contains DAI, DOES NOT contain USDC (Gem)
- address: 0xf6e72Db5454dd049d0788e411b06CfAF16853042
- initial_block_number: 20,283,666
- creation_tx: 0x61e5d04f14d1fea9c505fb4dc9b6cf6e97bc83f2076b53cb7e92d0a2e88b6bbd
- first_tx_block_number: 20,535,921
- first_tx (BuyGem): 0xc600c7c838194b469303526d3ab90fdbf803168d24e9ee005a5c52756150b8ba
- difference_creation_tx_first_tx: 252,255

## usds_psm_wrapper
> This contract DOES NOT contain USDS or USDC (Gem)
- address: 0xA188EEC8F81263234dA3622A406892F3D630f98c
- initial_block_number: 20,668,728
- creation_tx: 0x43ddae74123936f6737b78fcf785547f7f6b7b27e280fe7fbf98c81b3c018585
- first_tx_block_number: 20,791,763
- first_tx (SellGem): 0x723ae2dce4a2d2f675efc523763c18fecbcf058db8574b5921115eb23838eff8
- difference_creation_tx_first_tx: 123,035


## susds
> This contract contains USDS, DOES NOT contain sUSDS
- address: 0xa3931d71877C0E7a3148CB7Eb4463524FEc27fbD
- initial_block_number: 20,677,434
- creation_tx: 0xe1be00c4ea3c21cf536b98ac082a5bba8485cf75d6b2b94f4d6e3edd06472c00
- first_tx_block_number: 20,771,188
- first_tx (Withdraw): 0xfbdc07be16dced7a8bb74dc4357694cc0de0ad162d397a3514cba2673bad5f77
- difference_creation_tx_first_tx: 93,754

## mkr_sky_converter
> This contract DOES NOT contain MKR or SKY
- address: 0xBDcFCA946b6CDd965f99a839e4435Bcdc1bc470B
- initial_block_number: 20,663,740
- creation_tx: 0xbd89595dadba76ffb243cb446a355cfb833c1ea3cefbe427349f5b4644d5fa02
- first_tx_block_number: 20,770,588
- first_tx (MkrToSky): 0xbfe0e1d64ebd87775c6160124d5c8533b9759fa86cdea5f388aab156b1a26904
- difference_creation_tx_first_tx: 106,848







---

## First sDai Deposit tx details

**SUMMARY**
When deposit event DAI must be subtracted from totalbalance.
Shares (sDai) are added to totalSupply and user balance.

- Event: Deposit
- Block number: 16,932,340
- Tx hash: 0x0a3a532684b1778eef3ed18cea1134bd4f5d1cb9c05779684da2d05219097b4d
- TokenIn: DAI
- AmountIn: 50000000000000000000 (50*10^18)
- TokenOut: sDai
- AmountOut: 48965183040843668922 (48.965183040843668922*10^18)
- From: 0xd1236a6A111879d9862f8374BA15344b6B233Fbd (Spark: Deployer)
- To: 0x83F20F44975D03b1b09e64809B757c47f942BEeA (Savings Dai)

1. Spark:Deployer deposits 50 DAI (assets) (50*10^18)
2. SavingsDai `function _mint` -> `dai.transferFrom` -> assets are transferred to SavingsDai contract
```
    function _mint(uint256 assets, uint256 shares, address receiver) internal {
        require(receiver != address(0) && receiver != address(this), "SavingsDai/invalid-address");

        dai.transferFrom(msg.sender, address(this), assets);
        daiJoin.join(address(this), assets);
        pot.join(shares);

        // note: we don't need an overflow check here b/c shares totalSupply will always be <= dai totalSupply
        unchecked {
            balanceOf[receiver] = balanceOf[receiver] + shares;
            totalSupply = totalSupply + shares;
        }

        emit Deposit(msg.sender, receiver, assets, shares);
    }
```
3. SavingsDai calls [DaiJoin.join](https://etherscan.io/address/0x9759A6Ac90977b93B58547b4A71c78317f391A28#code#L217)
```
    function join(address usr, uint wad) external note {
        vat.move(address(this), usr, mul(ONE, wad));
        dai.burn(msg.sender, wad);
    }
```
4. Vat keeps track of the assets (DAI) deposited by the user through mapping `mapping (address => uint256) public dai;  // [rad]`. So that when the user deposits DAI the amount is added to the user's balance and subtracted from the source (SavingsDai address)
```
    function move(address src, address dst, uint256 rad) external note {
        require(wish(src, msg.sender), "Vat/not-allowed");
        dai[src] = sub(dai[src], rad);
        dai[dst] = add(dai[dst], rad);
    }
```
5. After `vat.move`, `dai.burn` is called
```
    function burn(address usr, uint wad) external {
        require(balanceOf[usr] >= wad, "Dai/insufficient-balance");
        if (usr != msg.sender && allowance[usr][msg.sender] != uint(-1)) {
            require(allowance[usr][msg.sender] >= wad, "Dai/insufficient-allowance");
            allowance[usr][msg.sender] = sub(allowance[usr][msg.sender], wad);
        }
        balanceOf[usr] = sub(balanceOf[usr], wad);
        totalSupply    = sub(totalSupply, wad);
        emit Transfer(usr, address(0), wad);
    }
```
6. `dai.burn` emits a `Transfer` event with `from` as the user's address and `to` as `address(0)` (Burn event), even thougn **I am not able to find effectively where this DAI are transferred to, after being transferred from user to SavingsDai.**
> Anyway in the ERC-20 Token Transfered is visible that 50 DAI were transferred from Spark:Deployer to SavingsDai and from SavingsDai to address(0) (Burn event)
7. SavingsDai calls `pot.join`
```
    // --- Savings Dai Management ---
    function join(uint wad) external note {
        require(now == rho, "Pot/rho-not-updated");
        pie[msg.sender] = add(pie[msg.sender], wad);
        Pie             = add(Pie,             wad);
        vat.move(msg.sender, address(this), mul(chi, wad));
    }

## First sDai **Redeem** tx details

**SUMMARY**


- Event: Redeem
- MethodId: 0xba087652
- Block number: 17,322,856
- Tx hash: 0xd5b5f52f10286a9c04faced2f7ba3e4e66fdad3565d89a0d59d935c39e26c602
- TokenIn: sDai
- AmountIn: 6709310969721276955771 (6709.310969721276955771*10^18)
- TokenOut: DAI
- AmountOut: 6861424195429254648600 (6861.424195429254648600*10^18)
- From: address(0)
- To: 0x52A8305f29f85bEc5fa6eE78B87Ddd2218d8E12E (sparkfi.eth)

# dai_usds_converter
- address: 0x3225737a9Bbb6473CB4a45b7244ACa2BeFdB276A
- initial_block_number: 20663734
- Age: Sep-02-2024 03:19:35 PM UTC
- creation_tx: 0xb63d6f4cfb9945130ab32d914aaaafbad956be3718176771467b4154f9afab61
- contract_creator: 0x4Ec216c476175a236BD70026b984D4adECa0cfb8 (Sky: Deployer)

## dai_lite_psm
- address: 0xf6e72Db5454dd049d0788e411b06CfAF16853042
- initial_block_number: 20283666
- Age: Jul-11-2024 01:48:47 PM UTC
- creation_tx: 0x61e5d04f14d1fea9c505fb4dc9b6cf6e97bc83f2076b53cb7e92d0a2e88b6bbd
- contract_creator: 0xb27B6fa77D7FBf3C1BD34B0f7DA59b39D3DB0f7e

## usds_psm_wrapper
- address: 0xA188EEC8F81263234dA3622A406892F3D630f98c
- initial_block_number: 20668728
- Age: Sep-03-2024 08:02:59 AM UTC
- creation_tx: 0x43ddae74123936f6737b78fcf785547f7f6b7b27e280fe7fbf98c81b3c018585
- contract_creator: 0x4E65a603a9170fa572E276D1B70D6295D433bAc5

## susds
- address: 0xa3931d71877C0E7a3148CB7Eb4463524FEc27fbD
- initial_block_number: 20677434
- Age: Sep-03-2024 08:02:59 AM UTC
- creation_tx: 0xe1be00c4ea3c21cf536b98ac082a5bba8485cf75d6b2b94f4d6e3edd06472c00
- contract_creator: 0xD6ec7a1b1f4c42C5208fF68b2436Fab8CC593fB7 (Sky Deployer 6)

## mkr_sky_converter
- address: 0xBDcFCA946b6CDd965f99a839e4435Bcdc1bc470B
- initial_block_number: 20663740
- Age: Sep-02-2024 03:20:47 PM UTC
- creation_tx: 0xbd89595dadba76ffb243cb446a355cfb833c1ea3cefbe427349f5b4644d5fa02
- contract_creator: 0x4Ec216c476175a236BD70026b984D4adECa0cfb8 (Sky: Deployer)