use reqwest::{Client, multipart};
use tracing::{trace, warn};

use crate::config::Config;
use crate::error::{Result, SdkError};

const HTTP_STATUS_METHOD_NOT_ALLOWED: u16 = 405;
const HTTP_STATUS_TOO_MANY_REQUESTS: u16 = 429;

pub struct LighterRestClient {
    client: Client,
    base_url: String,
    auth_token: Option<String>,
}

impl LighterRestClient {
    pub fn new(config: &Config) -> Result<Self> {
        let mut builder = Client::builder().pool_max_idle_per_host(config.pool_size);

        if let Some(ref proxy_url) = config.proxy {
            let proxy =
                reqwest::Proxy::all(proxy_url).map_err(|e| SdkError::Other(e.to_string()))?;
            builder = builder.proxy(proxy);
        }

        let client = builder.build()?;

        Ok(Self {
            client,
            base_url: config.api_base_url(),
            auth_token: None,
        })
    }

    pub fn set_auth_token(&mut self, token: String) {
        self.auth_token = Some(token);
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.get(&url);
        if let Some(ref token) = self.auth_token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        let resp = req.send().await?;
        Self::handle_response(resp).await
    }

    pub async fn get_with_query<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.get(&url).query(query);
        if let Some(ref token) = self.auth_token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        let resp = req.send().await?;
        Self::handle_response(resp).await
    }

    pub async fn get_with_auth<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
        auth: &str,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .query(query)
            .header("Authorization", auth)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    pub async fn post_form<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        form: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.post(&url).form(form);
        if let Some(ref token) = self.auth_token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        let resp = req.send().await?;
        Self::handle_response(resp).await
    }

    pub async fn post_multipart<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        form: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut multipart_form = multipart::Form::new();
        for (key, value) in form {
            multipart_form = multipart_form.text((*key).to_string(), (*value).to_string());
        }
        let mut req = self.client.post(&url).multipart(multipart_form);
        if let Some(ref token) = self.auth_token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        let resp = req.send().await?;
        Self::handle_response(resp).await
    }

    pub async fn post_form_with_auth<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        form: &[(&str, &str)],
        auth: &str,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .post(&url)
            .form(form)
            .header("Authorization", auth)
            .send()
            .await?;
        Self::handle_response(resp).await
    }

    pub async fn post_json<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self.client.post(&url).json(body);
        if let Some(ref token) = self.auth_token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        let resp = req.send().await?;
        Self::handle_response(resp).await
    }

    async fn handle_response<T: serde::de::DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
        let status = resp.status();
        if !status.is_success() {
            let status_code = status.as_u16() as i64;
            let body = resp.text().await.unwrap_or_default();
            warn!(
                http_status = status_code,
                response_body = %body,
                "REST error response"
            );
            // Try to parse as API error envelope
            if let Ok(err) = serde_json::from_str::<serde_json::Value>(&body)
                && let (Some(code), Some(message)) = (
                    err.get("code").and_then(|c| c.as_i64()),
                    err.get("message").and_then(|m| m.as_str()),
                )
            {
                if is_rate_limited_status(status.as_u16()) {
                    return Err(SdkError::RateLimited {
                        code: status_code,
                        message: message.to_string(),
                    });
                }
                return Err(SdkError::Api {
                    code,
                    message: message.to_string(),
                });
            }
            if is_rate_limited_status(status.as_u16()) {
                return Err(SdkError::RateLimited {
                    code: status_code,
                    message: body,
                });
            }
            return Err(SdkError::Api {
                code: status_code,
                message: body,
            });
        }
        let body = resp.text().await?;
        trace!(raw_body = %body, "REST response");
        serde_json::from_str(&body).map_err(Into::into)
    }
}

fn is_rate_limited_status(status_code: u16) -> bool {
    matches!(
        status_code,
        HTTP_STATUS_METHOD_NOT_ALLOWED | HTTP_STATUS_TOO_MANY_REQUESTS
    )
}

#[cfg(test)]
mod tests {
    use super::is_rate_limited_status;

    #[test]
    fn recognizes_supported_rate_limit_status_codes() {
        assert!(is_rate_limited_status(405));
        assert!(is_rate_limited_status(429));
        assert!(!is_rate_limited_status(400));
        assert!(!is_rate_limited_status(500));
    }
}
