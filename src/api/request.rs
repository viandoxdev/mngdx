use reqwest::{RequestBuilder, StatusCode};
use serde::Serialize;
use std::{collections::HashMap, fmt::Display};
use tokio::time::Duration;

use super::{structs::json::responses::Paginate, Api, ApiError};

#[derive(Clone)]
pub enum ApiRequestBody<T> {
    None,
    Json(T),
}

impl<T: Serialize> Display for ApiRequestBody<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiRequestBody::None => write!(f, "no body"),
            ApiRequestBody::Json(b) => write!(f, "{}", serde_json::to_string(b).unwrap()),
        }
    }
}

#[derive(Clone)]
pub enum ApiRequestKind {
    Post,
    Get,
}

impl Display for ApiRequestKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiRequestKind::Get => write!(f, "GET"),
            ApiRequestKind::Post => write!(f, "POST"),
        }
    }
}

/// Represents any request to the api.
/// A: type for the body of the request
/// B: type for the response of the request
/// R: type for the result of the processed request
/// F: closure
pub struct ApiRequest<A, B, R, I> {
    pub include: Vec<String>,
    pub query: HashMap<String, String>,
    pub kind: ApiRequestKind,
    pub endpoint: String,
    pub body: ApiRequestBody<A>,
    // stores data that can't be obtained from the api response but that is necesary for processing
    pub info: I,
    pub done: fn(&mut Api, B, &I) -> Result<R, ApiError>,
}

impl<A: Serialize, B, R, I> Display for ApiRequest<A, B, R, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} request to {} with {} {:?}",
            self.kind, self.endpoint, self.body, self.query
        )
    }
}

impl<A: Clone, B, R, I: Clone> Clone for ApiRequest<A, B, R, I> {
    fn clone(&self) -> Self {
        Self {
            include: self.include.clone(),
            query: self.query.clone(),
            kind: self.kind.clone(),
            endpoint: self.endpoint.clone(),
            body: self.body.clone(),
            info: self.info.clone(),
            done: self.done,
        }
    }
}

impl<A, B, R, I: Default> Default for ApiRequest<A, B, R, I> {
    fn default() -> Self {
        Self {
            include: vec![],
            query: HashMap::new(),
            kind: ApiRequestKind::Get,
            endpoint: "/".to_owned(),
            body: ApiRequestBody::None,
            info: I::default(),
            done: |_, _, _| Err(ApiError::Other),
        }
    }
}

impl<A, B, R, I> ApiRequest<A, B, R, I>
where
    A: serde::Serialize,
    B: serde::de::DeserializeOwned,
{
    /// Build a RequestBuilder from an ApiRequest
    pub fn build(&self, api: &mut Api) -> Result<RequestBuilder, ApiError> {
        let mut url = api.endpoint(&self.endpoint);
        let mut query = vec![];

        if !self.query.is_empty() {
            query.extend(self.query.iter().map(|(k, v)| format!("{k}={v}")));
        }

        if !self.include.is_empty() {
            query.extend(self.include.iter().map(|x| format!("include[]={}", x)));
        }

        if !query.is_empty() {
            url.set_query(Some(&query.join("&")));
        }

        let mut req = match self.kind {
            ApiRequestKind::Get => api.client.get(url),
            ApiRequestKind::Post => api.client.post(url),
        };

        if let ApiRequestBody::Json(ref s) = self.body {
            req = req.json(s);
        }

        if let Some(ref session) = api.session {
            req = req.header("Authorization", session);
        }
        log::trace!("built {}", self);
        Ok(req)
    }
    /// Send request, without any retry / auth logic.
    pub async fn send_simple(&self, api: &mut Api) -> Result<B, ApiError> {
        let req = self.build(api)?;
        let res = req.send().await.map_err(Into::<ApiError>::into)?;
        match res.status() {
            StatusCode::OK => Ok(res.json::<B>().await.expect("Error when deserializing")),
            StatusCode::BAD_REQUEST => Err(ApiError::BadRequest),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(ApiError::Auth),
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_ts = res.headers().get("X-RateLimit-Retry-After");
                let retry_in: i64;
                if let Some(ts) = retry_ts {
                    retry_in = ts.to_str().unwrap().parse::<i64>().unwrap()
                        - chrono::Utc::now().timestamp();
                } else {
                    retry_in = 5;
                }
                Err(ApiError::RateLimit(tokio::time::Duration::from_secs(
                    retry_in.try_into().unwrap(),
                )))
            }
            _ => Err(ApiError::Other),
        }
    }
    /// Sends request, tries to handle Auth and RateLimit errors.
    pub async fn send(&self, api: &mut Api) -> Result<B, ApiError> {
        let mut result = self.send_simple(api).await;

        if let Err(ApiError::RateLimit(retry)) = result {
            // sleep a little longer just in case
            tokio::time::sleep(retry + Duration::from_millis(500)).await;
            result = self.send_simple(api).await;
        }

        if let Err(ApiError::Auth) = result {
            match api.refresh().await {
                Ok(_) => {
                    result = self.send_simple(api).await;
                }
                Err(ApiError::Auth) => {
                    // TODO: ask for relogin
                    return Err(ApiError::Auth);
                }
                Err(what) => {
                    return Err(what);
                }
            }
        }

        result
    }
    /// Send and process request without any retry on error handling.
    pub async fn execute_simple(&self, api: &mut Api) -> Result<R, ApiError> {
        let data = self.send_simple(api).await?;
        (self.done)(api, data, &self.info)
    }
    /// Send request with rety on error.
    pub async fn execute(&self, api: &mut Api) -> Result<R, ApiError> {
        let data = self.send(api).await?;
        (self.done)(api, data, &self.info)
    }
}

impl<A, B, R, I> ApiRequest<A, B, R, I>
where
    A: serde::Serialize + Clone,
    B: serde::de::DeserializeOwned + Paginate,
    I: Clone,
{
    pub async fn send_paginated<const L: i32>(&self, api: &mut Api) -> Result<B, ApiError> {
        let mut req = <Self as Clone>::clone(self);

        req.query.insert("limit".to_owned(), L.to_string());
        let mut res = req.send(api).await?;

        let mut off = L;
        let total = res.total();

        while off < total {
            log::trace!("sending paginated [{}/{}]", off, total);

            req.query.insert("offset".to_owned(), off.to_string());
            res.concat(req.send(api).await?);

            off += L;
        }

        log::trace!("sending paginated [{}/{}]", total, total);
        Ok(res)
    }

    pub async fn execute_paginate<const L: i32>(&self, api: &mut Api) -> Result<R, ApiError> {
        let data = self.send_paginated::<L>(api).await?;
        (self.done)(api, data, &self.info)
    }
}
