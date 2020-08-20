use crate::errors::PocketError;
use crate::headers::{HEADER_XACCEPT, HEADER_XERROR, HEADER_XERROR_CODE};
use crate::PocketResult;
use bytes::buf::BufExt as _;
use futures::TryFutureExt;
use hyper::client::{Client, HttpConnector};
use hyper::Body;
use hyper::Method;
use hyper::Request;
use hyper::Uri;
use hyper_tls::HttpsConnector;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::convert::{From, TryFrom};

pub struct PocketClient {
    client: Client<HttpsConnector<HttpConnector>>,
}

impl PocketClient {
    pub fn new() -> PocketClient {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);

        PocketClient { client }
    }

    pub async fn get<T, Resp>(&self, url: T) -> PocketResult<Resp>
    where
        Uri: TryFrom<T>,
        <Uri as TryFrom<T>>::Error: Into<hyper::http::Error>,
        Resp: DeserializeOwned,
    {
        let request = Request::builder().uri(url).body(Body::empty()).unwrap();
        self.request(request).await
    }

    pub async fn post<T, B, Resp>(&self, url: T, body: &B) -> PocketResult<Resp>
    where
        Uri: TryFrom<T>,
        <Uri as TryFrom<T>>::Error: Into<hyper::http::Error>,
        B: Serialize,
        Resp: DeserializeOwned,
    {
        let app_json = "application/json";
        let body = serde_json::to_string(body).map(Body::from)?;
        let request = Request::builder()
            .method(Method::POST)
            .uri(url)
            .header(hyper::header::CONTENT_TYPE, app_json)
            .header(HEADER_XACCEPT, app_json)
            .body(body)
            .unwrap();

        self.request(request).await
    }

    async fn request<Resp: DeserializeOwned>(&self, request: Request<Body>) -> PocketResult<Resp> {
        self.client
            .request(request)
            .map_err(From::from)
            .and_then(|r| async move {
                match r.headers().get(HEADER_XERROR_CODE) {
                    None => {
                        let body = hyper::body::aggregate(r).await?;
                        serde_json::from_reader(body.reader()).map_err(From::from)
                    }
                    Some(code) => Err(PocketError::Proto(
                        code.to_str().unwrap().parse().unwrap(),
                        r.headers()
                            .get(HEADER_XERROR)
                            .map(|v| v.to_str().unwrap())
                            .unwrap_or("unknown protocol error")
                            .to_string(),
                    )),
                }
            })
            .await
    }
}
