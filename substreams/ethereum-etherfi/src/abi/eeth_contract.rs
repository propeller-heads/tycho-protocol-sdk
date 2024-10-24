const INTERNAL_ERR: &'static str = "`ethabi_derive` internal error";
/// Contract's functions.
#[allow(dead_code, unused_imports, unused_variables)]
pub mod functions {
    use super::INTERNAL_ERR;
    #[derive(Debug, Clone, PartialEq)]
    pub struct DomainSeparator {}
    impl DomainSeparator {
        const METHOD_ID: [u8; 4] = [54u8, 68u8, 229u8, 21u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
            let mut values =
                ethabi::decode(&[ethabi::ParamType::FixedBytes(32usize)], data.as_ref())
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
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for DomainSeparator {
        const NAME: &'static str = "DOMAIN_SEPARATOR";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<[u8; 32usize]> for DomainSeparator {
        fn output(data: &[u8]) -> Result<[u8; 32usize], String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Allowance {
        pub u_owner: Vec<u8>,
        pub u_spender: Vec<u8>,
    }
    impl Allowance {
        const METHOD_ID: [u8; 4] = [221u8, 98u8, 237u8, 62u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            let maybe_data = call.input.get(4..);
            if maybe_data.is_none() {
                return Err("no data to decode".to_string());
            }
            let mut values = ethabi::decode(
                &[ethabi::ParamType::Address, ethabi::ParamType::Address],
                maybe_data.unwrap(),
            )
            .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
            values.reverse();
            Ok(Self {
                u_owner: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                u_spender: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
            })
        }
        pub fn encode(&self) -> Vec<u8> {
            let data = ethabi::encode(&[
                ethabi::Token::Address(ethabi::Address::from_slice(&self.u_owner)),
                ethabi::Token::Address(ethabi::Address::from_slice(&self.u_spender)),
            ]);
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
            let mut values = ethabi::decode(&[ethabi::ParamType::Uint(256usize)], data.as_ref())
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
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for Allowance {
        const NAME: &'static str = "allowance";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt> for Allowance {
        fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Allowances {
        pub param0: Vec<u8>,
        pub param1: Vec<u8>,
    }
    impl Allowances {
        const METHOD_ID: [u8; 4] = [85u8, 182u8, 237u8, 92u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            let maybe_data = call.input.get(4..);
            if maybe_data.is_none() {
                return Err("no data to decode".to_string());
            }
            let mut values = ethabi::decode(
                &[ethabi::ParamType::Address, ethabi::ParamType::Address],
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
                param1: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
            })
        }
        pub fn encode(&self) -> Vec<u8> {
            let data = ethabi::encode(&[
                ethabi::Token::Address(ethabi::Address::from_slice(&self.param0)),
                ethabi::Token::Address(ethabi::Address::from_slice(&self.param1)),
            ]);
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
            let mut values = ethabi::decode(&[ethabi::ParamType::Uint(256usize)], data.as_ref())
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
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for Allowances {
        const NAME: &'static str = "allowances";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt> for Allowances {
        fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Approve {
        pub u_spender: Vec<u8>,
        pub u_amount: substreams::scalar::BigInt,
    }
    impl Approve {
        const METHOD_ID: [u8; 4] = [9u8, 94u8, 167u8, 179u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
                u_spender: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                u_amount: {
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
            let data = ethabi::encode(&[
                ethabi::Token::Address(ethabi::Address::from_slice(&self.u_spender)),
                ethabi::Token::Uint(ethabi::Uint::from_big_endian(
                    match self.u_amount.clone().to_bytes_be() {
                        (num_bigint::Sign::Plus, bytes) => bytes,
                        (num_bigint::Sign::NoSign, bytes) => bytes,
                        (num_bigint::Sign::Minus, _) => {
                            panic!("negative numbers are not supported")
                        }
                    }
                    .as_slice(),
                )),
            ]);
            let mut encoded = Vec::with_capacity(4 + data.len());
            encoded.extend(Self::METHOD_ID);
            encoded.extend(data);
            encoded
        }
        pub fn output_call(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<bool, String> {
            Self::output(call.return_data.as_ref())
        }
        pub fn output(data: &[u8]) -> Result<bool, String> {
            let mut values = ethabi::decode(&[ethabi::ParamType::Bool], data.as_ref())
                .map_err(|e| format!("unable to decode output data: {:?}", e))?;
            Ok(values
                .pop()
                .expect("one output data should have existed")
                .into_bool()
                .expect(INTERNAL_ERR))
        }
        pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            match call.input.get(0..4) {
                Some(signature) => Self::METHOD_ID == signature,
                None => false,
            }
        }
        pub fn call(&self, address: Vec<u8>) -> Option<bool> {
            use substreams_ethereum::pb::eth::rpc;
            let rpc_calls = rpc::RpcCalls {
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for Approve {
        const NAME: &'static str = "approve";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<bool> for Approve {
        fn output(data: &[u8]) -> Result<bool, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct BalanceOf {
        pub u_user: Vec<u8>,
    }
    impl BalanceOf {
        const METHOD_ID: [u8; 4] = [112u8, 160u8, 130u8, 49u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            let maybe_data = call.input.get(4..);
            if maybe_data.is_none() {
                return Err("no data to decode".to_string());
            }
            let mut values = ethabi::decode(&[ethabi::ParamType::Address], maybe_data.unwrap())
                .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
            values.reverse();
            Ok(Self {
                u_user: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
            })
        }
        pub fn encode(&self) -> Vec<u8> {
            let data = ethabi::encode(&[ethabi::Token::Address(ethabi::Address::from_slice(
                &self.u_user,
            ))]);
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
            let mut values = ethabi::decode(&[ethabi::ParamType::Uint(256usize)], data.as_ref())
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
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for BalanceOf {
        const NAME: &'static str = "balanceOf";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt> for BalanceOf {
        fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct BurnShares {
        pub u_user: Vec<u8>,
        pub u_share: substreams::scalar::BigInt,
    }
    impl BurnShares {
        const METHOD_ID: [u8; 4] = [238u8, 122u8, 124u8, 4u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
                u_user: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                u_share: {
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
            let data = ethabi::encode(&[
                ethabi::Token::Address(ethabi::Address::from_slice(&self.u_user)),
                ethabi::Token::Uint(ethabi::Uint::from_big_endian(
                    match self.u_share.clone().to_bytes_be() {
                        (num_bigint::Sign::Plus, bytes) => bytes,
                        (num_bigint::Sign::NoSign, bytes) => bytes,
                        (num_bigint::Sign::Minus, _) => {
                            panic!("negative numbers are not supported")
                        }
                    }
                    .as_slice(),
                )),
            ]);
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
    impl substreams_ethereum::Function for BurnShares {
        const NAME: &'static str = "burnShares";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Decimals {}
    impl Decimals {
        const METHOD_ID: [u8; 4] = [49u8, 60u8, 229u8, 103u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
            let mut values = ethabi::decode(&[ethabi::ParamType::Uint(8usize)], data.as_ref())
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
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for Decimals {
        const NAME: &'static str = "decimals";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt> for Decimals {
        fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct DecreaseAllowance {
        pub u_spender: Vec<u8>,
        pub u_decrease_amount: substreams::scalar::BigInt,
    }
    impl DecreaseAllowance {
        const METHOD_ID: [u8; 4] = [164u8, 87u8, 194u8, 215u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
                u_spender: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                u_decrease_amount: {
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
            let data = ethabi::encode(&[
                ethabi::Token::Address(ethabi::Address::from_slice(&self.u_spender)),
                ethabi::Token::Uint(ethabi::Uint::from_big_endian(
                    match self
                        .u_decrease_amount
                        .clone()
                        .to_bytes_be()
                    {
                        (num_bigint::Sign::Plus, bytes) => bytes,
                        (num_bigint::Sign::NoSign, bytes) => bytes,
                        (num_bigint::Sign::Minus, _) => {
                            panic!("negative numbers are not supported")
                        }
                    }
                    .as_slice(),
                )),
            ]);
            let mut encoded = Vec::with_capacity(4 + data.len());
            encoded.extend(Self::METHOD_ID);
            encoded.extend(data);
            encoded
        }
        pub fn output_call(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<bool, String> {
            Self::output(call.return_data.as_ref())
        }
        pub fn output(data: &[u8]) -> Result<bool, String> {
            let mut values = ethabi::decode(&[ethabi::ParamType::Bool], data.as_ref())
                .map_err(|e| format!("unable to decode output data: {:?}", e))?;
            Ok(values
                .pop()
                .expect("one output data should have existed")
                .into_bool()
                .expect(INTERNAL_ERR))
        }
        pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            match call.input.get(0..4) {
                Some(signature) => Self::METHOD_ID == signature,
                None => false,
            }
        }
        pub fn call(&self, address: Vec<u8>) -> Option<bool> {
            use substreams_ethereum::pb::eth::rpc;
            let rpc_calls = rpc::RpcCalls {
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for DecreaseAllowance {
        const NAME: &'static str = "decreaseAllowance";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<bool> for DecreaseAllowance {
        fn output(data: &[u8]) -> Result<bool, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct GetImplementation {}
    impl GetImplementation {
        const METHOD_ID: [u8; 4] = [170u8, 241u8, 15u8, 66u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
            let mut values = ethabi::decode(&[ethabi::ParamType::Address], data.as_ref())
                .map_err(|e| format!("unable to decode output data: {:?}", e))?;
            Ok(values
                .pop()
                .expect("one output data should have existed")
                .into_address()
                .expect(INTERNAL_ERR)
                .as_bytes()
                .to_vec())
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
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for GetImplementation {
        const NAME: &'static str = "getImplementation";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<Vec<u8>> for GetImplementation {
        fn output(data: &[u8]) -> Result<Vec<u8>, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct IncreaseAllowance {
        pub u_spender: Vec<u8>,
        pub u_increase_amount: substreams::scalar::BigInt,
    }
    impl IncreaseAllowance {
        const METHOD_ID: [u8; 4] = [57u8, 80u8, 147u8, 81u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
                u_spender: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                u_increase_amount: {
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
            let data = ethabi::encode(&[
                ethabi::Token::Address(ethabi::Address::from_slice(&self.u_spender)),
                ethabi::Token::Uint(ethabi::Uint::from_big_endian(
                    match self
                        .u_increase_amount
                        .clone()
                        .to_bytes_be()
                    {
                        (num_bigint::Sign::Plus, bytes) => bytes,
                        (num_bigint::Sign::NoSign, bytes) => bytes,
                        (num_bigint::Sign::Minus, _) => {
                            panic!("negative numbers are not supported")
                        }
                    }
                    .as_slice(),
                )),
            ]);
            let mut encoded = Vec::with_capacity(4 + data.len());
            encoded.extend(Self::METHOD_ID);
            encoded.extend(data);
            encoded
        }
        pub fn output_call(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<bool, String> {
            Self::output(call.return_data.as_ref())
        }
        pub fn output(data: &[u8]) -> Result<bool, String> {
            let mut values = ethabi::decode(&[ethabi::ParamType::Bool], data.as_ref())
                .map_err(|e| format!("unable to decode output data: {:?}", e))?;
            Ok(values
                .pop()
                .expect("one output data should have existed")
                .into_bool()
                .expect(INTERNAL_ERR))
        }
        pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            match call.input.get(0..4) {
                Some(signature) => Self::METHOD_ID == signature,
                None => false,
            }
        }
        pub fn call(&self, address: Vec<u8>) -> Option<bool> {
            use substreams_ethereum::pb::eth::rpc;
            let rpc_calls = rpc::RpcCalls {
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for IncreaseAllowance {
        const NAME: &'static str = "increaseAllowance";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<bool> for IncreaseAllowance {
        fn output(data: &[u8]) -> Result<bool, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Initialize {
        pub u_liquidity_pool: Vec<u8>,
    }
    impl Initialize {
        const METHOD_ID: [u8; 4] = [196u8, 214u8, 109u8, 232u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            let maybe_data = call.input.get(4..);
            if maybe_data.is_none() {
                return Err("no data to decode".to_string());
            }
            let mut values = ethabi::decode(&[ethabi::ParamType::Address], maybe_data.unwrap())
                .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
            values.reverse();
            Ok(Self {
                u_liquidity_pool: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
            })
        }
        pub fn encode(&self) -> Vec<u8> {
            let data = ethabi::encode(&[ethabi::Token::Address(ethabi::Address::from_slice(
                &self.u_liquidity_pool,
            ))]);
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
    impl substreams_ethereum::Function for Initialize {
        const NAME: &'static str = "initialize";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct LiquidityPool {}
    impl LiquidityPool {
        const METHOD_ID: [u8; 4] = [102u8, 90u8, 17u8, 202u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
            let mut values = ethabi::decode(&[ethabi::ParamType::Address], data.as_ref())
                .map_err(|e| format!("unable to decode output data: {:?}", e))?;
            Ok(values
                .pop()
                .expect("one output data should have existed")
                .into_address()
                .expect(INTERNAL_ERR)
                .as_bytes()
                .to_vec())
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
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for LiquidityPool {
        const NAME: &'static str = "liquidityPool";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<Vec<u8>> for LiquidityPool {
        fn output(data: &[u8]) -> Result<Vec<u8>, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct MintShares {
        pub u_user: Vec<u8>,
        pub u_share: substreams::scalar::BigInt,
    }
    impl MintShares {
        const METHOD_ID: [u8; 4] = [82u8, 140u8, 25u8, 138u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
                u_user: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                u_share: {
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
            let data = ethabi::encode(&[
                ethabi::Token::Address(ethabi::Address::from_slice(&self.u_user)),
                ethabi::Token::Uint(ethabi::Uint::from_big_endian(
                    match self.u_share.clone().to_bytes_be() {
                        (num_bigint::Sign::Plus, bytes) => bytes,
                        (num_bigint::Sign::NoSign, bytes) => bytes,
                        (num_bigint::Sign::Minus, _) => {
                            panic!("negative numbers are not supported")
                        }
                    }
                    .as_slice(),
                )),
            ]);
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
    impl substreams_ethereum::Function for MintShares {
        const NAME: &'static str = "mintShares";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Name {}
    impl Name {
        const METHOD_ID: [u8; 4] = [6u8, 253u8, 222u8, 3u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
        ) -> Result<String, String> {
            Self::output(call.return_data.as_ref())
        }
        pub fn output(data: &[u8]) -> Result<String, String> {
            let mut values = ethabi::decode(&[ethabi::ParamType::String], data.as_ref())
                .map_err(|e| format!("unable to decode output data: {:?}", e))?;
            Ok(values
                .pop()
                .expect("one output data should have existed")
                .into_string()
                .expect(INTERNAL_ERR))
        }
        pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            match call.input.get(0..4) {
                Some(signature) => Self::METHOD_ID == signature,
                None => false,
            }
        }
        pub fn call(&self, address: Vec<u8>) -> Option<String> {
            use substreams_ethereum::pb::eth::rpc;
            let rpc_calls = rpc::RpcCalls {
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for Name {
        const NAME: &'static str = "name";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<String> for Name {
        fn output(data: &[u8]) -> Result<String, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Nonces {
        pub owner: Vec<u8>,
    }
    impl Nonces {
        const METHOD_ID: [u8; 4] = [126u8, 206u8, 190u8, 0u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            let maybe_data = call.input.get(4..);
            if maybe_data.is_none() {
                return Err("no data to decode".to_string());
            }
            let mut values = ethabi::decode(&[ethabi::ParamType::Address], maybe_data.unwrap())
                .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
            values.reverse();
            Ok(Self {
                owner: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
            })
        }
        pub fn encode(&self) -> Vec<u8> {
            let data =
                ethabi::encode(&[ethabi::Token::Address(ethabi::Address::from_slice(&self.owner))]);
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
            let mut values = ethabi::decode(&[ethabi::ParamType::Uint(256usize)], data.as_ref())
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
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for Nonces {
        const NAME: &'static str = "nonces";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt> for Nonces {
        fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Owner {}
    impl Owner {
        const METHOD_ID: [u8; 4] = [141u8, 165u8, 203u8, 91u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
            let mut values = ethabi::decode(&[ethabi::ParamType::Address], data.as_ref())
                .map_err(|e| format!("unable to decode output data: {:?}", e))?;
            Ok(values
                .pop()
                .expect("one output data should have existed")
                .into_address()
                .expect(INTERNAL_ERR)
                .as_bytes()
                .to_vec())
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
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for Owner {
        const NAME: &'static str = "owner";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<Vec<u8>> for Owner {
        fn output(data: &[u8]) -> Result<Vec<u8>, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Permit {
        pub owner: Vec<u8>,
        pub spender: Vec<u8>,
        pub value: substreams::scalar::BigInt,
        pub deadline: substreams::scalar::BigInt,
        pub v: substreams::scalar::BigInt,
        pub r: [u8; 32usize],
        pub s: [u8; 32usize],
    }
    impl Permit {
        const METHOD_ID: [u8; 4] = [213u8, 5u8, 172u8, 207u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            let maybe_data = call.input.get(4..);
            if maybe_data.is_none() {
                return Err("no data to decode".to_string());
            }
            let mut values = ethabi::decode(
                &[
                    ethabi::ParamType::Address,
                    ethabi::ParamType::Address,
                    ethabi::ParamType::Uint(256usize),
                    ethabi::ParamType::Uint(256usize),
                    ethabi::ParamType::Uint(8usize),
                    ethabi::ParamType::FixedBytes(32usize),
                    ethabi::ParamType::FixedBytes(32usize),
                ],
                maybe_data.unwrap(),
            )
            .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
            values.reverse();
            Ok(Self {
                owner: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                spender: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                value: {
                    let mut v = [0 as u8; 32];
                    values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_uint()
                        .expect(INTERNAL_ERR)
                        .to_big_endian(v.as_mut_slice());
                    substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                },
                deadline: {
                    let mut v = [0 as u8; 32];
                    values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_uint()
                        .expect(INTERNAL_ERR)
                        .to_big_endian(v.as_mut_slice());
                    substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                },
                v: {
                    let mut v = [0 as u8; 32];
                    values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_uint()
                        .expect(INTERNAL_ERR)
                        .to_big_endian(v.as_mut_slice());
                    substreams::scalar::BigInt::from_unsigned_bytes_be(&v)
                },
                r: {
                    let mut result = [0u8; 32];
                    let v = values
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_fixed_bytes()
                        .expect(INTERNAL_ERR);
                    result.copy_from_slice(&v);
                    result
                },
                s: {
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
            let data = ethabi::encode(&[
                ethabi::Token::Address(ethabi::Address::from_slice(&self.owner)),
                ethabi::Token::Address(ethabi::Address::from_slice(&self.spender)),
                ethabi::Token::Uint(ethabi::Uint::from_big_endian(
                    match self.value.clone().to_bytes_be() {
                        (num_bigint::Sign::Plus, bytes) => bytes,
                        (num_bigint::Sign::NoSign, bytes) => bytes,
                        (num_bigint::Sign::Minus, _) => {
                            panic!("negative numbers are not supported")
                        }
                    }
                    .as_slice(),
                )),
                ethabi::Token::Uint(ethabi::Uint::from_big_endian(
                    match self.deadline.clone().to_bytes_be() {
                        (num_bigint::Sign::Plus, bytes) => bytes,
                        (num_bigint::Sign::NoSign, bytes) => bytes,
                        (num_bigint::Sign::Minus, _) => {
                            panic!("negative numbers are not supported")
                        }
                    }
                    .as_slice(),
                )),
                ethabi::Token::Uint(ethabi::Uint::from_big_endian(
                    match self.v.clone().to_bytes_be() {
                        (num_bigint::Sign::Plus, bytes) => bytes,
                        (num_bigint::Sign::NoSign, bytes) => bytes,
                        (num_bigint::Sign::Minus, _) => {
                            panic!("negative numbers are not supported")
                        }
                    }
                    .as_slice(),
                )),
                ethabi::Token::FixedBytes(self.r.as_ref().to_vec()),
                ethabi::Token::FixedBytes(self.s.as_ref().to_vec()),
            ]);
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
    impl substreams_ethereum::Function for Permit {
        const NAME: &'static str = "permit";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct ProxiableUuid {}
    impl ProxiableUuid {
        const METHOD_ID: [u8; 4] = [82u8, 209u8, 144u8, 45u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
            let mut values =
                ethabi::decode(&[ethabi::ParamType::FixedBytes(32usize)], data.as_ref())
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
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for ProxiableUuid {
        const NAME: &'static str = "proxiableUUID";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<[u8; 32usize]> for ProxiableUuid {
        fn output(data: &[u8]) -> Result<[u8; 32usize], String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct RenounceOwnership {}
    impl RenounceOwnership {
        const METHOD_ID: [u8; 4] = [113u8, 80u8, 24u8, 166u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Ok(Self {})
        }
        pub fn encode(&self) -> Vec<u8> {
            let data = ethabi::encode(&[]);
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
    impl substreams_ethereum::Function for RenounceOwnership {
        const NAME: &'static str = "renounceOwnership";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Shares {
        pub param0: Vec<u8>,
    }
    impl Shares {
        const METHOD_ID: [u8; 4] = [206u8, 124u8, 42u8, 194u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            let maybe_data = call.input.get(4..);
            if maybe_data.is_none() {
                return Err("no data to decode".to_string());
            }
            let mut values = ethabi::decode(&[ethabi::ParamType::Address], maybe_data.unwrap())
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
            })
        }
        pub fn encode(&self) -> Vec<u8> {
            let data = ethabi::encode(&[ethabi::Token::Address(ethabi::Address::from_slice(
                &self.param0,
            ))]);
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
            let mut values = ethabi::decode(&[ethabi::ParamType::Uint(256usize)], data.as_ref())
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
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for Shares {
        const NAME: &'static str = "shares";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt> for Shares {
        fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Symbol {}
    impl Symbol {
        const METHOD_ID: [u8; 4] = [149u8, 216u8, 155u8, 65u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
        ) -> Result<String, String> {
            Self::output(call.return_data.as_ref())
        }
        pub fn output(data: &[u8]) -> Result<String, String> {
            let mut values = ethabi::decode(&[ethabi::ParamType::String], data.as_ref())
                .map_err(|e| format!("unable to decode output data: {:?}", e))?;
            Ok(values
                .pop()
                .expect("one output data should have existed")
                .into_string()
                .expect(INTERNAL_ERR))
        }
        pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            match call.input.get(0..4) {
                Some(signature) => Self::METHOD_ID == signature,
                None => false,
            }
        }
        pub fn call(&self, address: Vec<u8>) -> Option<String> {
            use substreams_ethereum::pb::eth::rpc;
            let rpc_calls = rpc::RpcCalls {
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for Symbol {
        const NAME: &'static str = "symbol";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<String> for Symbol {
        fn output(data: &[u8]) -> Result<String, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct TotalShares {}
    impl TotalShares {
        const METHOD_ID: [u8; 4] = [58u8, 152u8, 239u8, 57u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
            let mut values = ethabi::decode(&[ethabi::ParamType::Uint(256usize)], data.as_ref())
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
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for TotalShares {
        const NAME: &'static str = "totalShares";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt> for TotalShares {
        fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct TotalSupply {}
    impl TotalSupply {
        const METHOD_ID: [u8; 4] = [24u8, 22u8, 13u8, 221u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
            let mut values = ethabi::decode(&[ethabi::ParamType::Uint(256usize)], data.as_ref())
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
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for TotalSupply {
        const NAME: &'static str = "totalSupply";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<substreams::scalar::BigInt> for TotalSupply {
        fn output(data: &[u8]) -> Result<substreams::scalar::BigInt, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Transfer {
        pub u_recipient: Vec<u8>,
        pub u_amount: substreams::scalar::BigInt,
    }
    impl Transfer {
        const METHOD_ID: [u8; 4] = [169u8, 5u8, 156u8, 187u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
                u_recipient: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                u_amount: {
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
            let data = ethabi::encode(&[
                ethabi::Token::Address(ethabi::Address::from_slice(&self.u_recipient)),
                ethabi::Token::Uint(ethabi::Uint::from_big_endian(
                    match self.u_amount.clone().to_bytes_be() {
                        (num_bigint::Sign::Plus, bytes) => bytes,
                        (num_bigint::Sign::NoSign, bytes) => bytes,
                        (num_bigint::Sign::Minus, _) => {
                            panic!("negative numbers are not supported")
                        }
                    }
                    .as_slice(),
                )),
            ]);
            let mut encoded = Vec::with_capacity(4 + data.len());
            encoded.extend(Self::METHOD_ID);
            encoded.extend(data);
            encoded
        }
        pub fn output_call(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<bool, String> {
            Self::output(call.return_data.as_ref())
        }
        pub fn output(data: &[u8]) -> Result<bool, String> {
            let mut values = ethabi::decode(&[ethabi::ParamType::Bool], data.as_ref())
                .map_err(|e| format!("unable to decode output data: {:?}", e))?;
            Ok(values
                .pop()
                .expect("one output data should have existed")
                .into_bool()
                .expect(INTERNAL_ERR))
        }
        pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            match call.input.get(0..4) {
                Some(signature) => Self::METHOD_ID == signature,
                None => false,
            }
        }
        pub fn call(&self, address: Vec<u8>) -> Option<bool> {
            use substreams_ethereum::pb::eth::rpc;
            let rpc_calls = rpc::RpcCalls {
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for Transfer {
        const NAME: &'static str = "transfer";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<bool> for Transfer {
        fn output(data: &[u8]) -> Result<bool, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct TransferFrom {
        pub u_sender: Vec<u8>,
        pub u_recipient: Vec<u8>,
        pub u_amount: substreams::scalar::BigInt,
    }
    impl TransferFrom {
        const METHOD_ID: [u8; 4] = [35u8, 184u8, 114u8, 221u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            let maybe_data = call.input.get(4..);
            if maybe_data.is_none() {
                return Err("no data to decode".to_string());
            }
            let mut values = ethabi::decode(
                &[
                    ethabi::ParamType::Address,
                    ethabi::ParamType::Address,
                    ethabi::ParamType::Uint(256usize),
                ],
                maybe_data.unwrap(),
            )
            .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
            values.reverse();
            Ok(Self {
                u_sender: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                u_recipient: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                u_amount: {
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
            let data = ethabi::encode(&[
                ethabi::Token::Address(ethabi::Address::from_slice(&self.u_sender)),
                ethabi::Token::Address(ethabi::Address::from_slice(&self.u_recipient)),
                ethabi::Token::Uint(ethabi::Uint::from_big_endian(
                    match self.u_amount.clone().to_bytes_be() {
                        (num_bigint::Sign::Plus, bytes) => bytes,
                        (num_bigint::Sign::NoSign, bytes) => bytes,
                        (num_bigint::Sign::Minus, _) => {
                            panic!("negative numbers are not supported")
                        }
                    }
                    .as_slice(),
                )),
            ]);
            let mut encoded = Vec::with_capacity(4 + data.len());
            encoded.extend(Self::METHOD_ID);
            encoded.extend(data);
            encoded
        }
        pub fn output_call(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<bool, String> {
            Self::output(call.return_data.as_ref())
        }
        pub fn output(data: &[u8]) -> Result<bool, String> {
            let mut values = ethabi::decode(&[ethabi::ParamType::Bool], data.as_ref())
                .map_err(|e| format!("unable to decode output data: {:?}", e))?;
            Ok(values
                .pop()
                .expect("one output data should have existed")
                .into_bool()
                .expect(INTERNAL_ERR))
        }
        pub fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            match call.input.get(0..4) {
                Some(signature) => Self::METHOD_ID == signature,
                None => false,
            }
        }
        pub fn call(&self, address: Vec<u8>) -> Option<bool> {
            use substreams_ethereum::pb::eth::rpc;
            let rpc_calls = rpc::RpcCalls {
                calls: vec![rpc::RpcCall { to_addr: address, data: self.encode() }],
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
                        Self::NAME,
                        err
                    );
                    None
                }
            }
        }
    }
    impl substreams_ethereum::Function for TransferFrom {
        const NAME: &'static str = "transferFrom";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    impl substreams_ethereum::rpc::RPCDecodable<bool> for TransferFrom {
        fn output(data: &[u8]) -> Result<bool, String> {
            Self::output(data)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct TransferOwnership {
        pub new_owner: Vec<u8>,
    }
    impl TransferOwnership {
        const METHOD_ID: [u8; 4] = [242u8, 253u8, 227u8, 139u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            let maybe_data = call.input.get(4..);
            if maybe_data.is_none() {
                return Err("no data to decode".to_string());
            }
            let mut values = ethabi::decode(&[ethabi::ParamType::Address], maybe_data.unwrap())
                .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
            values.reverse();
            Ok(Self {
                new_owner: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
            })
        }
        pub fn encode(&self) -> Vec<u8> {
            let data = ethabi::encode(&[ethabi::Token::Address(ethabi::Address::from_slice(
                &self.new_owner,
            ))]);
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
    impl substreams_ethereum::Function for TransferOwnership {
        const NAME: &'static str = "transferOwnership";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct UpgradeTo {
        pub new_implementation: Vec<u8>,
    }
    impl UpgradeTo {
        const METHOD_ID: [u8; 4] = [54u8, 89u8, 207u8, 230u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            let maybe_data = call.input.get(4..);
            if maybe_data.is_none() {
                return Err("no data to decode".to_string());
            }
            let mut values = ethabi::decode(&[ethabi::ParamType::Address], maybe_data.unwrap())
                .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
            values.reverse();
            Ok(Self {
                new_implementation: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
            })
        }
        pub fn encode(&self) -> Vec<u8> {
            let data = ethabi::encode(&[ethabi::Token::Address(ethabi::Address::from_slice(
                &self.new_implementation,
            ))]);
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
    impl substreams_ethereum::Function for UpgradeTo {
        const NAME: &'static str = "upgradeTo";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            Self::decode(call)
        }
        fn encode(&self) -> Vec<u8> {
            self.encode()
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct UpgradeToAndCall {
        pub new_implementation: Vec<u8>,
        pub data: Vec<u8>,
    }
    impl UpgradeToAndCall {
        const METHOD_ID: [u8; 4] = [79u8, 30u8, 242u8, 134u8];
        pub fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
            let maybe_data = call.input.get(4..);
            if maybe_data.is_none() {
                return Err("no data to decode".to_string());
            }
            let mut values = ethabi::decode(
                &[ethabi::ParamType::Address, ethabi::ParamType::Bytes],
                maybe_data.unwrap(),
            )
            .map_err(|e| format!("unable to decode call.input: {:?}", e))?;
            values.reverse();
            Ok(Self {
                new_implementation: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                data: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_bytes()
                    .expect(INTERNAL_ERR),
            })
        }
        pub fn encode(&self) -> Vec<u8> {
            let data = ethabi::encode(&[
                ethabi::Token::Address(ethabi::Address::from_slice(&self.new_implementation)),
                ethabi::Token::Bytes(self.data.clone()),
            ]);
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
    impl substreams_ethereum::Function for UpgradeToAndCall {
        const NAME: &'static str = "upgradeToAndCall";
        fn match_call(call: &substreams_ethereum::pb::eth::v2::Call) -> bool {
            Self::match_call(call)
        }
        fn decode(call: &substreams_ethereum::pb::eth::v2::Call) -> Result<Self, String> {
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
    pub struct AdminChanged1 {
        pub previous_admin: Vec<u8>,
        pub new_admin: Vec<u8>,
    }
    impl AdminChanged1 {
        const TOPIC_ID: [u8; 32] = [
            126u8, 100u8, 77u8, 121u8, 66u8, 47u8, 23u8, 192u8, 30u8, 72u8, 148u8, 181u8, 244u8,
            245u8, 136u8, 211u8, 49u8, 235u8, 250u8, 40u8, 101u8, 61u8, 66u8, 174u8, 131u8, 45u8,
            197u8, 158u8, 56u8, 201u8, 121u8, 143u8,
        ];
        pub fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            if log.topics.len() != 1usize {
                return false;
            }
            if log.data.len() != 64usize {
                return false;
            }
            return log
                .topics
                .get(0)
                .expect("bounds already checked")
                .as_ref() ==
                Self::TOPIC_ID;
        }
        pub fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            let mut values = ethabi::decode(
                &[ethabi::ParamType::Address, ethabi::ParamType::Address],
                log.data.as_ref(),
            )
            .map_err(|e| format!("unable to decode log.data: {:?}", e))?;
            values.reverse();
            Ok(Self {
                previous_admin: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                new_admin: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
            })
        }
    }
    impl substreams_ethereum::Event for AdminChanged1 {
        const NAME: &'static str = "AdminChanged1";
        fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            Self::match_log(log)
        }
        fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Self::decode(log)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct AdminChanged2 {
        pub previous_admin: Vec<u8>,
        pub new_admin: Vec<u8>,
    }
    impl AdminChanged2 {
        const TOPIC_ID: [u8; 32] = [
            126u8, 100u8, 77u8, 121u8, 66u8, 47u8, 23u8, 192u8, 30u8, 72u8, 148u8, 181u8, 244u8,
            245u8, 136u8, 211u8, 49u8, 235u8, 250u8, 40u8, 101u8, 61u8, 66u8, 174u8, 131u8, 45u8,
            197u8, 158u8, 56u8, 201u8, 121u8, 143u8,
        ];
        pub fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            if log.topics.len() != 1usize {
                return false;
            }
            if log.data.len() != 64usize {
                return false;
            }
            return log
                .topics
                .get(0)
                .expect("bounds already checked")
                .as_ref() ==
                Self::TOPIC_ID;
        }
        pub fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            let mut values = ethabi::decode(
                &[ethabi::ParamType::Address, ethabi::ParamType::Address],
                log.data.as_ref(),
            )
            .map_err(|e| format!("unable to decode log.data: {:?}", e))?;
            values.reverse();
            Ok(Self {
                previous_admin: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                new_admin: values
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
            })
        }
    }
    impl substreams_ethereum::Event for AdminChanged2 {
        const NAME: &'static str = "AdminChanged2";
        fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            Self::match_log(log)
        }
        fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Self::decode(log)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Approval {
        pub owner: Vec<u8>,
        pub spender: Vec<u8>,
        pub value: substreams::scalar::BigInt,
    }
    impl Approval {
        const TOPIC_ID: [u8; 32] = [
            140u8, 91u8, 225u8, 229u8, 235u8, 236u8, 125u8, 91u8, 209u8, 79u8, 113u8, 66u8, 125u8,
            30u8, 132u8, 243u8, 221u8, 3u8, 20u8, 192u8, 247u8, 178u8, 41u8, 30u8, 91u8, 32u8,
            10u8, 200u8, 199u8, 195u8, 185u8, 37u8,
        ];
        pub fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            if log.topics.len() != 3usize {
                return false;
            }
            if log.data.len() != 32usize {
                return false;
            }
            return log
                .topics
                .get(0)
                .expect("bounds already checked")
                .as_ref() ==
                Self::TOPIC_ID;
        }
        pub fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            let mut values =
                ethabi::decode(&[ethabi::ParamType::Uint(256usize)], log.data.as_ref())
                    .map_err(|e| format!("unable to decode log.data: {:?}", e))?;
            values.reverse();
            Ok(Self {
                owner: ethabi::decode(&[ethabi::ParamType::Address], log.topics[1usize].as_ref())
                    .map_err(|e| {
                        format!(
                            "unable to decode param 'owner' from topic of type 'address': {:?}",
                            e
                        )
                    })?
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                spender: ethabi::decode(&[ethabi::ParamType::Address], log.topics[2usize].as_ref())
                    .map_err(|e| {
                        format!(
                            "unable to decode param 'spender' from topic of type 'address': {:?}",
                            e
                        )
                    })?
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                value: {
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
    impl substreams_ethereum::Event for Approval {
        const NAME: &'static str = "Approval";
        fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            Self::match_log(log)
        }
        fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Self::decode(log)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct BeaconUpgraded1 {
        pub beacon: Vec<u8>,
    }
    impl BeaconUpgraded1 {
        const TOPIC_ID: [u8; 32] = [
            28u8, 243u8, 176u8, 58u8, 108u8, 241u8, 159u8, 162u8, 186u8, 186u8, 77u8, 241u8, 72u8,
            233u8, 220u8, 171u8, 237u8, 234u8, 127u8, 138u8, 92u8, 7u8, 132u8, 14u8, 32u8, 126u8,
            92u8, 8u8, 155u8, 233u8, 93u8, 62u8,
        ];
        pub fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            if log.topics.len() != 2usize {
                return false;
            }
            if log.data.len() != 0usize {
                return false;
            }
            return log
                .topics
                .get(0)
                .expect("bounds already checked")
                .as_ref() ==
                Self::TOPIC_ID;
        }
        pub fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Ok(Self {
                beacon: ethabi::decode(&[ethabi::ParamType::Address], log.topics[1usize].as_ref())
                    .map_err(|e| {
                        format!(
                            "unable to decode param 'beacon' from topic of type 'address': {:?}",
                            e
                        )
                    })?
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
            })
        }
    }
    impl substreams_ethereum::Event for BeaconUpgraded1 {
        const NAME: &'static str = "BeaconUpgraded1";
        fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            Self::match_log(log)
        }
        fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Self::decode(log)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct BeaconUpgraded2 {
        pub beacon: Vec<u8>,
    }
    impl BeaconUpgraded2 {
        const TOPIC_ID: [u8; 32] = [
            28u8, 243u8, 176u8, 58u8, 108u8, 241u8, 159u8, 162u8, 186u8, 186u8, 77u8, 241u8, 72u8,
            233u8, 220u8, 171u8, 237u8, 234u8, 127u8, 138u8, 92u8, 7u8, 132u8, 14u8, 32u8, 126u8,
            92u8, 8u8, 155u8, 233u8, 93u8, 62u8,
        ];
        pub fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            if log.topics.len() != 2usize {
                return false;
            }
            if log.data.len() != 0usize {
                return false;
            }
            return log
                .topics
                .get(0)
                .expect("bounds already checked")
                .as_ref() ==
                Self::TOPIC_ID;
        }
        pub fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Ok(Self {
                beacon: ethabi::decode(&[ethabi::ParamType::Address], log.topics[1usize].as_ref())
                    .map_err(|e| {
                        format!(
                            "unable to decode param 'beacon' from topic of type 'address': {:?}",
                            e
                        )
                    })?
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
            })
        }
    }
    impl substreams_ethereum::Event for BeaconUpgraded2 {
        const NAME: &'static str = "BeaconUpgraded2";
        fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            Self::match_log(log)
        }
        fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Self::decode(log)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Initialized {
        pub version: substreams::scalar::BigInt,
    }
    impl Initialized {
        const TOPIC_ID: [u8; 32] = [
            127u8, 38u8, 184u8, 63u8, 249u8, 110u8, 31u8, 43u8, 106u8, 104u8, 47u8, 19u8, 56u8,
            82u8, 246u8, 121u8, 138u8, 9u8, 196u8, 101u8, 218u8, 149u8, 146u8, 20u8, 96u8, 206u8,
            251u8, 56u8, 71u8, 64u8, 36u8, 152u8,
        ];
        pub fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            if log.topics.len() != 1usize {
                return false;
            }
            if log.data.len() != 32usize {
                return false;
            }
            return log
                .topics
                .get(0)
                .expect("bounds already checked")
                .as_ref() ==
                Self::TOPIC_ID;
        }
        pub fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            let mut values = ethabi::decode(&[ethabi::ParamType::Uint(8usize)], log.data.as_ref())
                .map_err(|e| format!("unable to decode log.data: {:?}", e))?;
            values.reverse();
            Ok(Self {
                version: {
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
    impl substreams_ethereum::Event for Initialized {
        const NAME: &'static str = "Initialized";
        fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            Self::match_log(log)
        }
        fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Self::decode(log)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct OwnershipTransferred {
        pub previous_owner: Vec<u8>,
        pub new_owner: Vec<u8>,
    }
    impl OwnershipTransferred {
        const TOPIC_ID: [u8; 32] = [
            139u8, 224u8, 7u8, 156u8, 83u8, 22u8, 89u8, 20u8, 19u8, 68u8, 205u8, 31u8, 208u8,
            164u8, 242u8, 132u8, 25u8, 73u8, 127u8, 151u8, 34u8, 163u8, 218u8, 175u8, 227u8, 180u8,
            24u8, 111u8, 107u8, 100u8, 87u8, 224u8,
        ];
        pub fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            if log.topics.len() != 3usize {
                return false;
            }
            if log.data.len() != 0usize {
                return false;
            }
            return log
                .topics
                .get(0)
                .expect("bounds already checked")
                .as_ref() ==
                Self::TOPIC_ID;
        }
        pub fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Ok(Self {
                    previous_owner: ethabi::decode(
                            &[ethabi::ParamType::Address],
                            log.topics[1usize].as_ref(),
                        )
                        .map_err(|e| {
                            format!(
                                "unable to decode param 'previous_owner' from topic of type 'address': {:?}",
                                e
                            )
                        })?
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                    new_owner: ethabi::decode(
                            &[ethabi::ParamType::Address],
                            log.topics[2usize].as_ref(),
                        )
                        .map_err(|e| {
                            format!(
                                "unable to decode param 'new_owner' from topic of type 'address': {:?}",
                                e
                            )
                        })?
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                })
        }
    }
    impl substreams_ethereum::Event for OwnershipTransferred {
        const NAME: &'static str = "OwnershipTransferred";
        fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            Self::match_log(log)
        }
        fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Self::decode(log)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Transfer {
        pub from: Vec<u8>,
        pub to: Vec<u8>,
        pub value: substreams::scalar::BigInt,
    }
    impl Transfer {
        const TOPIC_ID: [u8; 32] = [
            221u8, 242u8, 82u8, 173u8, 27u8, 226u8, 200u8, 155u8, 105u8, 194u8, 176u8, 104u8,
            252u8, 55u8, 141u8, 170u8, 149u8, 43u8, 167u8, 241u8, 99u8, 196u8, 161u8, 22u8, 40u8,
            245u8, 90u8, 77u8, 245u8, 35u8, 179u8, 239u8,
        ];
        pub fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            if log.topics.len() != 3usize {
                return false;
            }
            if log.data.len() != 32usize {
                return false;
            }
            return log
                .topics
                .get(0)
                .expect("bounds already checked")
                .as_ref() ==
                Self::TOPIC_ID;
        }
        pub fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            let mut values =
                ethabi::decode(&[ethabi::ParamType::Uint(256usize)], log.data.as_ref())
                    .map_err(|e| format!("unable to decode log.data: {:?}", e))?;
            values.reverse();
            Ok(Self {
                from: ethabi::decode(&[ethabi::ParamType::Address], log.topics[1usize].as_ref())
                    .map_err(|e| {
                        format!(
                            "unable to decode param 'from' from topic of type 'address': {:?}",
                            e
                        )
                    })?
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                to: ethabi::decode(&[ethabi::ParamType::Address], log.topics[2usize].as_ref())
                    .map_err(|e| {
                        format!("unable to decode param 'to' from topic of type 'address': {:?}", e)
                    })?
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                value: {
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
    impl substreams_ethereum::Event for Transfer {
        const NAME: &'static str = "Transfer";
        fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            Self::match_log(log)
        }
        fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Self::decode(log)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct TransferShares {
        pub from: Vec<u8>,
        pub to: Vec<u8>,
        pub shares_value: substreams::scalar::BigInt,
    }
    impl TransferShares {
        const TOPIC_ID: [u8; 32] = [
            157u8, 156u8, 144u8, 146u8, 150u8, 217u8, 198u8, 116u8, 69u8, 28u8, 12u8, 36u8, 240u8,
            44u8, 182u8, 73u8, 129u8, 235u8, 59u8, 114u8, 127u8, 153u8, 134u8, 89u8, 57u8, 25u8,
            47u8, 136u8, 10u8, 117u8, 93u8, 203u8,
        ];
        pub fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            if log.topics.len() != 3usize {
                return false;
            }
            if log.data.len() != 32usize {
                return false;
            }
            return log
                .topics
                .get(0)
                .expect("bounds already checked")
                .as_ref() ==
                Self::TOPIC_ID;
        }
        pub fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            let mut values =
                ethabi::decode(&[ethabi::ParamType::Uint(256usize)], log.data.as_ref())
                    .map_err(|e| format!("unable to decode log.data: {:?}", e))?;
            values.reverse();
            Ok(Self {
                from: ethabi::decode(&[ethabi::ParamType::Address], log.topics[1usize].as_ref())
                    .map_err(|e| {
                        format!(
                            "unable to decode param 'from' from topic of type 'address': {:?}",
                            e
                        )
                    })?
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                to: ethabi::decode(&[ethabi::ParamType::Address], log.topics[2usize].as_ref())
                    .map_err(|e| {
                        format!("unable to decode param 'to' from topic of type 'address': {:?}", e)
                    })?
                    .pop()
                    .expect(INTERNAL_ERR)
                    .into_address()
                    .expect(INTERNAL_ERR)
                    .as_bytes()
                    .to_vec(),
                shares_value: {
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
    impl substreams_ethereum::Event for TransferShares {
        const NAME: &'static str = "TransferShares";
        fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            Self::match_log(log)
        }
        fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Self::decode(log)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Upgraded1 {
        pub implementation: Vec<u8>,
    }
    impl Upgraded1 {
        const TOPIC_ID: [u8; 32] = [
            188u8, 124u8, 215u8, 90u8, 32u8, 238u8, 39u8, 253u8, 154u8, 222u8, 186u8, 179u8, 32u8,
            65u8, 247u8, 85u8, 33u8, 77u8, 188u8, 107u8, 255u8, 169u8, 12u8, 192u8, 34u8, 91u8,
            57u8, 218u8, 46u8, 92u8, 45u8, 59u8,
        ];
        pub fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            if log.topics.len() != 2usize {
                return false;
            }
            if log.data.len() != 0usize {
                return false;
            }
            return log
                .topics
                .get(0)
                .expect("bounds already checked")
                .as_ref() ==
                Self::TOPIC_ID;
        }
        pub fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Ok(Self {
                    implementation: ethabi::decode(
                            &[ethabi::ParamType::Address],
                            log.topics[1usize].as_ref(),
                        )
                        .map_err(|e| {
                            format!(
                                "unable to decode param 'implementation' from topic of type 'address': {:?}",
                                e
                            )
                        })?
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                })
        }
    }
    impl substreams_ethereum::Event for Upgraded1 {
        const NAME: &'static str = "Upgraded1";
        fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            Self::match_log(log)
        }
        fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Self::decode(log)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct Upgraded2 {
        pub implementation: Vec<u8>,
    }
    impl Upgraded2 {
        const TOPIC_ID: [u8; 32] = [
            188u8, 124u8, 215u8, 90u8, 32u8, 238u8, 39u8, 253u8, 154u8, 222u8, 186u8, 179u8, 32u8,
            65u8, 247u8, 85u8, 33u8, 77u8, 188u8, 107u8, 255u8, 169u8, 12u8, 192u8, 34u8, 91u8,
            57u8, 218u8, 46u8, 92u8, 45u8, 59u8,
        ];
        pub fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            if log.topics.len() != 2usize {
                return false;
            }
            if log.data.len() != 0usize {
                return false;
            }
            return log
                .topics
                .get(0)
                .expect("bounds already checked")
                .as_ref() ==
                Self::TOPIC_ID;
        }
        pub fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Ok(Self {
                    implementation: ethabi::decode(
                            &[ethabi::ParamType::Address],
                            log.topics[1usize].as_ref(),
                        )
                        .map_err(|e| {
                            format!(
                                "unable to decode param 'implementation' from topic of type 'address': {:?}",
                                e
                            )
                        })?
                        .pop()
                        .expect(INTERNAL_ERR)
                        .into_address()
                        .expect(INTERNAL_ERR)
                        .as_bytes()
                        .to_vec(),
                })
        }
    }
    impl substreams_ethereum::Event for Upgraded2 {
        const NAME: &'static str = "Upgraded2";
        fn match_log(log: &substreams_ethereum::pb::eth::v2::Log) -> bool {
            Self::match_log(log)
        }
        fn decode(log: &substreams_ethereum::pb::eth::v2::Log) -> Result<Self, String> {
            Self::decode(log)
        }
    }
}
