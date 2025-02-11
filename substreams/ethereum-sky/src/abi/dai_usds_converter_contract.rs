    const INTERNAL_ERR: &'static str = "`ethabi_derive` internal error";
    /// Contract's functions.
    #[allow(dead_code, unused_imports, unused_variables)]
    pub mod functions {
        use super::INTERNAL_ERR;
        #[derive(Debug, Clone, PartialEq)]
        pub struct Dai {}
        impl Dai {
            const METHOD_ID: [u8; 4] = [244u8, 185u8, 250u8, 117u8];
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
            ) -> Result<Vec<u8>, String> {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(data: &[u8]) -> Result<Vec<u8>, String> {
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Address],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                Ok(
                    values
                        .pop()
                        .expect("one output data should have existed")
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                )
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(&self, address: Vec<u8>) -> Option<Vec<u8>> {
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
        impl substreams_ethereum::Function for Dai {
            const NAME: &'static str = "dai";
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
        impl substreams_ethereum::rpc::RPCDecodable<Vec<u8>> for Dai {
            fn output(data: &[u8]) -> Result<Vec<u8>, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct DaiJoin {}
        impl DaiJoin {
            const METHOD_ID: [u8; 4] = [193u8, 22u8, 69u8, 188u8];
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
            ) -> Result<Vec<u8>, String> {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(data: &[u8]) -> Result<Vec<u8>, String> {
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Address],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                Ok(
                    values
                        .pop()
                        .expect("one output data should have existed")
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                )
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(&self, address: Vec<u8>) -> Option<Vec<u8>> {
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
        impl substreams_ethereum::Function for DaiJoin {
            const NAME: &'static str = "daiJoin";
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
        impl substreams_ethereum::rpc::RPCDecodable<Vec<u8>> for DaiJoin {
            fn output(data: &[u8]) -> Result<Vec<u8>, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct DaiToUsds {
            pub usr: Vec<u8>,
            pub wad: substreams::scalar::BigInt,
        }
        impl DaiToUsds {
            const METHOD_ID: [u8; 4] = [242u8, 192u8, 122u8, 174u8];
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
                    usr: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    wad: {
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
                        ethabi::Token::Address(ethabi::Address::from_slice(&self.usr)),
                        ethabi::Token::Uint(
                            ethabi::Uint::from_big_endian(
                                match self.wad.clone().to_bytes_be() {
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
        impl substreams_ethereum::Function for DaiToUsds {
            const NAME: &'static str = "daiToUsds";
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
        pub struct Usds {}
        impl Usds {
            const METHOD_ID: [u8; 4] = [76u8, 242u8, 130u8, 251u8];
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
            ) -> Result<Vec<u8>, String> {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(data: &[u8]) -> Result<Vec<u8>, String> {
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Address],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                Ok(
                    values
                        .pop()
                        .expect("one output data should have existed")
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                )
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(&self, address: Vec<u8>) -> Option<Vec<u8>> {
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
        impl substreams_ethereum::Function for Usds {
            const NAME: &'static str = "usds";
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
        impl substreams_ethereum::rpc::RPCDecodable<Vec<u8>> for Usds {
            fn output(data: &[u8]) -> Result<Vec<u8>, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct UsdsJoin {}
        impl UsdsJoin {
            const METHOD_ID: [u8; 4] = [250u8, 30u8, 46u8, 134u8];
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
            ) -> Result<Vec<u8>, String> {
                Self::output(call.return_data.as_ref())
            }
            pub fn output(data: &[u8]) -> Result<Vec<u8>, String> {
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Address],
                        data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode output data: {:?}", e))?;
                Ok(
                    values
                        .pop()
                        .expect("one output data should have existed")
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                )
            }
            pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
                match call.input.get(0..4) {
                    Some(signature) => Self::METHOD_ID == signature,
                    None => false,
                }
            }
            pub fn call(&self, address: Vec<u8>) -> Option<Vec<u8>> {
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
        impl substreams_ethereum::Function for UsdsJoin {
            const NAME: &'static str = "usdsJoin";
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
        impl substreams_ethereum::rpc::RPCDecodable<Vec<u8>> for UsdsJoin {
            fn output(data: &[u8]) -> Result<Vec<u8>, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct UsdsToDai {
            pub usr: Vec<u8>,
            pub wad: substreams::scalar::BigInt,
        }
        impl UsdsToDai {
            const METHOD_ID: [u8; 4] = [104u8, 243u8, 1u8, 80u8];
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
                    usr: values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    wad: {
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
                        ethabi::Token::Address(ethabi::Address::from_slice(&self.usr)),
                        ethabi::Token::Uint(
                            ethabi::Uint::from_big_endian(
                                match self.wad.clone().to_bytes_be() {
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
        impl substreams_ethereum::Function for UsdsToDai {
            const NAME: &'static str = "usdsToDai";
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
    }
    /// Contract's events.
    #[allow(dead_code, unused_imports, unused_variables)]
    pub mod events {
        use super::INTERNAL_ERR;
        #[derive(Debug, Clone, PartialEq)]
        pub struct DaiToUsds {
            pub caller: Vec<u8>,
            pub usr: Vec<u8>,
            pub wad: substreams::scalar::BigInt,
        }
        impl DaiToUsds {
            const TOPIC_ID: [u8; 32] = [
                35u8,
                136u8,
                157u8,
                182u8,
                178u8,
                68u8,
                210u8,
                57u8,
                52u8,
                77u8,
                234u8,
                200u8,
                142u8,
                199u8,
                120u8,
                29u8,
                129u8,
                11u8,
                56u8,
                115u8,
                254u8,
                50u8,
                57u8,
                57u8,
                209u8,
                206u8,
                14u8,
                138u8,
                201u8,
                86u8,
                66u8,
                53u8,
            ];
            pub fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
                if log.topics.len() != 3usize {
                    return false;
                }
                if log.data.len() != 32usize {
                    return false;
                }
                return log.topics.get(0).expect("bounds already checked").as_ref()
                    == Self::TOPIC_ID;
            }
            pub fn decode(
                log: &substreams_ethereum::pb::eth::v2::Log,
            ) -> Result<Self, String> {
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Uint(256usize)],
                        log.data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode log.data: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    caller: ethabi::decode(
                            &[ethabi::ParamType::Address],
                            log.topics[1usize].as_ref(),
                        )
                        .map_err(|e| {
                            format!(
                                "unable to decode param 'caller' from topic of type 'address': {:?}",
                                e
                            )
                        })?
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    usr: ethabi::decode(
                            &[ethabi::ParamType::Address],
                            log.topics[2usize].as_ref(),
                        )
                        .map_err(|e| {
                            format!(
                                "unable to decode param 'usr' from topic of type 'address': {:?}",
                                e
                            )
                        })?
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    wad: {
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
        }
        impl substreams_ethereum::Event for DaiToUsds {
            const NAME: &'static str = "DaiToUsds";
            fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
                Self::match_log(log)
            }
            fn decode(
                log: &substreams_ethereum::pb::eth::v2::Log,
            ) -> Result<Self, String> {
                Self::decode(log)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct UsdsToDai {
            pub caller: Vec<u8>,
            pub usr: Vec<u8>,
            pub wad: substreams::scalar::BigInt,
        }
        impl UsdsToDai {
            const TOPIC_ID: [u8; 32] = [
                156u8,
                123u8,
                211u8,
                182u8,
                11u8,
                126u8,
                70u8,
                252u8,
                29u8,
                122u8,
                56u8,
                216u8,
                139u8,
                125u8,
                212u8,
                190u8,
                14u8,
                2u8,
                139u8,
                128u8,
                183u8,
                6u8,
                132u8,
                143u8,
                195u8,
                192u8,
                101u8,
                234u8,
                188u8,
                142u8,
                106u8,
                155u8,
            ];
            pub fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
                if log.topics.len() != 3usize {
                    return false;
                }
                if log.data.len() != 32usize {
                    return false;
                }
                return log.topics.get(0).expect("bounds already checked").as_ref()
                    == Self::TOPIC_ID;
            }
            pub fn decode(
                log: &substreams_ethereum::pb::eth::v2::Log,
            ) -> Result<Self, String> {
                let mut values = ethabi::decode(
                        &[ethabi::ParamType::Uint(256usize)],
                        log.data.as_ref(),
                    )
                    .map_err(|e| format!("unable to decode log.data: {:?}", e))?;
                values.reverse();
                Ok(Self {
                    caller: ethabi::decode(
                            &[ethabi::ParamType::Address],
                            log.topics[1usize].as_ref(),
                        )
                        .map_err(|e| {
                            format!(
                                "unable to decode param 'caller' from topic of type 'address': {:?}",
                                e
                            )
                        })?
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    usr: ethabi::decode(
                            &[ethabi::ParamType::Address],
                            log.topics[2usize].as_ref(),
                        )
                        .map_err(|e| {
                            format!(
                                "unable to decode param 'usr' from topic of type 'address': {:?}",
                                e
                            )
                        })?
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    wad: {
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
        }
        impl substreams_ethereum::Event for UsdsToDai {
            const NAME: &'static str = "UsdsToDai";
            fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
                Self::match_log(log)
            }
            fn decode(
                log: &substreams_ethereum::pb::eth::v2::Log,
            ) -> Result<Self, String> {
                Self::decode(log)
            }
        }
    }