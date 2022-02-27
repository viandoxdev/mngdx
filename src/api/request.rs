use reqwest::{RequestBuilder, StatusCode};
use serde::Serialize;
use std::{fmt::Display, marker::PhantomData};
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
pub struct ApiRequest<A, B> {
    pub include: Vec<String>,
    pub query: Vec<(String, String)>,
    pub kind: ApiRequestKind,
    pub endpoint: String,
    pub body: ApiRequestBody<A>,

    pub _phantom: PhantomData<B>,
}

impl<A: Serialize, B> Display for ApiRequest<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} request to {} with {} {:?}",
            self.kind, self.endpoint, self.body, self.query
        )
    }
}

impl<A: Clone, B> Clone for ApiRequest<A, B> {
    fn clone(&self) -> Self {
        Self {
            include: self.include.clone(),
            query: self.query.clone(),
            kind: self.kind.clone(),
            endpoint: self.endpoint.clone(),
            body: self.body.clone(),

            _phantom: PhantomData,
        }
    }
}

impl<A, B> Default for ApiRequest<A, B> {
    fn default() -> Self {
        Self {
            include: vec![],
            query: Vec::new(),
            kind: ApiRequestKind::Get,
            endpoint: "/".to_owned(),
            body: ApiRequestBody::None,

            _phantom: PhantomData,
        }
    }
}

impl<A, B> ApiRequest<A, B>
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
}

impl<A, B> ApiRequest<A, B>
where
    A: serde::Serialize + Clone,
    B: serde::de::DeserializeOwned + Paginate,
{
    pub async fn send_paginated<const L: i32>(
        &self,
        api: &mut Api,
        mut offset: i32,
        mut count: i32,
    ) -> Result<B, ApiError> {
        let mut req = self.clone();
        // save query
        let query = req.query.clone();

        // Yes I know this is repeated, but its late and rust won't let me do it otherwise.

        let mut chunk = L.min(count);

        req.query.push(("limit".to_owned(), chunk.to_string()));
        req.query.push(("offset".to_owned(), offset.to_string()));

        let mut res = req.send(api).await?;

        offset += res.count();
        count = (res.total() - offset).min(count - res.count());

        while count > 0 {
            log::trace!("sending paginated...");
            // reset query
            req.query = query.clone();

            req.query.push(("limit".to_owned(), chunk.to_string()));
            req.query.push(("offset".to_owned(), offset.to_string()));

            let r = req.send(api).await?;

            offset += r.count();
            count = (r.total() - offset).min(count - r.count());
            chunk = L.min(count);

            res.concat(r);
        }
        Ok(res)
    }

    pub async fn send_paginated_all<const L: i32>(&self, api: &mut Api) -> Result<B, ApiError> {
        let mut req = self.clone();

        req.query.push(("limit".to_owned(), L.to_string()));
        // save query
        let query = req.query.clone();

        let mut res = req.send(api).await?;

        let mut off = L;
        let total = res.total();

        while off < total {
            log::trace!("sending paginated [{}/{}]", off, total);

            // restore query
            req.query = query.clone();
            // add offset
            req.query.push(("offset".to_owned(), off.to_string()));

            res.concat(req.send(api).await?);

            off += L;
        }

        log::trace!("sending paginated [{}/{}]", total, total);
        Ok(res)
    }
}
