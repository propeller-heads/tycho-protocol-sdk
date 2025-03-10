// SPDX-License-Identifier: GPL-2.0-or-later
pragma solidity >=0.4.0;

interface ICurvePool {
    function initialize(
        string memory _name,
        string memory _symbol,
        address _coin,
        uint256 _decimals,
        uint256 _A,
        uint256 _fee,
        address _admin
    ) external;
    function decimals() external view returns (uint256);
    function transfer(address _to, uint256 _value) external returns (bool);
    function transferFrom(address _from, address _to, uint256 _value)
        external
        returns (bool);
    function approve(address _spender, uint256 _value)
        external
        returns (bool);
    function get_previous_balances()
        external
        view
        returns (uint256[2] memory);
    function get_balances() external view returns (uint256[2] memory);
    function get_twap_balances(
        uint256[2] memory _first_balances,
        uint256[2] memory _last_balances,
        uint256 _time_elapsed
    ) external view returns (uint256[2] memory);
    function get_price_cumulative_last()
        external
        view
        returns (uint256[2] memory);
    function admin_fee() external view returns (uint256);
    function A() external view returns (uint256);
    function A_precise() external view returns (uint256);
    function get_virtual_price() external view returns (uint256);
    function calc_token_amount(uint256[2] memory _amounts, bool _is_deposit)
        external
        view
        returns (uint256);
    function calc_token_amount(
        uint256[2] memory _amounts,
        bool _is_deposit,
        bool _previous
    ) external view returns (uint256);
    function add_liquidity(uint256[2] memory _amounts, uint256 _min_mint_amount)
        external
        returns (uint256);
    function add_liquidity(
        uint256[2] memory _amounts,
        uint256 _min_mint_amount,
        address _receiver
    ) external returns (uint256);
    function get_dy(int128 i, int128 j, uint256 dx)
        external
        view
        returns (uint256);
    function get_dy(int128 i, int128 j, uint256 dx, uint256[2] memory _balances)
        external
        view
        returns (uint256);
    function get_dy_underlying(int128 i, int128 j, uint256 dx)
        external
        view
        returns (uint256);
    function get_dy_underlying(
        int128 i,
        int128 j,
        uint256 dx,
        uint256[2] memory _balances
    ) external view returns (uint256);
    function exchange(uint256 i, uint256 j, uint256 dx, uint256 min_dy)
        external;
    function exchange(
        uint256 i,
        uint256 j,
        uint256 dx,
        uint256 min_dy,
        address _receiver
    ) external;
    function exchange(int128 i, int128 j, uint256 dx, uint256 min_dy)
        external;
    function exchange(
        int128 i,
        int128 j,
        uint256 dx,
        uint256 min_dy,
        address _receiver
    ) external;
    function exchange_underlying(
        uint256 i,
        uint256 j,
        uint256 dx,
        uint256 min_dy
    ) external;
    function exchange_underlying(
        uint256 i,
        uint256 j,
        uint256 dx,
        uint256 min_dy,
        address _receiver
    ) external;
    function exchange_underlying(int128 i, int128 j, uint256 dx, uint256 min_dy)
        external;
    function exchange_underlying(
        int128 i,
        int128 j,
        uint256 dx,
        uint256 min_dy,
        address _receiver
    ) external;
    function remove_liquidity(
        uint256 _burn_amount,
        uint256[2] memory _min_amounts
    ) external returns (uint256[2] memory);
    function remove_liquidity(
        uint256 _burn_amount,
        uint256[2] memory _min_amounts,
        address _receiver
    ) external returns (uint256[2] memory);
    function remove_liquidity_imbalance(
        uint256[2] memory _amounts,
        uint256 _max_burn_amount
    ) external returns (uint256);
    function remove_liquidity_imbalance(
        uint256[2] memory _amounts,
        uint256 _max_burn_amount,
        address _receiver
    ) external returns (uint256);
    function calc_withdraw_one_coin(uint256 _burn_amount, int128 i)
        external
        view
        returns (uint256);
    function calc_withdraw_one_coin(
        uint256 _burn_amount,
        int128 i,
        bool _previous
    ) external view returns (uint256);
    function remove_liquidity_one_coin(
        uint256 _burn_amount,
        int128 i,
        uint256 _min_received
    ) external returns (uint256);
    function remove_liquidity_one_coin(
        uint256 _burn_amount,
        int128 i,
        uint256 _min_received,
        address _receiver
    ) external returns (uint256);
    function ramp_A(uint256 _future_A, uint256 _future_time) external;
    function stop_ramp_A() external;
    function admin_balances(uint256 i) external view returns (uint256);
    function withdraw_admin_fees() external;
    function admin() external view returns (address);
    function coins(uint256 arg0) external view returns (address);
    function balances(uint256 arg0) external view returns (uint256);
    function fee() external view returns (uint256);
    function block_timestamp_last() external view returns (uint256);
    function initial_A() external view returns (uint256);
    function future_A() external view returns (uint256);
    function initial_A_time() external view returns (uint256);
    function future_A_time() external view returns (uint256);
    function name() external view returns (string memory);
    function symbol() external view returns (string memory);
    function balanceOf(address arg0) external view returns (uint256);
    function allowance(address arg0, address arg1)
        external
        view
        returns (uint256);
    function totalSupply() external view returns (uint256);
}
