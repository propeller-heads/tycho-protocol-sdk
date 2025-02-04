    const INTERNAL_ERR: &'static str = "`ethabi_derive` internal error";
    /// Contract's functions.
    #[allow(dead_code, unused_imports, unused_variables)]
    pub mod functions {
        use super::INTERNAL_ERR;
        #[derive(Debug, Clone, PartialEq)]
        pub struct Halted {}
        impl Halted {
            const METHOD_ID: [u8; 4] = [103u8, 141u8, 119u8, 50u8];
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
                        &[ethabi::ParamType::Uint(256usize)],
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
        impl substreams_ethereum::Function for Halted {
            const NAME: &'static str = "HALTED";
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
        for Halted {
            fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct Buf {}
        impl Buf {
            const METHOD_ID: [u8; 4] = [21u8, 35u8, 37u8, 21u8];
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
                        &[ethabi::ParamType::Uint(256usize)],
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
        impl substreams_ethereum::Function for Buf {
            const NAME: &'static str = "buf";
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
        impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt> for Buf {
            fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct BuyGem {
            pub usr: Vec<u8>,
            pub gem_amt: substreams::scalar::BigInt,
        }
        impl BuyGem {
            const METHOD_ID: [u8; 4] = [141u8, 126u8, 249u8, 187u8];
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
                    gem_amt: {
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
                                match self.gem_amt.clone().to_bytes_be() {
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
                        &[ethabi::ParamType::Uint(256usize)],
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
        impl substreams_ethereum::Function for BuyGem {
            const NAME: &'static str = "buyGem";
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
        for BuyGem {
            fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                Self::output(data)
            }
        }
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
        pub struct Dec {}
        impl Dec {
            const METHOD_ID: [u8; 4] = [179u8, 188u8, 250u8, 130u8];
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
                        &[ethabi::ParamType::Uint(256usize)],
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
        impl substreams_ethereum::Function for Dec {
            const NAME: &'static str = "dec";
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
        impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt> for Dec {
            fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct Gem {}
        impl Gem {
            const METHOD_ID: [u8; 4] = [123u8, 210u8, 190u8, 167u8];
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
        impl substreams_ethereum::Function for Gem {
            const NAME: &'static str = "gem";
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
        impl substreams_ethereum::rpc::RPCDecodable<Vec<u8>> for Gem {
            fn output(data: &[u8]) -> Result<Vec<u8>, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct GemJoin {}
        impl GemJoin {
            const METHOD_ID: [u8; 4] = [1u8, 102u8, 79u8, 102u8];
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
        impl substreams_ethereum::Function for GemJoin {
            const NAME: &'static str = "gemJoin";
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
        impl substreams_ethereum::rpc::RPCDecodable<Vec<u8>> for GemJoin {
            fn output(data: &[u8]) -> Result<Vec<u8>, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct Ilk {}
        impl Ilk {
            const METHOD_ID: [u8; 4] = [197u8, 206u8, 40u8, 30u8];
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
        impl substreams_ethereum::Function for Ilk {
            const NAME: &'static str = "ilk";
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
        impl substreams_ethereum::rpc::RPCDecodable<[u8; 32usize]> for Ilk {
            fn output(data: &[u8]) -> Result<[u8; 32usize], String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct Live {}
        impl Live {
            const METHOD_ID: [u8; 4] = [149u8, 122u8, 165u8, 140u8];
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
                        &[ethabi::ParamType::Uint(256usize)],
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
        impl substreams_ethereum::Function for Live {
            const NAME: &'static str = "live";
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
        for Live {
            fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct Pocket {}
        impl Pocket {
            const METHOD_ID: [u8; 4] = [204u8, 206u8, 249u8, 226u8];
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
        impl substreams_ethereum::Function for Pocket {
            const NAME: &'static str = "pocket";
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
        impl substreams_ethereum::rpc::RPCDecodable<Vec<u8>> for Pocket {
            fn output(data: &[u8]) -> Result<Vec<u8>, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct Psm {}
        impl Psm {
            const METHOD_ID: [u8; 4] = [4u8, 189u8, 162u8, 98u8];
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
        impl substreams_ethereum::Function for Psm {
            const NAME: &'static str = "psm";
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
        impl substreams_ethereum::rpc::RPCDecodable<Vec<u8>> for Psm {
            fn output(data: &[u8]) -> Result<Vec<u8>, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct SellGem {
            pub usr: Vec<u8>,
            pub gem_amt: substreams::scalar::BigInt,
        }
        impl SellGem {
            const METHOD_ID: [u8; 4] = [149u8, 153u8, 18u8, 118u8];
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
                    gem_amt: {
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
                                match self.gem_amt.clone().to_bytes_be() {
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
                        &[ethabi::ParamType::Uint(256usize)],
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
        impl substreams_ethereum::Function for SellGem {
            const NAME: &'static str = "sellGem";
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
        for SellGem {
            fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct Tin {}
        impl Tin {
            const METHOD_ID: [u8; 4] = [86u8, 141u8, 75u8, 111u8];
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
                        &[ethabi::ParamType::Uint(256usize)],
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
        impl substreams_ethereum::Function for Tin {
            const NAME: &'static str = "tin";
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
        impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt> for Tin {
            fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct To18ConversionFactor {}
        impl To18ConversionFactor {
            const METHOD_ID: [u8; 4] = [64u8, 16u8, 247u8, 119u8];
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
                        &[ethabi::ParamType::Uint(256usize)],
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
        impl substreams_ethereum::Function for To18ConversionFactor {
            const NAME: &'static str = "to18ConversionFactor";
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
        for To18ConversionFactor {
            fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct Tout {}
        impl Tout {
            const METHOD_ID: [u8; 4] = [250u8, 224u8, 54u8, 213u8];
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
                        &[ethabi::ParamType::Uint(256usize)],
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
        impl substreams_ethereum::Function for Tout {
            const NAME: &'static str = "tout";
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
        for Tout {
            fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
                Self::output(data)
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
        pub struct Vat {}
        impl Vat {
            const METHOD_ID: [u8; 4] = [54u8, 86u8, 158u8, 119u8];
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
        impl substreams_ethereum::Function for Vat {
            const NAME: &'static str = "vat";
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
        impl substreams_ethereum::rpc::RPCDecodable<Vec<u8>> for Vat {
            fn output(data: &[u8]) -> Result<Vec<u8>, String> {
                Self::output(data)
            }
        }
        #[derive(Debug, Clone, PartialEq)]
        pub struct Vow {}
        impl Vow {
            const METHOD_ID: [u8; 4] = [98u8, 108u8, 179u8, 197u8];
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
        impl substreams_ethereum::Function for Vow {
            const NAME: &'static str = "vow";
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
        impl substreams_ethereum::rpc::RPCDecodable<Vec<u8>> for Vow {
            fn output(data: &[u8]) -> Result<Vec<u8>, String> {
                Self::output(data)
            }
        }
    }
    /// Contract's events.
    #[allow(dead_code, unused_imports, unused_variables)]
    pub mod events {
        use super::INTERNAL_ERR;
    }