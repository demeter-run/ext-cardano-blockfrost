use regex::{Error as RegexError, Regex};

#[derive(Debug, Clone)]
pub struct Endpoint {
    regex: Regex,
}

impl Endpoint {
    pub fn new(endpoint: &str) -> Result<Self, RegexError> {
        Ok(Self {
            regex: Regex::new(endpoint)?,
        })
    }

    pub fn matches(&self, uri: &str) -> bool {
        self.regex.is_match(uri)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_endpoint() {
        let fe = Endpoint::new(r"/network").unwrap();
        assert!(fe.matches("/network"));
        assert!(!fe.matches("/cacheable"));

        let fe = Endpoint::new(r"/pools/\w+$").unwrap();
        assert!(fe.matches("/pools/pool18v9r8afalh50l4lstct2awdc3zspnvurcs7t45nv29uc2mnxc6c"));
        assert!(
            !fe.matches("/pools/pool18v9r8afalh50l4lstct2awdc3zspnvurcs7t45nv29uc2mnxc6c/blocks")
        );
    }
}
