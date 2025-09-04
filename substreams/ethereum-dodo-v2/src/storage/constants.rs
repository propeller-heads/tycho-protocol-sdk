use super::pool_storage::StorageLocation;
use hex_literal::hex;

const GSP_B_SLOT: StorageLocation = StorageLocation {
    name: "B",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000005"),
    offset: 0,
    number_of_bytes: 14,
    signed: false,
};

const GSP_Q_SLOT: StorageLocation = StorageLocation {
    name: "Q",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000005"),
    offset: 14,
    number_of_bytes: 14,
    signed: false,
};

const GSP_B0_SLOT: StorageLocation = StorageLocation {
    name: "B0",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000007"),
    offset: 0,
    number_of_bytes: 14,
    signed: false,
};

const GSP_Q0_SLOT: StorageLocation = StorageLocation {
    name: "Q0",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000007"),
    offset: 14,
    number_of_bytes: 14,
    signed: false,
};

const GSP_R_SLOT: StorageLocation = StorageLocation {
    name: "R",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000007"),
    offset: 28,
    number_of_bytes: 4,
    signed: false,
};

const GSP_K_SLOT: StorageLocation = StorageLocation {
    name: "K",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000012"),
    offset: 0,
    number_of_bytes: 32,
    signed: false,
};

const GSP_I_SLOT: StorageLocation = StorageLocation {
    name: "I",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000013"),
    offset: 0,
    number_of_bytes: 32,
    signed: false,
};

const GSP_MT_FEE_RATE_SLOT: StorageLocation = StorageLocation {
    name: "MT_FEE_RATE",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000010"),
    offset: 0,
    number_of_bytes: 32,
    signed: false,
};

const GSP_LP_FEE_RATE_SLOT: StorageLocation = StorageLocation {
    name: "LP_FEE_RATE",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000011"),
    offset: 0,
    number_of_bytes: 32,
    signed: false,
};

const GSP_MT_FEE_BASE_SLOT: StorageLocation = StorageLocation {
    name: "MT_FEE_BASE",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000015"),
    offset: 0,
    number_of_bytes: 32,
    signed: false,
};

const GSP_MT_FEE_QUOTE_SLOT: StorageLocation = StorageLocation {
    name: "MT_FEE_QUOTE",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000016"),
    offset: 0,
    number_of_bytes: 32,
    signed: false,
};

pub(crate) const GSP_PMM_SLOTS: [StorageLocation; 11] = [
    GSP_B_SLOT,
    GSP_Q_SLOT,
    GSP_B0_SLOT,
    GSP_Q0_SLOT,
    GSP_R_SLOT,
    GSP_K_SLOT,
    GSP_I_SLOT,
    GSP_MT_FEE_RATE_SLOT,
    GSP_LP_FEE_RATE_SLOT,
    GSP_MT_FEE_BASE_SLOT,
    GSP_MT_FEE_QUOTE_SLOT,
];

const DSP_B_SLOT: StorageLocation = StorageLocation {
    name: "B",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000003"),
    offset: 0,
    number_of_bytes: 14,
    signed: false,
};

const DSP_Q_SLOT: StorageLocation = StorageLocation {
    name: "Q",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000003"),
    offset: 14,
    number_of_bytes: 14,
    signed: false,
};

const DSP_B0_SLOT: StorageLocation = StorageLocation {
    name: "B0",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000005"),
    offset: 0,
    number_of_bytes: 14,
    signed: false,
};

const DSP_Q0_SLOT: StorageLocation = StorageLocation {
    name: "Q0",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000005"),
    offset: 14,
    number_of_bytes: 14,
    signed: false,
};

const DSP_R_SLOT: StorageLocation = StorageLocation {
    name: "R",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000005"),
    offset: 28,
    number_of_bytes: 4,
    signed: false,
};

const DSP_K_SLOT: StorageLocation = StorageLocation {
    name: "K",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000010"),
    offset: 0,
    number_of_bytes: 32,
    signed: false,
};

pub(crate) const DSP_PMM_SLOTS: [StorageLocation; 6] =
    [DSP_B_SLOT, DSP_Q_SLOT, DSP_B0_SLOT, DSP_Q0_SLOT, DSP_R_SLOT, DSP_K_SLOT];

const DPP_B_SLOT: StorageLocation = StorageLocation {
    name: "B",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000005"),
    offset: 0,
    number_of_bytes: 14,
    signed: false,
};

const DPP_Q_SLOT: StorageLocation = StorageLocation {
    name: "Q",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000005"),
    offset: 14,
    number_of_bytes: 14,
    signed: false,
};

const DPP_B0_SLOT: StorageLocation = StorageLocation {
    name: "B0",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000006"),
    offset: 0,
    number_of_bytes: 14,
    signed: false,
};

const DPP_Q0_SLOT: StorageLocation = StorageLocation {
    name: "Q0",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000006"),
    offset: 14,
    number_of_bytes: 14,
    signed: false,
};

const DPP_R_SLOT: StorageLocation = StorageLocation {
    name: "R",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000006"),
    offset: 28,
    number_of_bytes: 4,
    signed: false,
};

const DPP_K_SLOT: StorageLocation = StorageLocation {
    name: "K",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000009"),
    offset: 0,
    number_of_bytes: 8,
    signed: false,
};

const DPP_I_SLOT: StorageLocation = StorageLocation {
    name: "I",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000009"),
    offset: 8,
    number_of_bytes: 8,
    signed: false,
};

pub(crate) const DPP_PMM_SLOTS: [StorageLocation; 7] =
    [DPP_B_SLOT, DPP_Q_SLOT, DPP_B0_SLOT, DPP_Q0_SLOT, DPP_R_SLOT, DPP_K_SLOT, DPP_I_SLOT];

const DVM_B_SLOT: StorageLocation = StorageLocation {
    name: "B",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000003"),
    offset: 0,
    number_of_bytes: 14,
    signed: false,
};

const DVM_Q_SLOT: StorageLocation = StorageLocation {
    name: "Q",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000003"),
    offset: 14,
    number_of_bytes: 14,
    signed: false,
};

const DVM_K_SLOT: StorageLocation = StorageLocation {
    name: "K",
    slot: hex!("000000000000000000000000000000000000000000000000000000000000000f"),
    offset: 0,
    number_of_bytes: 32,
    signed: false,
};

const DVM_I_SLOT: StorageLocation = StorageLocation {
    name: "I",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000010"),
    offset: 0,
    number_of_bytes: 32,
    signed: false,
};

pub(crate) const DVM_PMM_SLOTS: [StorageLocation; 4] =
    [DVM_B_SLOT, DVM_Q_SLOT, DVM_K_SLOT, DVM_I_SLOT];
