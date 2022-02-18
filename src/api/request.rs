use std::{collections::HashMap, fmt::Display};
use reqwest::{RequestBuilder, StatusCode};
use serde::Serialize;
use tokio::time::Duration;

use super::{ApiError, Api, structs::*};

// 1-500
const PAGINATE_LIMIT: i32 = 100;

#[derive(Clone)]
pub enum ApiRequestBody<T> {
    None,
    Json(T)
}

//impl<T: Display> Display for ApiRequestBody<T> {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        match self {
//            ApiRequestBody::None => write!(f, "(no body)"),
//            ApiRequestBody::Json(b) => write!(f, "{}", b),
//        }
//    }
//}

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
    Get
}

impl Display for ApiRequestKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiRequestKind::Get => write!(f, "GET"),
            ApiRequestKind::Post => write!(f, "POST"),
        }
    }
}

pub struct ApiRequest<A,B,R> {
    pub include: Vec<String>,
    pub query: HashMap<String, String>,
    pub kind: ApiRequestKind,
    pub endpoint: String,
    pub body: ApiRequestBody<A>,
    pub done: fn(&mut Api, B) -> Result<R, ApiError>,
}

impl<A:Serialize,B,R> Display for ApiRequest<A,B,R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} request to {} with {} {:?} {:?}", self.kind, self.endpoint, self.body, self.query, self.include)
    }
}

impl<A: Clone,B,R> Clone for ApiRequest<A,B,R> {
    fn clone(&self) -> Self {
        Self {
            include: self.include.clone(),
            query: self.query.clone(),
            kind: self.kind.clone(),
            endpoint: self.endpoint.clone(),
            body: self.body.clone(),
            done: self.done.clone()
        }
    }
}

impl<A,B,R> Default for ApiRequest<A,B,R> {
    fn default() -> Self {
        Self {
            include: vec![],
            query: HashMap::new(),
            kind: ApiRequestKind::Get,
            endpoint: "/".to_owned(),
            body: ApiRequestBody::None,
            done: |_, _| Err(ApiError::Other),
        }
    }
}

impl<A,B,R> ApiRequest<A,B,R> {
}

impl<A: serde::Serialize, B: serde::de::DeserializeOwned, R> ApiRequest<A,B,R> {
    pub fn build(&self, api: &mut Api) -> Result<RequestBuilder, ApiError> {
        let mut url = api.endpoint(&self.endpoint);

        if self.include.len() > 0 {
            let query = self.include.iter().map(|x| format!("include[]={}", x))
                .chain(self.query.iter().map(|(k, v)| format!("{}={}", k, v)))
                .collect::<Vec<String>>();
            url.set_query(Some(&query.join("&")));
        }

        let mut req = match self.kind {
            ApiRequestKind::Get => api.client.get(url),
            ApiRequestKind::Post => api.client.post(url),
        };

        match self.body {
            ApiRequestBody::Json(ref s) => {
                req = req.json(s);
            },
            _ => {},
        }

        if let Some(ref session) = api.session {
            req = req.header("Authorization", session);
        }
        log::trace!("built {}", self);
        Ok(req)
    }

    pub async fn send_simple(&self, api: &mut Api) -> Result<B, ApiError> {
        let req = self.build(api)?;
        let res = req.send().await.map_err(Into::<ApiError>::into)?;
        match res.status() {
            StatusCode::OK => {
                Ok(res.json::<B>().await.expect("Error when deserializing"))
            },
            StatusCode::BAD_REQUEST => {
                Err(ApiError::BadRequest)
            },
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(ApiError::Auth)
            }
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_ts = res.headers().get("X-RateLimit-Retry-After");
                let retry_in: i64;
                if let Some(ts) = retry_ts {
                    retry_in = ts.to_str().unwrap().parse::<i64>().unwrap() - chrono::Utc::now().timestamp();
                } else {
                    retry_in = 5;
                }
                Err(ApiError::RateLimit(tokio::time::Duration::from_secs(retry_in.try_into().unwrap())))
            }
            _ => Err(ApiError::Other)
        }
    }

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

    pub async fn execute_simple(&self, api: &mut Api) -> Result<R, ApiError> {
        let data = self.send_simple(api).await?;
        (self.done)(api, data)
    }

    pub async fn execute(&self, api: &mut Api) -> Result<R, ApiError> {
        let data = self.send(api).await?;
        (self.done)(api, data)
    }
}


impl<A: serde::Serialize + Clone, B: serde::de::DeserializeOwned + Paginate, R> ApiRequest<A,B,R> {
    pub async fn send_paginated(&self, api: &mut Api) -> Result<B, ApiError> {
        let mut req = <Self as Clone>::clone(self);

        req.query.insert("limit".to_owned(), PAGINATE_LIMIT.to_string());
        let mut res = req.send(api).await?;

        let mut off = PAGINATE_LIMIT;
        let total = res.total();

        while off < total {
            req.query.insert("offset".to_owned(), off.to_string());
            res.concat(req.send(api).await?);

            off += PAGINATE_LIMIT;
        }

        Ok(res)
    }

    pub async fn execute_paginate(&self, api: &mut Api) -> Result<R, ApiError> {
        let data = self.send_paginated(api).await?;
        (self.done)(api, data)
    }
}
