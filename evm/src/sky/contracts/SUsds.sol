// SPDX-License-Identifier: AGPL-3.0-or-later

/// SUsds.sol

// Copyright (C) 2017, 2018, 2019 dbrock, rain, mrchico
// Copyright (C) 2021 Dai Foundation
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
/*
pragma solidity ^0.8.21;

import "@openzeppelin/contracts-upgradeable/proxy/utils/UUPSUpgradeable.sol";

interface IERC1271 {
    function isValidSignature(
        bytes32,
        bytes memory
    ) external view returns (bytes4);
}

interface VatLike {
    function hope(address) external;
    function suck(address, address, uint256) external;
}

interface UsdsJoinLike {
    function vat() external view returns (address);
    function usds() external view returns (address);
    function exit(address, uint256) external;
}

interface UsdsLike {
    function transfer(address, uint256) external;
    function transferFrom(address, address, uint256) external;
}

contract SUsds is UUPSUpgradeable {

    // --- Storage Variables ---

    // Admin
    mapping (address => uint256) public wards;
    // ERC20
    uint256                                           public totalSupply;
    mapping (address => uint256)                      public balanceOf;
    mapping (address => mapping (address => uint256)) public allowance;
    mapping (address => uint256)                      public nonces;
    // Savings yield
    uint192 public chi;   // The Rate Accumulator  [ray]
    uint64  public rho;   // Time of last drip     [unix epoch time]
    uint256 public ssr;   // The USDS Savings Rate [ray]

    // --- Constants ---

    // ERC20
    string  public constant name     = "Savings USDS";
    string  public constant symbol   = "sUSDS";
    string  public constant version  = "1";
    uint8   public constant decimals = 18;
    // Math
    uint256 private constant RAY = 10 ** 27;

    // --- Immutables ---

    // EIP712
    bytes32 public constant PERMIT_TYPEHASH = keccak256("Permit(address owner,address spender,uint256 value,uint256 nonce,uint256 deadline)");
    // Savings yield
    UsdsJoinLike public immutable usdsJoin;
    VatLike      public immutable vat;
    UsdsLike     public immutable usds;
    address      public immutable vow;

    // --- Events ---

    // Admin
    event Rely(address indexed usr);
    event Deny(address indexed usr);
    event File(bytes32 indexed what, uint256 data);
    // ERC20
    event Approval(address indexed owner, address indexed spender, uint256 value);
    event Transfer(address indexed from, address indexed to, uint256 value);
    // ERC4626
    event Deposit(address indexed sender, address indexed owner, uint256 assets, uint256 shares);
    event Withdraw(address indexed sender, address indexed receiver, address indexed owner, uint256 assets, uint256 shares);
    // Referral
    event Referral(uint16 indexed referral, address indexed owner, uint256 assets, uint256 shares);
    // Savings yield
    event Drip(uint256 chi, uint256 diff);

    // --- Modifiers ---

    modifier auth {
        require(wards[msg.sender] == 1, "SUsds/not-authorized");
        _;
    }

    // --- Constructor ---

    constructor(address usdsJoin_, address vow_) {
        _disableInitializers(); // Avoid initializing in the context of the implementation

        usdsJoin = UsdsJoinLike(usdsJoin_);
        vat = VatLike(UsdsJoinLike(usdsJoin_).vat());
        usds = UsdsLike(UsdsJoinLike(usdsJoin_).usds());
        vow = vow_;
    }

    // --- Upgradability ---

    function initialize() initializer external {
        __UUPSUpgradeable_init();

        chi = uint192(RAY);
        rho = uint64(block.timestamp);
        ssr = RAY;
        vat.hope(address(usdsJoin));
        wards[msg.sender] = 1;
        emit Rely(msg.sender);
    }

    function _authorizeUpgrade(address newImplementation) internal override auth {}

    function getImplementation() external view returns (address) {
        return ERC1967Utils.getImplementation();
    }

    // --- Internals ---

    // EIP712

    function _calculateDomainSeparator(uint256 chainId) private view returns (bytes32) {
        return keccak256(
            abi.encode(
                keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"),
                keccak256(bytes(name)),
                keccak256(bytes(version)),
                chainId,
                address(this)
            )
        );
    }

    function DOMAIN_SEPARATOR() external view returns (bytes32) {
        return _calculateDomainSeparator(block.chainid);
    }

    // Math

    function _rpow(uint256 x, uint256 n) internal pure returns (uint256 z) {
        assembly {
            switch x case 0 {switch n case 0 {z := RAY} default {z := 0}}
            default {
                switch mod(n, 2) case 0 { z := RAY } default { z := x }
                let half := div(RAY, 2)  // for rounding.
                for { n := div(n, 2) } n { n := div(n,2) } {
                    let xx := mul(x, x)
                    if iszero(eq(div(xx, x), x)) { revert(0,0) }
                    let xxRound := add(xx, half)
                    if lt(xxRound, xx) { revert(0,0) }
                    x := div(xxRound, RAY)
                    if mod(n,2) {
                        let zx := mul(z, x)
                        if and(iszero(iszero(x)), iszero(eq(div(zx, x), z))) { revert(0,0) }
                        let zxRound := add(zx, half)
                        if lt(zxRound, zx) { revert(0,0) }
                        z := div(zxRound, RAY)
                    }
                }
            }
        }
    }

    function _divup(uint256 x, uint256 y) internal pure returns (uint256 z) {
        // Note: _divup(0,0) will return 0 differing from natural solidity division
        unchecked {
            z = x != 0 ? ((x - 1) / y) + 1 : 0;
        }
    }

    // --- Admin external functions ---

    function rely(address usr) external auth {
        wards[usr] = 1;
        emit Rely(usr);
    }

    function deny(address usr) external auth {
        wards[usr] = 0;
        emit Deny(usr);
    }

    function file(bytes32 what, uint256 data) external auth {
        if (what == "ssr") {
            require(data >= RAY, "SUsds/wrong-ssr-value");
            require(rho == block.timestamp, "SUsds/chi-not-up-to-date");
            ssr = data;
        } else revert("SUsds/file-unrecognized-param");
        emit File(what, data);
    }

    // --- Savings Rate Accumulation external/internal function ---

    function drip() public returns (uint256 nChi) {
        (uint256 chi_, uint256 rho_) = (chi, rho);
        uint256 diff;
        if (block.timestamp > rho_) {
            nChi = _rpow(ssr, block.timestamp - rho_) * chi_ / RAY;
            uint256 totalSupply_ = totalSupply;
            diff = totalSupply_ * nChi / RAY - totalSupply_ * chi_ / RAY;
            vat.suck(address(vow), address(this), diff * RAY);
            usdsJoin.exit(address(this), diff);
            chi = uint192(nChi); // safe as nChi is limited to maxUint256/RAY (which is < maxUint192)
            rho = uint64(block.timestamp);
        } else {
            nChi = chi_;
        }
        emit Drip(nChi, diff);
    }

    // --- ERC20 Mutations ---

    function transfer(address to, uint256 value) external returns (bool) {
        require(to != address(0) && to != address(this), "SUsds/invalid-address");
        uint256 balance = balanceOf[msg.sender];
        require(balance >= value, "SUsds/insufficient-balance");

        unchecked {
            balanceOf[msg.sender] = balance - value;
            balanceOf[to] += value; // note: we don't need an overflow check here b/c sum of all balances == totalSupply
        }

        emit Transfer(msg.sender, to, value);

        return true;
    }

    function transferFrom(address from, address to, uint256 value) external returns (bool) {
        require(to != address(0) && to != address(this), "SUsds/invalid-address");
        uint256 balance = balanceOf[from];
        require(balance >= value, "SUsds/insufficient-balance");

        if (from != msg.sender) {
            uint256 allowed = allowance[from][msg.sender];
            if (allowed != type(uint256).max) {
                require(allowed >= value, "SUsds/insufficient-allowance");

                unchecked {
                    allowance[from][msg.sender] = allowed - value;
                }
            }
        }

        unchecked {
            balanceOf[from] = balance - value;
            balanceOf[to] += value; // note: we don't need an overflow check here b/c sum of all balances == totalSupply
        }

        emit Transfer(from, to, value);

        return true;
    }

    function approve(address spender, uint256 value) external returns (bool) {
        allowance[msg.sender][spender] = value;

        emit Approval(msg.sender, spender, value);

        return true;
    }

    // --- Mint/Burn Internal ---

    function _mint(uint256 assets, uint256 shares, address receiver) internal {
        require(receiver != address(0) && receiver != address(this), "SUsds/invalid-address");

        usds.transferFrom(msg.sender, address(this), assets);

        unchecked {
            balanceOf[receiver] = balanceOf[receiver] + shares; // note: we don't need an overflow check here b/c balanceOf[receiver] <= totalSupply
            totalSupply = totalSupply + shares; // note: we don't need an overflow check here b/c shares totalSupply will always be <= usds totalSupply
        }

        emit Deposit(msg.sender, receiver, assets, shares);
        emit Transfer(address(0), receiver, shares);
    }

    function _burn(uint256 assets, uint256 shares, address receiver, address owner) internal {
        uint256 balance = balanceOf[owner];
        require(balance >= shares, "SUsds/insufficient-balance");

        if (owner != msg.sender) {
            uint256 allowed = allowance[owner][msg.sender];
            if (allowed != type(uint256).max) {
                require(allowed >= shares, "SUsds/insufficient-allowance");

                unchecked {
                    allowance[owner][msg.sender] = allowed - shares;
                }
            }
        }

        unchecked {
            balanceOf[owner] = balance - shares; // note: we don't need overflow checks b/c require(balance >= shares) and balance <= totalSupply
            totalSupply      = totalSupply - shares;
        }

        usds.transfer(receiver, assets);

        emit Transfer(owner, address(0), shares);
        emit Withdraw(msg.sender, receiver, owner, assets, shares);
    }

    // --- ERC-4626 ---

    function asset() external view returns (address) {
        return address(usds);
    }

    function totalAssets() external view returns (uint256) {
        return convertToAssets(totalSupply);
    }

    function convertToShares(uint256 assets) public view returns (uint256) {
        uint256 chi_ = (block.timestamp > rho) ? _rpow(ssr, block.timestamp - rho) * chi / RAY : chi;
        return assets * RAY / chi_;
    }

    function convertToAssets(uint256 shares) public view returns (uint256) {
        uint256 chi_ = (block.timestamp > rho) ? _rpow(ssr, block.timestamp - rho) * chi / RAY : chi;
        return shares * chi_ / RAY;
    }

    function maxDeposit(address) external pure returns (uint256) {
        return type(uint256).max;
    }

    function previewDeposit(uint256 assets) external view returns (uint256) {
        return convertToShares(assets);
    }

    function deposit(uint256 assets, address receiver) public returns (uint256 shares) {
        shares = assets * RAY / drip();
        _mint(assets, shares, receiver);
    }

    function deposit(uint256 assets, address receiver, uint16 referral) external returns (uint256 shares) {
        shares = deposit(assets, receiver);
        emit Referral(referral, receiver, assets, shares);
    }

    function maxMint(address) external pure returns (uint256) {
        return type(uint256).max;
    }

    function previewMint(uint256 shares) external view returns (uint256) {
        uint256 chi_ = (block.timestamp > rho) ? _rpow(ssr, block.timestamp - rho) * chi / RAY : chi;
        return _divup(shares * chi_, RAY);
    }

    function mint(uint256 shares, address receiver) public returns (uint256 assets) {
        assets = _divup(shares * drip(), RAY);
        _mint(assets, shares, receiver);
    }

    function mint(uint256 shares, address receiver, uint16 referral) external returns (uint256 assets) {
        assets = mint(shares, receiver);
        emit Referral(referral, receiver, assets, shares);
    }

    function maxWithdraw(address owner) external view returns (uint256) {
        return convertToAssets(balanceOf[owner]);
    }

    function previewWithdraw(uint256 assets) external view returns (uint256) {
        uint256 chi_ = (block.timestamp > rho) ? _rpow(ssr, block.timestamp - rho) * chi / RAY : chi;
        return _divup(assets * RAY, chi_);
    }

    function withdraw(uint256 assets, address receiver, address owner) external returns (uint256 shares) {
        shares = _divup(assets * RAY, drip());
        _burn(assets, shares, receiver, owner);
    }

    function maxRedeem(address owner) external view returns (uint256) {
        return balanceOf[owner];
    }

    function previewRedeem(uint256 shares) external view returns (uint256) {
        return convertToAssets(shares);
    }

    function redeem(uint256 shares, address receiver, address owner) external returns (uint256 assets) {
        assets = shares * drip() / RAY;
        _burn(assets, shares, receiver, owner);
    }

    // --- Approve by signature ---

    function _isValidSignature(
        address signer,
        bytes32 digest,
        bytes memory signature
    ) internal view returns (bool valid) {
        if (signature.length == 65) {
            bytes32 r;
            bytes32 s;
            uint8 v;
            assembly {
                r := mload(add(signature, 0x20))
                s := mload(add(signature, 0x40))
                v := byte(0, mload(add(signature, 0x60)))
            }
            if (signer == ecrecover(digest, v, r, s)) {
                return true;
            }
        }

        if (signer.code.length > 0) {
            (bool success, bytes memory result) = signer.staticcall(
                abi.encodeCall(IERC1271.isValidSignature, (digest, signature))
            );
            valid = (success &&
                result.length == 32 &&
                abi.decode(result, (bytes4)) == IERC1271.isValidSignature.selector);
        }
    }

    function permit(
        address owner,
        address spender,
        uint256 value,
        uint256 deadline,
        bytes memory signature
    ) public {
        require(block.timestamp <= deadline, "SUsds/permit-expired");
        require(owner != address(0), "SUsds/invalid-owner");

        uint256 nonce;
        unchecked { nonce = nonces[owner]++; }

        bytes32 digest =
            keccak256(abi.encodePacked(
                "\x19\x01",
                _calculateDomainSeparator(block.chainid),
                keccak256(abi.encode(
                    PERMIT_TYPEHASH,
                    owner,
                    spender,
                    value,
                    nonce,
                    deadline
                ))
            ));

        require(_isValidSignature(owner, digest, signature), "SUsds/invalid-permit");

        allowance[owner][spender] = value;
        emit Approval(owner, spender, value);
    }

    function permit(
        address owner,
        address spender,
        uint256 value,
        uint256 deadline,
        uint8 v,
        bytes32 r,
        bytes32 s
    ) external {
        permit(owner, spender, value, deadline, abi.encodePacked(r, s, v));
    }
}
*/