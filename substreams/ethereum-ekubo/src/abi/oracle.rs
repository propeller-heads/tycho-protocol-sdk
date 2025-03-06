    const INTERNAL_ERR: &'static str = "`ethabi_derive` internal error";
    /// Contract's functions.
    #[allow(dead_code, unused_imports, unused_variables)]
    pub mod functions {
        use super::INTERNAL_ERR;
        #[derive(Debug, Clone, PartialEq)]
        pub struct AfterCollectFees {
            pub param0: Vec<u8>,
            pub param1: (Vec<u8>, Vec<u8>, [u8; 32usize]),
            pub param2: [u8; 32usize],
            pub param3: (substreams::scalar::BigInt, substreams::scalar::BigInt),
            pub param4: substreams::scalar::BigInt,
            pub param5: substreams::scalar::BigInt,
        }
        impl AfterCollectFees {
            const METHOD_ID: [u8; 4] = [205u8, 208u8, 250u8, 14u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Address,
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::Address, ethabi::ParamType::Address,
                                    ethabi::ParamType::FixedBytes(32usize)
                                ],
                            ),
                            ethabi::ParamType::FixedBytes(32usize),
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::Int(32usize),
                                    ethabi::ParamType::Int(32usize)
                                ],
                            ),
                            ethabi::ParamType::Uint(128usize),
                            ethabi::ParamType::Uint(128usize),
                        ],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    param0: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    param1: {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            tuple_elements[0usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            tuple_elements[1usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            {
                                let mut result = [0u8; 32];
                                let v = tuple_elements[2usize]
                                    .clone()
                                    .into_fixed_bytes()
                                    .expect(INTERNAL_ERR);
                                result.copy_from_slice(&v);
                                result
                            },
                        )
                    },
                    param2: {
                        let mut result = [0u8; 32];
                        let v = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_fixed_bytes()
                            .expect(INTERNAL_ERR);
                        result.copy_from_slice(&v);
                        result
                    },
                    param3: {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[0usize]
                                    .clone()
                                    .into_int()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_signed_bytes_be(&v)
                            },
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[1usize]
                                    .clone()
                                    .into_int()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_signed_bytes_be(&v)
                            },
                        )
                    },
                    param4: {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                    param5: {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[
                        ethabi::Token::Address(
                            ethabi::Address::from_slice(&self.param0),
                        ),
                        ethabi::Token::Tuple(
                            vec![
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .param1.0)),
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .param1.1)), ethabi::Token::FixedBytes(self.param1.2
                                .as_ref().to_vec())
                            ],
                        ),
                        ethabi::Token::FixedBytes(self.param2.as_ref().to_vec()),
                        ethabi::Token::Tuple(
                            vec![
                                { let non_full_signed_bytes = self.param3.0
                                .to_signed_bytes_be(); let full_signed_bytes_init = if
                                non_full_signed_bytes[0] & 0x80 == 0x80 { 0xff } else { 0x00
                                }; let mut full_signed_bytes = [full_signed_bytes_init as
                                u8; 32]; non_full_signed_bytes.into_iter().rev().enumerate()
                                .for_each(| (i, byte) | full_signed_bytes[31 - i] = byte);
                                ethabi::Token::Int(ethabi::Int::from_big_endian(full_signed_bytes
                                .as_ref())) }, { let non_full_signed_bytes = self.param3.1
                                .to_signed_bytes_be(); let full_signed_bytes_init = if
                                non_full_signed_bytes[0] & 0x80 == 0x80 { 0xff } else { 0x00
                                }; let mut full_signed_bytes = [full_signed_bytes_init as
                                u8; 32]; non_full_signed_bytes.into_iter().rev().enumerate()
                                .for_each(| (i, byte) | full_signed_bytes[31 - i] = byte);
                                ethabi::Token::Int(ethabi::Int::from_big_endian(full_signed_bytes
                                .as_ref())) }
                            ],
                        ),
                        ethabi::Token::Uint(
                            ethabi::Uint::from_big_endian(
                                match self.param4.clone().to_bytes_be() {
                                    (num_bigint::Sign::Plus, bytes) => bytes,
                                    (num_bigint::Sign::NoSign, bytes) => bytes,
                                    (num_bigint::Sign::Minus, _) => {
                                        panic!("negative numbers are not supported")
                                    }
                                }
                                    .as_slice(),
                            ),
                        ),
                        ethabi::Token::Uint(
                            ethabi::Uint::from_big_endian(
                                match self.param5.clone().to_bytes_be() {
                                    (num_bigint::Sign::Plus, bytes) => bytes,
                                    (num_bigint::Sign::NoSign, bytes) => bytes,
                                    (num_bigint::Sign::Minus, _) => {
                                        panic!("negative numbers are not supported")
                                    }
                                }
                                    .as_slice(),
                            ),
                        ),
                    ],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
        }
        impl substreams_ethereum::Function for AfterCollectFees {
            const NAME: &'static str = "afterCollectFees";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct AfterInitializePool {
            pub param0: Vec<u8>,
            pub param1: (Vec<u8>, Vec<u8>, [u8; 32usize]),
            pub param2: substreams::scalar::BigInt,
            pub param3: substreams::scalar::BigInt,
        }
        impl AfterInitializePool {
            const METHOD_ID: [u8; 4] = [148u8, 131u8, 116u8, 255u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Address,
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::Address, ethabi::ParamType::Address,
                                    ethabi::ParamType::FixedBytes(32usize)
                                ],
                            ),
                            ethabi::ParamType::Int(32usize),
                            ethabi::ParamType::Uint(96usize),
                        ],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    param0: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    param1: {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            tuple_elements[0usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            tuple_elements[1usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            {
                                let mut result = [0u8; 32];
                                let v = tuple_elements[2usize]
                                    .clone()
                                    .into_fixed_bytes()
                                    .expect(INTERNAL_ERR);
                                result.copy_from_slice(&v);
                                result
                            },
                        )
                    },
                    param2: {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_int()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_signed_bytes_be(&v)
                    },
                    param3: {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[
                        ethabi::Token::Address(
                            ethabi::Address::from_slice(&self.param0),
                        ),
                        ethabi::Token::Tuple(
                            vec![
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .param1.0)),
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .param1.1)), ethabi::Token::FixedBytes(self.param1.2
                                .as_ref().to_vec())
                            ],
                        ),
                        {
                            let non_full_signed_bytes = self.param2.to_signed_bytes_be();
                            let full_signed_bytes_init = if non_full_signed_bytes[0]
                                & 0x80 == 0x80
                            {
                                0xff
                            } else {
                                0x00
                            };
                            let mut full_signed_bytes = [full_signed_bytes_init
                                as u8; 32];
                            non_full_signed_bytes
                                .into_iter()
                                .rev()
                                .enumerate()
                                .for_each(|(i, byte)| full_signed_bytes[31 - i] = byte);
                            ethabi::Token::Int(
                                ethabi::Int::from_big_endian(full_signed_bytes.as_ref()),
                            )
                        },
                        ethabi::Token::Uint(
                            ethabi::Uint::from_big_endian(
                                match self.param3.clone().to_bytes_be() {
                                    (num_bigint::Sign::Plus, bytes) => bytes,
                                    (num_bigint::Sign::NoSign, bytes) => bytes,
                                    (num_bigint::Sign::Minus, _) => {
                                        panic!("negative numbers are not supported")
                                    }
                                }
                                    .as_slice(),
                            ),
                        ),
                    ],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
        }
        impl substreams_ethereum::Function for AfterInitializePool {
            const NAME: &'static str = "afterInitializePool";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct AfterSwap {
            pub param0: Vec<u8>,
            pub param1: (Vec<u8>, Vec<u8>, [u8; 32usize]),
            pub param2: (
                substreams::scalar::BigInt,
                bool,
                substreams::scalar::BigInt,
                substreams::scalar::BigInt,
            ),
            pub param3: substreams::scalar::BigInt,
            pub param4: substreams::scalar::BigInt,
        }
        impl AfterSwap {
            const METHOD_ID: [u8; 4] = [189u8, 255u8, 60u8, 4u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Address,
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::Address, ethabi::ParamType::Address,
                                    ethabi::ParamType::FixedBytes(32usize)
                                ],
                            ),
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::Int(128usize), ethabi::ParamType::Bool,
                                    ethabi::ParamType::Uint(96usize),
                                    ethabi::ParamType::Uint(256usize)
                                ],
                            ),
                            ethabi::ParamType::Int(128usize),
                            ethabi::ParamType::Int(128usize),
                        ],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    param0: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    param1: {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            tuple_elements[0usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            tuple_elements[1usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            {
                                let mut result = [0u8; 32];
                                let v = tuple_elements[2usize]
                                    .clone()
                                    .into_fixed_bytes()
                                    .expect(INTERNAL_ERR);
                                result.copy_from_slice(&v);
                                result
                            },
                        )
                    },
                    param2: {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[0usize]
                                    .clone()
                                    .into_int()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_signed_bytes_be(&v)
                            },
                            tuple_elements[1usize]
                                .clone()
                                .into_bool()
                                .expect(INTERNAL_ERR),
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[2usize]
                                    .clone()
                                    .into_uint()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                            },
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[3usize]
                                    .clone()
                                    .into_uint()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                            },
                        )
                    },
                    param3: {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_int()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_signed_bytes_be(&v)
                    },
                    param4: {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_int()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_signed_bytes_be(&v)
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[
                        ethabi::Token::Address(
                            ethabi::Address::from_slice(&self.param0),
                        ),
                        ethabi::Token::Tuple(
                            vec![
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .param1.0)),
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .param1.1)), ethabi::Token::FixedBytes(self.param1.2
                                .as_ref().to_vec())
                            ],
                        ),
                        ethabi::Token::Tuple(
                            vec![
                                { let non_full_signed_bytes = self.param2.0
                                .to_signed_bytes_be(); let full_signed_bytes_init = if
                                non_full_signed_bytes[0] & 0x80 == 0x80 { 0xff } else { 0x00
                                }; let mut full_signed_bytes = [full_signed_bytes_init as
                                u8; 32]; non_full_signed_bytes.into_iter().rev().enumerate()
                                .for_each(| (i, byte) | full_signed_bytes[31 - i] = byte);
                                ethabi::Token::Int(ethabi::Int::from_big_endian(full_signed_bytes
                                .as_ref())) }, ethabi::Token::Bool(self.param2.1.clone()),
                                ethabi::Token::Uint(ethabi::Uint::from_big_endian(match self
                                .param2.2.clone().to_bytes_be() { (num_bigint::Sign::Plus,
                                bytes) => bytes, (num_bigint::Sign::NoSign, bytes) => bytes,
                                (num_bigint::Sign::Minus, _) => {
                                panic!("negative numbers are not supported") }, }
                                .as_slice(),),),
                                ethabi::Token::Uint(ethabi::Uint::from_big_endian(match self
                                .param2.3.clone().to_bytes_be() { (num_bigint::Sign::Plus,
                                bytes) => bytes, (num_bigint::Sign::NoSign, bytes) => bytes,
                                (num_bigint::Sign::Minus, _) => {
                                panic!("negative numbers are not supported") }, }
                                .as_slice(),),)
                            ],
                        ),
                        {
                            let non_full_signed_bytes = self.param3.to_signed_bytes_be();
                            let full_signed_bytes_init = if non_full_signed_bytes[0]
                                & 0x80 == 0x80
                            {
                                0xff
                            } else {
                                0x00
                            };
                            let mut full_signed_bytes = [full_signed_bytes_init
                                as u8; 32];
                            non_full_signed_bytes
                                .into_iter()
                                .rev()
                                .enumerate()
                                .for_each(|(i, byte)| full_signed_bytes[31 - i] = byte);
                            ethabi::Token::Int(
                                ethabi::Int::from_big_endian(full_signed_bytes.as_ref()),
                            )
                        },
                        {
                            let non_full_signed_bytes = self.param4.to_signed_bytes_be();
                            let full_signed_bytes_init = if non_full_signed_bytes[0]
                                & 0x80 == 0x80
                            {
                                0xff
                            } else {
                                0x00
                            };
                            let mut full_signed_bytes = [full_signed_bytes_init
                                as u8; 32];
                            non_full_signed_bytes
                                .into_iter()
                                .rev()
                                .enumerate()
                                .for_each(|(i, byte)| full_signed_bytes[31 - i] = byte);
                            ethabi::Token::Int(
                                ethabi::Int::from_big_endian(full_signed_bytes.as_ref()),
                            )
                        },
                    ],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
        }
        impl substreams_ethereum::Function for AfterSwap {
            const NAME: &'static str = "afterSwap";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct AfterUpdatePosition {
            pub param0: Vec<u8>,
            pub param1: (Vec<u8>, Vec<u8>, [u8; 32usize]),
            pub param2: (
                [u8; 32usize],
                (substreams::scalar::BigInt, substreams::scalar::BigInt),
                substreams::scalar::BigInt,
            ),
            pub param3: substreams::scalar::BigInt,
            pub param4: substreams::scalar::BigInt,
        }
        impl AfterUpdatePosition {
            const METHOD_ID: [u8; 4] = [60u8, 133u8, 229u8, 161u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Address,
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::Address, ethabi::ParamType::Address,
                                    ethabi::ParamType::FixedBytes(32usize)
                                ],
                            ),
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::FixedBytes(32usize),
                                    ethabi::ParamType::Tuple(vec![ethabi::ParamType::Int(32usize),
                                    ethabi::ParamType::Int(32usize)]),
                                    ethabi::ParamType::Int(128usize)
                                ],
                            ),
                            ethabi::ParamType::Int(128usize),
                            ethabi::ParamType::Int(128usize),
                        ],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    param0: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    param1: {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            tuple_elements[0usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            tuple_elements[1usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            {
                                let mut result = [0u8; 32];
                                let v = tuple_elements[2usize]
                                    .clone()
                                    .into_fixed_bytes()
                                    .expect(INTERNAL_ERR);
                                result.copy_from_slice(&v);
                                result
                            },
                        )
                    },
                    param2: {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            {
                                let mut result = [0u8; 32];
                                let v = tuple_elements[0usize]
                                    .clone()
                                    .into_fixed_bytes()
                                    .expect(INTERNAL_ERR);
                                result.copy_from_slice(&v);
                                result
                            },
                            {
                                let tuple_elements = tuple_elements[1usize]
                                    .clone()
                                    .into_tuple()
                                    .expect(INTERNAL_ERR);
                                (
                                    {
                                        let mut v = [0 as u8; 32];
                                        tuple_elements[0usize]
                                            .clone()
                                            .into_int()
                                            .expect(INTERNAL_ERR)
                                            .to_big_endian(v.as_mut_slice());
                                        substreams::scalar::BigInt::from_signed_bytes_be(&v)
                                    },
                                    {
                                        let mut v = [0 as u8; 32];
                                        tuple_elements[1usize]
                                            .clone()
                                            .into_int()
                                            .expect(INTERNAL_ERR)
                                            .to_big_endian(v.as_mut_slice());
                                        substreams::scalar::BigInt::from_signed_bytes_be(&v)
                                    },
                                )
                            },
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[2usize]
                                    .clone()
                                    .into_int()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_signed_bytes_be(&v)
                            },
                        )
                    },
                    param3: {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_int()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_signed_bytes_be(&v)
                    },
                    param4: {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_int()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_signed_bytes_be(&v)
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[
                        ethabi::Token::Address(
                            ethabi::Address::from_slice(&self.param0),
                        ),
                        ethabi::Token::Tuple(
                            vec![
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .param1.0)),
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .param1.1)), ethabi::Token::FixedBytes(self.param1.2
                                .as_ref().to_vec())
                            ],
                        ),
                        ethabi::Token::Tuple(
                            vec![
                                ethabi::Token::FixedBytes(self.param2.0.as_ref().to_vec()),
                                ethabi::Token::Tuple(vec![{ let non_full_signed_bytes = self
                                .param2.1.0.to_signed_bytes_be(); let full_signed_bytes_init
                                = if non_full_signed_bytes[0] & 0x80 == 0x80 { 0xff } else {
                                0x00 }; let mut full_signed_bytes = [full_signed_bytes_init
                                as u8; 32]; non_full_signed_bytes.into_iter().rev()
                                .enumerate().for_each(| (i, byte) | full_signed_bytes[31 -
                                i] = byte);
                                ethabi::Token::Int(ethabi::Int::from_big_endian(full_signed_bytes
                                .as_ref())) }, { let non_full_signed_bytes = self.param2.1.1
                                .to_signed_bytes_be(); let full_signed_bytes_init = if
                                non_full_signed_bytes[0] & 0x80 == 0x80 { 0xff } else { 0x00
                                }; let mut full_signed_bytes = [full_signed_bytes_init as
                                u8; 32]; non_full_signed_bytes.into_iter().rev().enumerate()
                                .for_each(| (i, byte) | full_signed_bytes[31 - i] = byte);
                                ethabi::Token::Int(ethabi::Int::from_big_endian(full_signed_bytes
                                .as_ref())) }]), { let non_full_signed_bytes = self.param2.2
                                .to_signed_bytes_be(); let full_signed_bytes_init = if
                                non_full_signed_bytes[0] & 0x80 == 0x80 { 0xff } else { 0x00
                                }; let mut full_signed_bytes = [full_signed_bytes_init as
                                u8; 32]; non_full_signed_bytes.into_iter().rev().enumerate()
                                .for_each(| (i, byte) | full_signed_bytes[31 - i] = byte);
                                ethabi::Token::Int(ethabi::Int::from_big_endian(full_signed_bytes
                                .as_ref())) }
                            ],
                        ),
                        {
                            let non_full_signed_bytes = self.param3.to_signed_bytes_be();
                            let full_signed_bytes_init = if non_full_signed_bytes[0]
                                & 0x80 == 0x80
                            {
                                0xff
                            } else {
                                0x00
                            };
                            let mut full_signed_bytes = [full_signed_bytes_init
                                as u8; 32];
                            non_full_signed_bytes
                                .into_iter()
                                .rev()
                                .enumerate()
                                .for_each(|(i, byte)| full_signed_bytes[31 - i] = byte);
                            ethabi::Token::Int(
                                ethabi::Int::from_big_endian(full_signed_bytes.as_ref()),
                            )
                        },
                        {
                            let non_full_signed_bytes = self.param4.to_signed_bytes_be();
                            let full_signed_bytes_init = if non_full_signed_bytes[0]
                                & 0x80 == 0x80
                            {
                                0xff
                            } else {
                                0x00
                            };
                            let mut full_signed_bytes = [full_signed_bytes_init
                                as u8; 32];
                            non_full_signed_bytes
                                .into_iter()
                                .rev()
                                .enumerate()
                                .for_each(|(i, byte)| full_signed_bytes[31 - i] = byte);
                            ethabi::Token::Int(
                                ethabi::Int::from_big_endian(full_signed_bytes.as_ref()),
                            )
                        },
                    ],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
        }
        impl substreams_ethereum::Function for AfterUpdatePosition {
            const NAME: &'static str = "afterUpdatePosition";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct BeforeCollectFees {
            pub param0: Vec<u8>,
            pub param1: (Vec<u8>, Vec<u8>, [u8; 32usize]),
            pub param2: [u8; 32usize],
            pub param3: (substreams::scalar::BigInt, substreams::scalar::BigInt),
        }
        impl BeforeCollectFees {
            const METHOD_ID: [u8; 4] = [111u8, 181u8, 191u8, 227u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Address,
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::Address, ethabi::ParamType::Address,
                                    ethabi::ParamType::FixedBytes(32usize)
                                ],
                            ),
                            ethabi::ParamType::FixedBytes(32usize),
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::Int(32usize),
                                    ethabi::ParamType::Int(32usize)
                                ],
                            ),
                        ],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    param0: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    param1: {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            tuple_elements[0usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            tuple_elements[1usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            {
                                let mut result = [0u8; 32];
                                let v = tuple_elements[2usize]
                                    .clone()
                                    .into_fixed_bytes()
                                    .expect(INTERNAL_ERR);
                                result.copy_from_slice(&v);
                                result
                            },
                        )
                    },
                    param2: {
                        let mut result = [0u8; 32];
                        let v = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_fixed_bytes()
                            .expect(INTERNAL_ERR);
                        result.copy_from_slice(&v);
                        result
                    },
                    param3: {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[0usize]
                                    .clone()
                                    .into_int()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_signed_bytes_be(&v)
                            },
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[1usize]
                                    .clone()
                                    .into_int()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_signed_bytes_be(&v)
                            },
                        )
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[
                        ethabi::Token::Address(
                            ethabi::Address::from_slice(&self.param0),
                        ),
                        ethabi::Token::Tuple(
                            vec![
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .param1.0)),
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .param1.1)), ethabi::Token::FixedBytes(self.param1.2
                                .as_ref().to_vec())
                            ],
                        ),
                        ethabi::Token::FixedBytes(self.param2.as_ref().to_vec()),
                        ethabi::Token::Tuple(
                            vec![
                                { let non_full_signed_bytes = self.param3.0
                                .to_signed_bytes_be(); let full_signed_bytes_init = if
                                non_full_signed_bytes[0] & 0x80 == 0x80 { 0xff } else { 0x00
                                }; let mut full_signed_bytes = [full_signed_bytes_init as
                                u8; 32]; non_full_signed_bytes.into_iter().rev().enumerate()
                                .for_each(| (i, byte) | full_signed_bytes[31 - i] = byte);
                                ethabi::Token::Int(ethabi::Int::from_big_endian(full_signed_bytes
                                .as_ref())) }, { let non_full_signed_bytes = self.param3.1
                                .to_signed_bytes_be(); let full_signed_bytes_init = if
                                non_full_signed_bytes[0] & 0x80 == 0x80 { 0xff } else { 0x00
                                }; let mut full_signed_bytes = [full_signed_bytes_init as
                                u8; 32]; non_full_signed_bytes.into_iter().rev().enumerate()
                                .for_each(| (i, byte) | full_signed_bytes[31 - i] = byte);
                                ethabi::Token::Int(ethabi::Int::from_big_endian(full_signed_bytes
                                .as_ref())) }
                            ],
                        ),
                    ],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
        }
        impl substreams_ethereum::Function for BeforeCollectFees {
            const NAME: &'static str = "beforeCollectFees";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct BeforeInitializePool {
            pub param0: Vec<u8>,
            pub key: (Vec<u8>, Vec<u8>, [u8; 32usize]),
            pub param2: substreams::scalar::BigInt,
        }
        impl BeforeInitializePool {
            const METHOD_ID: [u8; 4] = [31u8, 187u8, 180u8, 98u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Address,
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::Address, ethabi::ParamType::Address,
                                    ethabi::ParamType::FixedBytes(32usize)
                                ],
                            ),
                            ethabi::ParamType::Int(32usize),
                        ],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    param0: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    key: {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            tuple_elements[0usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            tuple_elements[1usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            {
                                let mut result = [0u8; 32];
                                let v = tuple_elements[2usize]
                                    .clone()
                                    .into_fixed_bytes()
                                    .expect(INTERNAL_ERR);
                                result.copy_from_slice(&v);
                                result
                            },
                        )
                    },
                    param2: {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_int()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_signed_bytes_be(&v)
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[
                        ethabi::Token::Address(
                            ethabi::Address::from_slice(&self.param0),
                        ),
                        ethabi::Token::Tuple(
                            vec![
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .key.0)),
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .key.1)), ethabi::Token::FixedBytes(self.key.2.as_ref()
                                .to_vec())
                            ],
                        ),
                        {
                            let non_full_signed_bytes = self.param2.to_signed_bytes_be();
                            let full_signed_bytes_init = if non_full_signed_bytes[0]
                                & 0x80 == 0x80
                            {
                                0xff
                            } else {
                                0x00
                            };
                            let mut full_signed_bytes = [full_signed_bytes_init
                                as u8; 32];
                            non_full_signed_bytes
                                .into_iter()
                                .rev()
                                .enumerate()
                                .for_each(|(i, byte)| full_signed_bytes[31 - i] = byte);
                            ethabi::Token::Int(
                                ethabi::Int::from_big_endian(full_signed_bytes.as_ref()),
                            )
                        },
                    ],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
        }
        impl substreams_ethereum::Function for BeforeInitializePool {
            const NAME: &'static str = "beforeInitializePool";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct BeforeSwap {
            pub param0: Vec<u8>,
            pub pool_key: (Vec<u8>, Vec<u8>, [u8; 32usize]),
            pub params: (
                substreams::scalar::BigInt,
                bool,
                substreams::scalar::BigInt,
                substreams::scalar::BigInt,
            ),
        }
        impl BeforeSwap {
            const METHOD_ID: [u8; 4] = [62u8, 6u8, 223u8, 54u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Address,
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::Address, ethabi::ParamType::Address,
                                    ethabi::ParamType::FixedBytes(32usize)
                                ],
                            ),
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::Int(128usize), ethabi::ParamType::Bool,
                                    ethabi::ParamType::Uint(96usize),
                                    ethabi::ParamType::Uint(256usize)
                                ],
                            ),
                        ],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    param0: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    pool_key: {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            tuple_elements[0usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            tuple_elements[1usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            {
                                let mut result = [0u8; 32];
                                let v = tuple_elements[2usize]
                                    .clone()
                                    .into_fixed_bytes()
                                    .expect(INTERNAL_ERR);
                                result.copy_from_slice(&v);
                                result
                            },
                        )
                    },
                    params: {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[0usize]
                                    .clone()
                                    .into_int()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_signed_bytes_be(&v)
                            },
                            tuple_elements[1usize]
                                .clone()
                                .into_bool()
                                .expect(INTERNAL_ERR),
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[2usize]
                                    .clone()
                                    .into_uint()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                            },
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[3usize]
                                    .clone()
                                    .into_uint()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                            },
                        )
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[
                        ethabi::Token::Address(
                            ethabi::Address::from_slice(&self.param0),
                        ),
                        ethabi::Token::Tuple(
                            vec![
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .pool_key.0)),
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .pool_key.1)), ethabi::Token::FixedBytes(self.pool_key.2
                                .as_ref().to_vec())
                            ],
                        ),
                        ethabi::Token::Tuple(
                            vec![
                                { let non_full_signed_bytes = self.params.0
                                .to_signed_bytes_be(); let full_signed_bytes_init = if
                                non_full_signed_bytes[0] & 0x80 == 0x80 { 0xff } else { 0x00
                                }; let mut full_signed_bytes = [full_signed_bytes_init as
                                u8; 32]; non_full_signed_bytes.into_iter().rev().enumerate()
                                .for_each(| (i, byte) | full_signed_bytes[31 - i] = byte);
                                ethabi::Token::Int(ethabi::Int::from_big_endian(full_signed_bytes
                                .as_ref())) }, ethabi::Token::Bool(self.params.1.clone()),
                                ethabi::Token::Uint(ethabi::Uint::from_big_endian(match self
                                .params.2.clone().to_bytes_be() { (num_bigint::Sign::Plus,
                                bytes) => bytes, (num_bigint::Sign::NoSign, bytes) => bytes,
                                (num_bigint::Sign::Minus, _) => {
                                panic!("negative numbers are not supported") }, }
                                .as_slice(),),),
                                ethabi::Token::Uint(ethabi::Uint::from_big_endian(match self
                                .params.3.clone().to_bytes_be() { (num_bigint::Sign::Plus,
                                bytes) => bytes, (num_bigint::Sign::NoSign, bytes) => bytes,
                                (num_bigint::Sign::Minus, _) => {
                                panic!("negative numbers are not supported") }, }
                                .as_slice(),),)
                            ],
                        ),
                    ],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
        }
        impl substreams_ethereum::Function for BeforeSwap {
            const NAME: &'static str = "beforeSwap";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct BeforeUpdatePosition {
            pub param0: Vec<u8>,
            pub pool_key: (Vec<u8>, Vec<u8>, [u8; 32usize]),
            pub params: (
                [u8; 32usize],
                (substreams::scalar::BigInt, substreams::scalar::BigInt),
                substreams::scalar::BigInt,
            ),
        }
        impl BeforeUpdatePosition {
            const METHOD_ID: [u8; 4] = [177u8, 75u8, 16u8, 109u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Address,
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::Address, ethabi::ParamType::Address,
                                    ethabi::ParamType::FixedBytes(32usize)
                                ],
                            ),
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::FixedBytes(32usize),
                                    ethabi::ParamType::Tuple(vec![ethabi::ParamType::Int(32usize),
                                    ethabi::ParamType::Int(32usize)]),
                                    ethabi::ParamType::Int(128usize)
                                ],
                            ),
                        ],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    param0: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    pool_key: {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            tuple_elements[0usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            tuple_elements[1usize]
                                .clone()
                                .into_address()
                                .expect(INTERNAL_ERR)
                                .as_bytes()
                                .to_vec(),
                            {
                                let mut result = [0u8; 32];
                                let v = tuple_elements[2usize]
                                    .clone()
                                    .into_fixed_bytes()
                                    .expect(INTERNAL_ERR);
                                result.copy_from_slice(&v);
                                result
                            },
                        )
                    },
                    params: {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            {
                                let mut result = [0u8; 32];
                                let v = tuple_elements[0usize]
                                    .clone()
                                    .into_fixed_bytes()
                                    .expect(INTERNAL_ERR);
                                result.copy_from_slice(&v);
                                result
                            },
                            {
                                let tuple_elements = tuple_elements[1usize]
                                    .clone()
                                    .into_tuple()
                                    .expect(INTERNAL_ERR);
                                (
                                    {
                                        let mut v = [0 as u8; 32];
                                        tuple_elements[0usize]
                                            .clone()
                                            .into_int()
                                            .expect(INTERNAL_ERR)
                                            .to_big_endian(v.as_mut_slice());
                                        substreams::scalar::BigInt::from_signed_bytes_be(&v)
                                    },
                                    {
                                        let mut v = [0 as u8; 32];
                                        tuple_elements[1usize]
                                            .clone()
                                            .into_int()
                                            .expect(INTERNAL_ERR)
                                            .to_big_endian(v.as_mut_slice());
                                        substreams::scalar::BigInt::from_signed_bytes_be(&v)
                                    },
                                )
                            },
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[2usize]
                                    .clone()
                                    .into_int()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_signed_bytes_be(&v)
                            },
                        )
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[
                        ethabi::Token::Address(
                            ethabi::Address::from_slice(&self.param0),
                        ),
                        ethabi::Token::Tuple(
                            vec![
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .pool_key.0)),
                                ethabi::Token::Address(ethabi::Address::from_slice(& self
                                .pool_key.1)), ethabi::Token::FixedBytes(self.pool_key.2
                                .as_ref().to_vec())
                            ],
                        ),
                        ethabi::Token::Tuple(
                            vec![
                                ethabi::Token::FixedBytes(self.params.0.as_ref().to_vec()),
                                ethabi::Token::Tuple(vec![{ let non_full_signed_bytes = self
                                .params.1.0.to_signed_bytes_be(); let full_signed_bytes_init
                                = if non_full_signed_bytes[0] & 0x80 == 0x80 { 0xff } else {
                                0x00 }; let mut full_signed_bytes = [full_signed_bytes_init
                                as u8; 32]; non_full_signed_bytes.into_iter().rev()
                                .enumerate().for_each(| (i, byte) | full_signed_bytes[31 -
                                i] = byte);
                                ethabi::Token::Int(ethabi::Int::from_big_endian(full_signed_bytes
                                .as_ref())) }, { let non_full_signed_bytes = self.params.1.1
                                .to_signed_bytes_be(); let full_signed_bytes_init = if
                                non_full_signed_bytes[0] & 0x80 == 0x80 { 0xff } else { 0x00
                                }; let mut full_signed_bytes = [full_signed_bytes_init as
                                u8; 32]; non_full_signed_bytes.into_iter().rev().enumerate()
                                .for_each(| (i, byte) | full_signed_bytes[31 - i] = byte);
                                ethabi::Token::Int(ethabi::Int::from_big_endian(full_signed_bytes
                                .as_ref())) }]), { let non_full_signed_bytes = self.params.2
                                .to_signed_bytes_be(); let full_signed_bytes_init = if
                                non_full_signed_bytes[0] & 0x80 == 0x80 { 0xff } else { 0x00
                                }; let mut full_signed_bytes = [full_signed_bytes_init as
                                u8; 32]; non_full_signed_bytes.into_iter().rev().enumerate()
                                .for_each(| (i, byte) | full_signed_bytes[31 - i] = byte);
                                ethabi::Token::Int(ethabi::Int::from_big_endian(full_signed_bytes
                                .as_ref())) }
                            ],
                        ),
                    ],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
        }
        impl substreams_ethereum::Function for BeforeUpdatePosition {
            const NAME: &'static str = "beforeUpdatePosition";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct Counts {
            pub token: Vec<u8>,
        }
        impl Counts {
            const METHOD_ID: [u8; 4] = [5u8, 104u8, 230u8, 94u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Address],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    token: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[ethabi::Token::Address(ethabi::Address::from_slice(&self.token))],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn output_call(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<
                (
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                ),
                String,
            > {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(
                data: &[u8],
            ) -> Result<
                (
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                ),
                String,
            > {
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Uint(64usize),
                            ethabi::ParamType::Uint(64usize),
                            ethabi::ParamType::Uint(64usize),
                        ],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                values.reverse();
                Ok((
                    {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                    {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                    {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                ))
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(
                &self,
                address: Vec<u8>,
            ) -> Option<
                (
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                ),
            > {
                use substreams_ethereum::pb::eth::rpc;
                let rpc_calls = rpc::RpcCalls {
                    calls: vec![
                        rpc::RpcCall { to_addr : address, data : self.encode(), }
                    ],
                };
                let responses = substreams_ethereum::rpc::eth_call(&rpc_calls).responses;
                let response = responses
                    .get(0)
                    .expect("one response should have existed");
                if response.failed {
                    return None;
                }
                match Self::output(response.raw.as_ref()) {
                    Ok(data) => Some(data),
                    Err(err) => {
                        use substreams_ethereum::Function;
                        substreams::log::info!(
                            "Call output for function `{}` failed to decode with error: {}",
                            Self::NAME, err
                        );
                        None
                    }
                }
            }
        }
        impl substreams_ethereum::Function for Counts {
            const NAME: &'static str = "counts";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        impl substreams_ethereum::rpc::RPCDecodable<
            (
                substreams::scalar::BigInt,
                substreams::scalar::BigInt,
                substreams::scalar::BigInt,
            ),
        > for Counts {
            fn output(
                data: &[u8],
            ) -> Result<
                (
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                ),
                String,
            > {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct ExpandCapacity {
            pub token: Vec<u8>,
            pub min_capacity: substreams::scalar::BigInt,
        }
        impl ExpandCapacity {
            const METHOD_ID: [u8; 4] = [3u8, 90u8, 143u8, 4u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Address, ethabi::ParamType::Uint(64usize)],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    token: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    min_capacity: {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[
                        ethabi::Token::Address(ethabi::Address::from_slice(&self.token)),
                        ethabi::Token::Uint(
                            ethabi::Uint::from_big_endian(
                                match self.min_capacity.clone().to_bytes_be() {
                                    (num_bigint::Sign::Plus, bytes) => bytes,
                                    (num_bigint::Sign::NoSign, bytes) => bytes,
                                    (num_bigint::Sign::Minus, _) => {
                                        panic!("negative numbers are not supported")
                                    }
                                }
                                    .as_slice(),
                            ),
                        ),
                    ],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn output_call(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<substreams::scalar::BigInt, String> {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Uint(64usize)],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                Ok({
                    let mut v = [0 as u8; 32];
                    values
                        .pop()
                        .expect("one output data should have existed")
                        .into_uint()
                        .expect(INTERNAL_ERR)
                        .to_big_endian(v.as_mut_slice());
                    substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                })
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(&self, address: Vec<u8>) -> Option<substreams::scalar::BigInt> {
                use substreams_ethereum::pb::eth::rpc;
                let rpc_calls = rpc::RpcCalls {
                    calls: vec![
                        rpc::RpcCall { to_addr : address, data : self.encode(), }
                    ],
                };
                let responses = substreams_ethereum::rpc::eth_call(&rpc_calls).responses;
                let response = responses
                    .get(0)
                    .expect("one response should have existed");
                if response.failed {
                    return None;
                }
                match Self::output(response.raw.as_ref()) {
                    Ok(data) => Some(data),
                    Err(err) => {
                        use substreams_ethereum::Function;
                        substreams::log::info!(
                            "Call output for function `{}` failed to decode with error: {}",
                            Self::NAME, err
                        );
                        None
                    }
                }
            }
        }
        impl substreams_ethereum::Function for ExpandCapacity {
            const NAME: &'static str = "expandCapacity";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt>
        for ExpandCapacity {
            fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct ExtrapolateSnapshot {
            pub token: Vec<u8>,
            pub at_time: substreams::scalar::BigInt,
        }
        impl ExtrapolateSnapshot {
            const METHOD_ID: [u8; 4] = [201u8, 221u8, 26u8, 131u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Address, ethabi::ParamType::Uint(64usize)],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    token: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    at_time: {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[
                        ethabi::Token::Address(ethabi::Address::from_slice(&self.token)),
                        ethabi::Token::Uint(
                            ethabi::Uint::from_big_endian(
                                match self.at_time.clone().to_bytes_be() {
                                    (num_bigint::Sign::Plus, bytes) => bytes,
                                    (num_bigint::Sign::NoSign, bytes) => bytes,
                                    (num_bigint::Sign::Minus, _) => {
                                        panic!("negative numbers are not supported")
                                    }
                                }
                                    .as_slice(),
                            ),
                        ),
                    ],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn output_call(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<
                (substreams::scalar::BigInt, substreams::scalar::BigInt),
                String,
            > {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(
                data: &[u8],
            ) -> Result<
                (substreams::scalar::BigInt, substreams::scalar::BigInt),
                String,
            > {
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Uint(160usize),
                            ethabi::ParamType::Int(64usize),
                        ],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                values.reverse();
                Ok((
                    {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                    {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_int()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_signed_bytes_be(&v)
                    },
                ))
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(
                &self,
                address: Vec<u8>,
            ) -> Option<(substreams::scalar::BigInt, substreams::scalar::BigInt)> {
                use substreams_ethereum::pb::eth::rpc;
                let rpc_calls = rpc::RpcCalls {
                    calls: vec![
                        rpc::RpcCall { to_addr : address, data : self.encode(), }
                    ],
                };
                let responses = substreams_ethereum::rpc::eth_call(&rpc_calls).responses;
                let response = responses
                    .get(0)
                    .expect("one response should have existed");
                if response.failed {
                    return None;
                }
                match Self::output(response.raw.as_ref()) {
                    Ok(data) => Some(data),
                    Err(err) => {
                        use substreams_ethereum::Function;
                        substreams::log::info!(
                            "Call output for function `{}` failed to decode with error: {}",
                            Self::NAME, err
                        );
                        None
                    }
                }
            }
        }
        impl substreams_ethereum::Function for ExtrapolateSnapshot {
            const NAME: &'static str = "extrapolateSnapshot";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        impl substreams_ethereum::rpc::RPCDecodable<
            (substreams::scalar::BigInt, substreams::scalar::BigInt),
        > for ExtrapolateSnapshot {
            fn output(
                data: &[u8],
            ) -> Result<
                (substreams::scalar::BigInt, substreams::scalar::BigInt),
                String,
            > {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct FindPreviousSnapshot {
            pub token: Vec<u8>,
            pub time: substreams::scalar::BigInt,
        }
        impl FindPreviousSnapshot {
            const METHOD_ID: [u8; 4] = [147u8, 148u8, 118u8, 78u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Address, ethabi::ParamType::Uint(64usize)],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    token: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    time: {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[
                        ethabi::Token::Address(ethabi::Address::from_slice(&self.token)),
                        ethabi::Token::Uint(
                            ethabi::Uint::from_big_endian(
                                match self.time.clone().to_bytes_be() {
                                    (num_bigint::Sign::Plus, bytes) => bytes,
                                    (num_bigint::Sign::NoSign, bytes) => bytes,
                                    (num_bigint::Sign::Minus, _) => {
                                        panic!("negative numbers are not supported")
                                    }
                                }
                                    .as_slice(),
                            ),
                        ),
                    ],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn output_call(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<
                (
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                    (
                        substreams::scalar::BigInt,
                        substreams::scalar::BigInt,
                        substreams::scalar::BigInt,
                    ),
                ),
                String,
            > {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(
                data: &[u8],
            ) -> Result<
                (
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                    (
                        substreams::scalar::BigInt,
                        substreams::scalar::BigInt,
                        substreams::scalar::BigInt,
                    ),
                ),
                String,
            > {
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Uint(256usize),
                            ethabi::ParamType::Uint(256usize),
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::Uint(32usize),
                                    ethabi::ParamType::Uint(160usize),
                                    ethabi::ParamType::Int(64usize)
                                ],
                            ),
                        ],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                values.reverse();
                Ok((
                    {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                    {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                    {
                        let tuple_elements = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_tuple()
                            .expect(INTERNAL_ERR);
                        (
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[0usize]
                                    .clone()
                                    .into_uint()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                            },
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[1usize]
                                    .clone()
                                    .into_uint()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                            },
                            {
                                let mut v = [0 as u8; 32];
                                tuple_elements[2usize]
                                    .clone()
                                    .into_int()
                                    .expect(INTERNAL_ERR)
                                    .to_big_endian(v.as_mut_slice());
                                substreams::scalar::BigInt::from_signed_bytes_be(&v)
                            },
                        )
                    },
                ))
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(
                &self,
                address: Vec<u8>,
            ) -> Option<
                (
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                    (
                        substreams::scalar::BigInt,
                        substreams::scalar::BigInt,
                        substreams::scalar::BigInt,
                    ),
                ),
            > {
                use substreams_ethereum::pb::eth::rpc;
                let rpc_calls = rpc::RpcCalls {
                    calls: vec![
                        rpc::RpcCall { to_addr : address, data : self.encode(), }
                    ],
                };
                let responses = substreams_ethereum::rpc::eth_call(&rpc_calls).responses;
                let response = responses
                    .get(0)
                    .expect("one response should have existed");
                if response.failed {
                    return None;
                }
                match Self::output(response.raw.as_ref()) {
                    Ok(data) => Some(data),
                    Err(err) => {
                        use substreams_ethereum::Function;
                        substreams::log::info!(
                            "Call output for function `{}` failed to decode with error: {}",
                            Self::NAME, err
                        );
                        None
                    }
                }
            }
        }
        impl substreams_ethereum::Function for FindPreviousSnapshot {
            const NAME: &'static str = "findPreviousSnapshot";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        impl substreams_ethereum::rpc::RPCDecodable<
            (
                substreams::scalar::BigInt,
                substreams::scalar::BigInt,
                (
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                ),
            ),
        > for FindPreviousSnapshot {
            fn output(
                data: &[u8],
            ) -> Result<
                (
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                    (
                        substreams::scalar::BigInt,
                        substreams::scalar::BigInt,
                        substreams::scalar::BigInt,
                    ),
                ),
                String,
            > {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct GetExtrapolatedSnapshotsForSortedTimestamps {
            pub token: Vec<u8>,
            pub timestamps: Vec<substreams::scalar::BigInt>,
        }
        impl GetExtrapolatedSnapshotsForSortedTimestamps {
            const METHOD_ID: [u8; 4] = [93u8, 44u8, 205u8, 73u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Address,
                            ethabi::ParamType::Array(
                                Box::new(ethabi::ParamType::Uint(64usize)),
                            ),
                        ],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    token: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    timestamps: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_array()
                        .expect(INTERNAL_ERR)
                        .into_iter()
                        .map(|inner| {
                            let mut v = [0 as u8; 32];
                            inner
                                .into_uint()
                                .expect(INTERNAL_ERR)
                                .to_big_endian(v.as_mut_slice());
                            substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                        })
                        .collect(),
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[
                        ethabi::Token::Address(ethabi::Address::from_slice(&self.token)),
                        {
                            let v = self
                                .timestamps
                                .iter()
                                .map(|inner| ethabi::Token::Uint(
                                    ethabi::Uint::from_big_endian(
                                        match inner.clone().to_bytes_be() {
                                            (num_bigint::Sign::Plus, bytes) => bytes,
                                            (num_bigint::Sign::NoSign, bytes) => bytes,
                                            (num_bigint::Sign::Minus, _) => {
                                                panic!("negative numbers are not supported")
                                            }
                                        }
                                            .as_slice(),
                                    ),
                                ))
                                .collect();
                            ethabi::Token::Array(v)
                        },
                    ],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn output_call(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<
                Vec<(substreams::scalar::BigInt, substreams::scalar::BigInt)>,
                String,
            > {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(
                data: &[u8],
            ) -> Result<
                Vec<(substreams::scalar::BigInt, substreams::scalar::BigInt)>,
                String,
            > {
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Array(
                                Box::new(
                                    ethabi::ParamType::Tuple(
                                        vec![
                                            ethabi::ParamType::Uint(160usize),
                                            ethabi::ParamType::Int(64usize)
                                        ],
                                    ),
                                ),
                            ),
                        ],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                Ok(
                    values
                        .pop()
                        .expect("one output data should have existed")
                        .into_array()
                        .expect(INTERNAL_ERR)
                        .into_iter()
                        .map(|inner| {
                            let tuple_elements = inner.into_tuple().expect(INTERNAL_ERR);
                            (
                                {
                                    let mut v = [0 as u8; 32];
                                    tuple_elements[0usize]
                                        .clone()
                                        .into_uint()
                                        .expect(INTERNAL_ERR)
                                        .to_big_endian(v.as_mut_slice());
                                    substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                                },
                                {
                                    let mut v = [0 as u8; 32];
                                    tuple_elements[1usize]
                                        .clone()
                                        .into_int()
                                        .expect(INTERNAL_ERR)
                                        .to_big_endian(v.as_mut_slice());
                                    substreams::scalar::BigInt::from_signed_bytes_be(&v)
                                },
                            )
                        })
                        .collect(),
                )
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(
                &self,
                address: Vec<u8>,
            ) -> Option<Vec<(substreams::scalar::BigInt, substreams::scalar::BigInt)>> {
                use substreams_ethereum::pb::eth::rpc;
                let rpc_calls = rpc::RpcCalls {
                    calls: vec![
                        rpc::RpcCall { to_addr : address, data : self.encode(), }
                    ],
                };
                let responses = substreams_ethereum::rpc::eth_call(&rpc_calls).responses;
                let response = responses
                    .get(0)
                    .expect("one response should have existed");
                if response.failed {
                    return None;
                }
                match Self::output(response.raw.as_ref()) {
                    Ok(data) => Some(data),
                    Err(err) => {
                        use substreams_ethereum::Function;
                        substreams::log::info!(
                            "Call output for function `{}` failed to decode with error: {}",
                            Self::NAME, err
                        );
                        None
                    }
                }
            }
        }
        impl substreams_ethereum::Function
        for GetExtrapolatedSnapshotsForSortedTimestamps {
            const NAME: &'static str = "getExtrapolatedSnapshotsForSortedTimestamps";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        impl substreams_ethereum::rpc::RPCDecodable<
            Vec<(substreams::scalar::BigInt, substreams::scalar::BigInt)>,
        > for GetExtrapolatedSnapshotsForSortedTimestamps {
            fn output(
                data: &[u8],
            ) -> Result<
                Vec<(substreams::scalar::BigInt, substreams::scalar::BigInt)>,
                String,
            > {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct GetPoolKey {
            pub token: Vec<u8>,
        }
        impl GetPoolKey {
            const METHOD_ID: [u8; 4] = [110u8, 127u8, 253u8, 75u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Address],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    token: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[ethabi::Token::Address(ethabi::Address::from_slice(&self.token))],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn output_call(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<(Vec<u8>, Vec<u8>, [u8; 32usize]), String> {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(
                data: &[u8],
            ) -> Result<(Vec<u8>, Vec<u8>, [u8; 32usize]), String> {
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Tuple(
                                vec![
                                    ethabi::ParamType::Address, ethabi::ParamType::Address,
                                    ethabi::ParamType::FixedBytes(32usize)
                                ],
                            ),
                        ],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                Ok({
                    let tuple_elements = values
                        .pop()
                        .expect("one output data should have existed")
                        .into_tuple()
                        .expect(INTERNAL_ERR);
                    (
                        tuple_elements[0usize]
                            .clone()
                            .into_address()
                            .expect(INTERNAL_ERR)
                            .as_bytes()
                            .to_vec(),
                        tuple_elements[1usize]
                            .clone()
                            .into_address()
                            .expect(INTERNAL_ERR)
                            .as_bytes()
                            .to_vec(),
                        {
                            let mut result = [0u8; 32];
                            let v = tuple_elements[2usize]
                                .clone()
                                .into_fixed_bytes()
                                .expect(INTERNAL_ERR);
                            result.copy_from_slice(&v);
                            result
                        },
                    )
                })
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(
                &self,
                address: Vec<u8>,
            ) -> Option<(Vec<u8>, Vec<u8>, [u8; 32usize])> {
                use substreams_ethereum::pb::eth::rpc;
                let rpc_calls = rpc::RpcCalls {
                    calls: vec![
                        rpc::RpcCall { to_addr : address, data : self.encode(), }
                    ],
                };
                let responses = substreams_ethereum::rpc::eth_call(&rpc_calls).responses;
                let response = responses
                    .get(0)
                    .expect("one response should have existed");
                if response.failed {
                    return None;
                }
                match Self::output(response.raw.as_ref()) {
                    Ok(data) => Some(data),
                    Err(err) => {
                        use substreams_ethereum::Function;
                        substreams::log::info!(
                            "Call output for function `{}` failed to decode with error: {}",
                            Self::NAME, err
                        );
                        None
                    }
                }
            }
        }
        impl substreams_ethereum::Function for GetPoolKey {
            const NAME: &'static str = "getPoolKey";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        impl substreams_ethereum::rpc::RPCDecodable<(Vec<u8>, Vec<u8>, [u8; 32usize])>
        for GetPoolKey {
            fn output(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>, [u8; 32usize]), String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct SecondsSinceOffset {}
        impl SecondsSinceOffset {
            const METHOD_ID: [u8; 4] = [124u8, 142u8, 40u8, 16u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Ok(Self {})
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(&[]);
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn output_call(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<substreams::scalar::BigInt, String> {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Uint(32usize)],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                Ok({
                    let mut v = [0 as u8; 32];
                    values
                        .pop()
                        .expect("one output data should have existed")
                        .into_uint()
                        .expect(INTERNAL_ERR)
                        .to_big_endian(v.as_mut_slice());
                    substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                })
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(&self, address: Vec<u8>) -> Option<substreams::scalar::BigInt> {
                use substreams_ethereum::pb::eth::rpc;
                let rpc_calls = rpc::RpcCalls {
                    calls: vec![
                        rpc::RpcCall { to_addr : address, data : self.encode(), }
                    ],
                };
                let responses = substreams_ethereum::rpc::eth_call(&rpc_calls).responses;
                let response = responses
                    .get(0)
                    .expect("one response should have existed");
                if response.failed {
                    return None;
                }
                match Self::output(response.raw.as_ref()) {
                    Ok(data) => Some(data),
                    Err(err) => {
                        use substreams_ethereum::Function;
                        substreams::log::info!(
                            "Call output for function `{}` failed to decode with error: {}",
                            Self::NAME, err
                        );
                        None
                    }
                }
            }
        }
        impl substreams_ethereum::Function for SecondsSinceOffset {
            const NAME: &'static str = "secondsSinceOffset";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt>
        for SecondsSinceOffset {
            fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct SecondsSinceOffsetToTimestamp {
            pub sso: substreams::scalar::BigInt,
        }
        impl SecondsSinceOffsetToTimestamp {
            const METHOD_ID: [u8; 4] = [140u8, 182u8, 183u8, 72u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Uint(32usize)],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    sso: {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[
                        ethabi::Token::Uint(
                            ethabi::Uint::from_big_endian(
                                match self.sso.clone().to_bytes_be() {
                                    (num_bigint::Sign::Plus, bytes) => bytes,
                                    (num_bigint::Sign::NoSign, bytes) => bytes,
                                    (num_bigint::Sign::Minus, _) => {
                                        panic!("negative numbers are not supported")
                                    }
                                }
                                    .as_slice(),
                            ),
                        ),
                    ],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn output_call(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<substreams::scalar::BigInt, String> {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Uint(64usize)],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                Ok({
                    let mut v = [0 as u8; 32];
                    values
                        .pop()
                        .expect("one output data should have existed")
                        .into_uint()
                        .expect(INTERNAL_ERR)
                        .to_big_endian(v.as_mut_slice());
                    substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                })
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(&self, address: Vec<u8>) -> Option<substreams::scalar::BigInt> {
                use substreams_ethereum::pb::eth::rpc;
                let rpc_calls = rpc::RpcCalls {
                    calls: vec![
                        rpc::RpcCall { to_addr : address, data : self.encode(), }
                    ],
                };
                let responses = substreams_ethereum::rpc::eth_call(&rpc_calls).responses;
                let response = responses
                    .get(0)
                    .expect("one response should have existed");
                if response.failed {
                    return None;
                }
                match Self::output(response.raw.as_ref()) {
                    Ok(data) => Some(data),
                    Err(err) => {
                        use substreams_ethereum::Function;
                        substreams::log::info!(
                            "Call output for function `{}` failed to decode with error: {}",
                            Self::NAME, err
                        );
                        None
                    }
                }
            }
        }
        impl substreams_ethereum::Function for SecondsSinceOffsetToTimestamp {
            const NAME: &'static str = "secondsSinceOffsetToTimestamp";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt>
        for SecondsSinceOffsetToTimestamp {
            fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct Sload {
            pub slot: [u8; 32usize],
        }
        impl Sload {
            const METHOD_ID: [u8; 4] = [244u8, 145u8, 10u8, 115u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::FixedBytes(32usize)],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    slot: {
                        let mut result = [0u8; 32];
                        let v = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_fixed_bytes()
                            .expect(INTERNAL_ERR);
                        result.copy_from_slice(&v);
                        result
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[ethabi::Token::FixedBytes(self.slot.as_ref().to_vec())],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn output_call(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<[u8; 32usize], String> {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(data: &[u8]) -> Result<[u8; 32usize], String> {
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::FixedBytes(32usize)],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                Ok({
                    let mut result = [0u8; 32];
                    let v = values
                        .pop()
                        .expect("one output data should have existed")
                        .into_fixed_bytes()
                        .expect(INTERNAL_ERR);
                    result.copy_from_slice(&v);
                    result
                })
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(&self, address: Vec<u8>) -> Option<[u8; 32usize]> {
                use substreams_ethereum::pb::eth::rpc;
                let rpc_calls = rpc::RpcCalls {
                    calls: vec![
                        rpc::RpcCall { to_addr : address, data : self.encode(), }
                    ],
                };
                let responses = substreams_ethereum::rpc::eth_call(&rpc_calls).responses;
                let response = responses
                    .get(0)
                    .expect("one response should have existed");
                if response.failed {
                    return None;
                }
                match Self::output(response.raw.as_ref()) {
                    Ok(data) => Some(data),
                    Err(err) => {
                        use substreams_ethereum::Function;
                        substreams::log::info!(
                            "Call output for function `{}` failed to decode with error: {}",
                            Self::NAME, err
                        );
                        None
                    }
                }
            }
        }
        impl substreams_ethereum::Function for Sload {
            const NAME: &'static str = "sload";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        impl substreams_ethereum::rpc::RPCDecodable<[u8; 32usize]> for Sload {
            fn output(data: &[u8]) -> Result<[u8; 32usize], String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct Snapshots {
            pub token: Vec<u8>,
            pub index: substreams::scalar::BigInt,
        }
        impl Snapshots {
            const METHOD_ID: [u8; 4] = [219u8, 85u8, 23u8, 176u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Address, ethabi::ParamType::Uint(256usize)],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    token: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    index: {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[
                        ethabi::Token::Address(ethabi::Address::from_slice(&self.token)),
                        ethabi::Token::Uint(
                            ethabi::Uint::from_big_endian(
                                match self.index.clone().to_bytes_be() {
                                    (num_bigint::Sign::Plus, bytes) => bytes,
                                    (num_bigint::Sign::NoSign, bytes) => bytes,
                                    (num_bigint::Sign::Minus, _) => {
                                        panic!("negative numbers are not supported")
                                    }
                                }
                                    .as_slice(),
                            ),
                        ),
                    ],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn output_call(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<
                (
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                ),
                String,
            > {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(
                data: &[u8],
            ) -> Result<
                (
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                ),
                String,
            > {
                let mut values = ethabi::decode(
                        &[
                            ethabi::ParamType::Uint(32usize),
                            ethabi::ParamType::Uint(160usize),
                            ethabi::ParamType::Int(64usize),
                        ],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                values.reverse();
                Ok((
                    {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                    {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_uint()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                    },
                    {
                        let mut v = [0 as u8; 32];
                        values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_int()
                            .expect(INTERNAL_ERR)
                            .to_big_endian(v.as_mut_slice());
                        substreams::scalar::BigInt::from_signed_bytes_be(&v)
                    },
                ))
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(
                &self,
                address: Vec<u8>,
            ) -> Option<
                (
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                ),
            > {
                use substreams_ethereum::pb::eth::rpc;
                let rpc_calls = rpc::RpcCalls {
                    calls: vec![
                        rpc::RpcCall { to_addr : address, data : self.encode(), }
                    ],
                };
                let responses = substreams_ethereum::rpc::eth_call(&rpc_calls).responses;
                let response = responses
                    .get(0)
                    .expect("one response should have existed");
                if response.failed {
                    return None;
                }
                match Self::output(response.raw.as_ref()) {
                    Ok(data) => Some(data),
                    Err(err) => {
                        use substreams_ethereum::Function;
                        substreams::log::info!(
                            "Call output for function `{}` failed to decode with error: {}",
                            Self::NAME, err
                        );
                        None
                    }
                }
            }
        }
        impl substreams_ethereum::Function for Snapshots {
            const NAME: &'static str = "snapshots";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        impl substreams_ethereum::rpc::RPCDecodable<
            (
                substreams::scalar::BigInt,
                substreams::scalar::BigInt,
                substreams::scalar::BigInt,
            ),
        > for Snapshots {
            fn output(
                data: &[u8],
            ) -> Result<
                (
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                    substreams::scalar::BigInt,
                ),
                String,
            > {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct TimestampOffset {}
        impl TimestampOffset {
            const METHOD_ID: [u8; 4] = [210u8, 246u8, 128u8, 53u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Ok(Self {})
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(&[]);
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn output_call(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<substreams::scalar::BigInt, String> {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Uint(64usize)],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                Ok({
                    let mut v = [0 as u8; 32];
                    values
                        .pop()
                        .expect("one output data should have existed")
                        .into_uint()
                        .expect(INTERNAL_ERR)
                        .to_big_endian(v.as_mut_slice());
                    substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                })
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(&self, address: Vec<u8>) -> Option<substreams::scalar::BigInt> {
                use substreams_ethereum::pb::eth::rpc;
                let rpc_calls = rpc::RpcCalls {
                    calls: vec![
                        rpc::RpcCall { to_addr : address, data : self.encode(), }
                    ],
                };
                let responses = substreams_ethereum::rpc::eth_call(&rpc_calls).responses;
                let response = responses
                    .get(0)
                    .expect("one response should have existed");
                if response.failed {
                    return None;
                }
                match Self::output(response.raw.as_ref()) {
                    Ok(data) => Some(data),
                    Err(err) => {
                        use substreams_ethereum::Function;
                        substreams::log::info!(
                            "Call output for function `{}` failed to decode with error: {}",
                            Self::NAME, err
                        );
                        None
                    }
                }
            }
        }
        impl substreams_ethereum::Function for TimestampOffset {
            const NAME: &'static str = "timestampOffset";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt>
        for TimestampOffset {
            fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct Tload {
            pub slot: [u8; 32usize],
        }
        impl Tload {
            const METHOD_ID: [u8; 4] = [189u8, 46u8, 88u8, 125u8];
            pub fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                let maybe_data = call.input.get(4..);
                if maybe_data.is_none() {
                    return Err("no data to decode".to_string());
                }
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::FixedBytes(32usize)],
                        maybe_data.unwrap(),
                    )
                    .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    slot: {
                        let mut result = [0u8; 32];
                        let v = values
                            .pop()
                            .expect(INTERNAL_ERR)
                            .into_fixed_bytes()
                            .expect(INTERNAL_ERR);
                        result.copy_from_slice(&v);
                        result
                    },
                })
            }
            pub fn encode(&self) -> Vec<u8> {
                let data = ethabi::encode(
                    &[ethabi::Token::FixedBytes(self.slot.as_ref().to_vec())],
                );
                let mut encoded = Vec::with_capacity(4 + data.len());
                encoded.extend(Self::METHOD_ID);
                encoded.extend(data);
                encoded
            }
            pub fn output_call(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<[u8; 32usize], String> {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(data: &[u8]) -> Result<[u8; 32usize], String> {
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::FixedBytes(32usize)],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                Ok({
                    let mut result = [0u8; 32];
                    let v = values
                        .pop()
                        .expect("one output data should have existed")
                        .into_fixed_bytes()
                        .expect(INTERNAL_ERR);
                    result.copy_from_slice(&v);
                    result
                })
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(&self, address: Vec<u8>) -> Option<[u8; 32usize]> {
                use substreams_ethereum::pb::eth::rpc;
                let rpc_calls = rpc::RpcCalls {
                    calls: vec![
                        rpc::RpcCall { to_addr : address, data : self.encode(), }
                    ],
                };
                let responses = substreams_ethereum::rpc::eth_call(&rpc_calls).responses;
                let response = responses
                    .get(0)
                    .expect("one response should have existed");
                if response.failed {
                    return None;
                }
                match Self::output(response.raw.as_ref()) {
                    Ok(data) => Some(data),
                    Err(err) => {
                        use substreams_ethereum::Function;
                        substreams::log::info!(
                            "Call output for function `{}` failed to decode with error: {}",
                            Self::NAME, err
                        );
                        None
                    }
                }
            }
        }
        impl substreams_ethereum::Function for Tload {
            const NAME: &'static str = "tload";
            fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                Self::match_call(call)
            }
            fn decode(
                call: &substreams_ethereum::pb::eth::v2::Call,
            ) -> Result<Self, String> {
                Self::decode(call)
            }
            fn encode(&self) -> Vec<u8> {
                self.encode()
            }
        }
        impl substreams_ethereum::rpc::RPCDecodable<[u8; 32usize]> for Tload {
            fn output(data: &[u8]) -> Result<[u8; 32usize], String> {
                Self::output(data)
            }
        }
    }
    /// Contract's events.
    #[allow(dead_code, unused_imports, unused_variables)]
    pub mod events {
        use super::INTERNAL_ERR;
    }