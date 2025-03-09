import { IFluidDexT1 } from "./iDexT1.sol";
import { Structs } from "./structs.sol";

interface FluidDexReservesResolver is Structs{
    function getPoolAddress(uint256 poolId_) external view returns (address pool_);
    function getTotalPools() external view returns (uint);
    function getAllPoolAddresses() external view returns (address[] memory pools_);
    function getDexCollateralReserves(address dex_) external returns (IFluidDexT1.CollateralReserves memory reserves_);
    function getDexCollateralReservesAdjusted(address dex_) external returns (IFluidDexT1.CollateralReserves memory reserves_);
    function getDexDebtReserves(address dex_) external returns (IFluidDexT1.DebtReserves memory reserves_);
    function getDexDebtReservesAdjusted(address dex_) external returns (IFluidDexT1.DebtReserves memory reserves_);
    function getPoolConstantsView(address pool_) external view returns (IFluidDexT1.ConstantViews memory constantsView_);
    function getPoolConstantsView2(address pool_) external view returns (IFluidDexT1.ConstantViews2 memory constantsView2_);
    function getPoolTokens(address pool_) external view returns (address token0_, address token1_);
    function getDexLimits(address dex_) external view returns (Structs.DexLimits memory limits_);
    function estimateSwapIn(address dex_, bool swap0to1_, uint256 amountIn_, uint256 amountOutMin_) external payable returns (uint256 amountOut_);
    function estimateSwapOut(address dex_, bool swap0to1_, uint256 amountOut_, uint256 amountInMax_) external payable returns (uint256 amountIn_);
    function getPool(uint256 poolId_) external view returns (Structs.Pool memory pool_);
    function getPoolFee(address pool_) external view returns (uint256 fee_);
    function getAllPools() external view returns (Structs.Pool[] memory pools_);
    function getPoolReserves(address pool_) external returns (Structs.PoolWithReserves memory poolReserves_);
    function getPoolsReserves(address[] memory pools_) external returns (Structs.PoolWithReserves[] memory poolsReserves_);
    function getAllPoolsReserves() external returns (Structs.PoolWithReserves[] memory poolsReserves_);
    function getPoolReservesAdjusted(address pool_) external returns (Structs.PoolWithReserves memory poolReserves_);
    function getPoolsReservesAdjusted(address[] memory pools_) external returns (Structs.PoolWithReserves[] memory poolsReserves_);
    function getAllPoolsReservesAdjusted() external returns (Structs.PoolWithReserves[] memory poolsReserves_);
}
