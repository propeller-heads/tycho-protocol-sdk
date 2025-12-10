use substreams::hex;

// LiquidityPool Stoage Layout: https://repo.sourcify.dev/1/0xA5C1ddD9185901E3c05E0660126627E039D0a626
const TOTAL_VALUE_OUT_OF_LP_SLOT: StorageLocation = StorageLocation {
    name: "totalValueOutOfLp",
    slot: hex!("00000000000000000000000000000000000000000000000000000000000000cf"),
    offset: 0,
    number_of_bytes: 16,
    signed: false,
};

const TOTAL_VALUE_IN_LP_SLOT: StorageLocation = StorageLocation {
    name: "totalValueInLp",
    slot: hex!("00000000000000000000000000000000000000000000000000000000000000cf"),
    offset: 16,
    number_of_bytes: 16,
    signed: false,
};

const ETH_AMOUNT_LOCKED_FOR_WITHDRAWL_SLOT: StorageLocation = StorageLocation {
    name: "ethAmountLockedForWithdrawl",
    slot: hex!("00000000000000000000000000000000000000000000000000000000000000dc"),
    offset: 1,
    number_of_bytes: 16,
    signed: false,
};

// eETH Storage Layout: https://repo.sourcify.dev/1/0xcb3d917a965a70214f430a135154cd5adda2ad84
const E_ETH_TOTAL_SUPPLY_SLOT: StorageLocation = StorageLocation {
    name: "totalShares",
    slot: hex!("00000000000000000000000000000000000000000000000000000000000000ca"),
    offset: 0,
    number_of_bytes: 32,
    signed: false,
};

pub(crate) const TICKS_MAP_SLOT: [u8; 32] =
    hex!("0000000000000000000000000000000000000000000000000000000000000005");

pub(crate) const LIQUIDITY_POOL_TRACKED_SLOTS: [StorageLocation; 3] = [
    TOTAL_VALUE_OUT_OF_LP_SLOT,
    TOTAL_VALUE_IN_LP_SLOT,
    ETH_AMOUNT_LOCKED_FOR_WITHDRAWL_SLOT,
];

#[derive(Clone)]
pub struct StorageLocation<'a> {
    pub name: &'a str,
    pub slot: [u8; 32],
    pub offset: usize,
    pub number_of_bytes: usize,
    pub signed: bool,
}

pub fn read_bytes(buf: &[u8], offset: usize, number_of_bytes: usize) -> &[u8] {
    let buf_length = buf.len();
    if buf_length < number_of_bytes {
        panic!(
            "attempting to read {number_of_bytes} bytes in buffer  size {buf_size}",
            number_of_bytes = number_of_bytes,
            buf_size = buf.len()
        )
    }

    if offset > (buf_length - 1) {
        panic!(
            "offset {offset} exceeds buffer size {buf_size}",
            offset = offset,
            buf_size = buf.len()
        )
    }

    let end = buf_length - 1 - offset;
    let start_opt = (end + 1).checked_sub(number_of_bytes);
    if start_opt.is_none() {
        panic!(
            "number of bytes {number_of_bytes} with offset {offset} exceeds buffer size
{buf_size}",
            number_of_bytes = number_of_bytes,
            offset = offset,
            buf_size = buf.len()
        )
    }
    let start = start_opt.unwrap();

    &buf[start..=end]
}
