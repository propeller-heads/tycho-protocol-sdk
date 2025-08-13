use anyhow::Result;
use url::form_urlencoded;

pub struct Params {
    pub buffer_tokens: Vec<String>,
}

impl Params {
    pub fn parse_from_query(input: &str) -> Result<Self> {
        let values: Vec<String> = form_urlencoded::parse(input.as_bytes())
            .map(|(_k, v)| v.into_owned())
            .collect();
        Ok(Params { buffer_tokens: values })
    }

    pub fn contains_wrapped_token(&self, wrapped_token: &Vec<u8>) -> bool {
        self.buffer_tokens
            .iter()
            .any(|token_str| match hex::decode(token_str) {
                Ok(decoded) => decoded == *wrapped_token,
                Err(_) => false,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_contains() {
        let qs =
            "a=1111111111111111111111111111111111111111&b=2222222222222222222222222222222222222222";
        let params = Params::parse_from_query(qs).unwrap();

        assert_eq!(params.buffer_tokens.len(), 2);

        let token_a = hex::decode("1111111111111111111111111111111111111111").unwrap();
        assert!(params.contains_wrapped_token(&token_a));

        let token_b = hex::decode("2222222222222222222222222222222222222222").unwrap();
        assert!(params.contains_wrapped_token(&token_b));

        let token_c = hex::decode("3333333333333333333333333333333333333333").unwrap();
        assert!(!params.contains_wrapped_token(&token_c));
    }

    #[test]
    fn test_contains_invalid_hex() {
        let qs = "a=zzzz&b=1111111111111111111111111111111111111111";
        let params = Params::parse_from_query(qs).unwrap();

        let token_valid = hex::decode("1111111111111111111111111111111111111111").unwrap();
        assert!(params.contains_wrapped_token(&token_valid));

        let token_invalid = hex::decode("zzzz").ok();
        assert!(token_invalid.is_none());
    }
}
