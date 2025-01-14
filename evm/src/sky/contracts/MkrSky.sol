// SPDX-License-Identifier: AGPL-3.0-or-later

pragma solidity ^0.8.21;

interface GemLike {
    function burn(address, uint256) external;
    function mint(address, uint256) external;
}

contract MkrSky {
    GemLike public immutable mkr;
    GemLike public immutable sky;
    uint256 public immutable rate;
    
    event MkrToSky(address indexed caller, address indexed usr, uint256 mkrAmt, uint256 skyAmt);
    event SkyToMkr(address indexed caller, address indexed usr, uint256 skyAmt, uint256 mkrAmt);

    constructor(address mkr_, address sky_, uint256 rate_) {
        mkr  = GemLike(mkr_);
        sky  = GemLike(sky_);
        rate = rate_; 
    }

    function mkrToSky(address usr, uint256 mkrAmt) external {
        mkr.burn(msg.sender, mkrAmt);
        uint256 skyAmt = mkrAmt * rate;
        sky.mint(usr, skyAmt);
        emit MkrToSky(msg.sender, usr, mkrAmt, skyAmt);
    }

    function skyToMkr(address usr, uint256 skyAmt) external {
        sky.burn(msg.sender, skyAmt);
        uint256 mkrAmt = skyAmt / rate; // Rounding down, dust will be lost if it is not multiple of rate
        mkr.mint(usr, mkrAmt);
        emit SkyToMkr(msg.sender, usr, skyAmt, mkrAmt);
    }
}