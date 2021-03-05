// Copyright (c) Microsoft. All rights reserved.

#![deny(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::default_trait_access,
    clippy::let_and_return,
    clippy::let_unit_value,
    clippy::missing_errors_doc,
    clippy::similar_names,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::type_complexity
)]

use std::sync::Arc;

use aziot_cloud_client_async_common::{get_sas_connector, get_x509_connector};

pub const IOT_HUB_ENCODE_SET: &percent_encoding::AsciiSet =
    &http_common::PATH_SEGMENT_ENCODE_SET.add(b'=');

pub struct Client {
    device: aziot_identity_common::IoTHubDevice,
    key_client: Arc<aziot_key_client_async::Client>,
    key_engine: Arc<futures_util::lock::Mutex<openssl2::FunctionalEngine>>,
    cert_client: Arc<aziot_cert_client_async::Client>,
    tpm_client: Arc<aziot_tpm_client_async::Client>,
    proxy_uri: Option<hyper::Uri>,
}

impl Client {
    #[must_use]
    pub fn new(
        device: aziot_identity_common::IoTHubDevice,
        key_client: Arc<aziot_key_client_async::Client>,
        key_engine: Arc<futures_util::lock::Mutex<openssl2::FunctionalEngine>>,
        cert_client: Arc<aziot_cert_client_async::Client>,
        tpm_client: Arc<aziot_tpm_client_async::Client>,
        proxy_uri: Option<hyper::Uri>,
    ) -> Self {
        Client {
            device,
            key_client,
            key_engine,
            cert_client,
            tpm_client,
            proxy_uri,
        }
    }
}

impl Client {
    pub async fn get_token(&self) -> Result<String, std::io::Error> {
        let access_token_provider_uri = "mtlsauth.windows-ppe.net";
        let tenant_id = "f35bf5fa-7977-447c-a1af-4c457bad7d7e";
        let azure_resource_scope = "https://ppe.cognitiveservices.azure.com/.default";
        let app_id = "fb6b46e5-08c9-4b9f-bd4f-9c53cd0347a5";
        let device_id = "cb2fefd9-95e9-4f75-9e8f-8c2ea5bf18db";

        let params: String = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("grant_type", "sub_mtls")
            .append_pair("scope", azure_resource_scope)
            .append_pair("client_id", app_id)
            .append_pair("external_device_id", device_id)
            .append_pair("debugmodeflight", "true")
            .finish();

        let uri = format!(
            "https://{}/{}/oauth2/v2.0/token",
            access_token_provider_uri, tenant_id
        );

        // let body = AADRequest {
        //     grant_type: "sub_mtls",
        //     scope: azure_resource_scope,
        //     client_id: app_id,
        //     external_device_id: device_id,
        // };

        // println!("\n\n\nGetting token.\nUri: {}\nBody:{:#?}", uri, body);

        let res: AADResponse = self.request(http::Method::POST, &uri, Some(params)).await?;
        //::<(), AADResponse>
        Ok(res.access_token)

        // Ok("eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsIng1dCI6InZQVWt1UGdUTFd6WkNETzhocVZGOUVxdXN6byIsImtpZCI6InZQVWt1UGdUTFd6WkNETzhocVZGOUVxdXN6byJ9.eyJhdWQiOiJodHRwczovL3BwZS5jb2duaXRpdmVzZXJ2aWNlcy5henVyZS5jb20iLCJpc3MiOiJodHRwczovL3N0cy53aW5kb3dzLXBwZS5uZXQvZjM1YmY1ZmEtNzk3Ny00NDdjLWExYWYtNGM0NTdiYWQ3ZDdlLyIsImlhdCI6MTYxNDYyMTIwMCwibmJmIjoxNjE0NjIxMjAwLCJleHAiOjE2MTQ2MjUxMDAsImFpbyI6IkUyTmdZRGdsVi9weFNYVnA5aC9IU2FaWGxsVk5BZ0E9IiwiYXBwaWQiOiJmYjZiNDZlNS0wOGM5LTRiOWYtYmQ0Zi05YzUzY2QwMzQ3YTUiLCJhcHBpZGFjciI6IjIiLCJkZXZpY2VpZCI6Ijk0YmUzM2Y1LWE5ZmMtNDZjMi1iMDA3LWVhYTEzZTA3M2Q5YiIsImlkcCI6Imh0dHBzOi8vc3RzLndpbmRvd3MtcHBlLm5ldC9mMzViZjVmYS03OTc3LTQ0N2MtYTFhZi00YzQ1N2JhZDdkN2UvIiwiaXBhZGRyIjoiMTMxLjEwNy4xNjAuNjgiLCJvaWQiOiJjYjhjMjk2MS00YWFiLTQyNTEtODQ5Mi0wZjBmZDU5M2Y1ZDkiLCJyaCI6IjAuQUFBQS12VmI4M2Q1ZkVTaHIweEZlNjE5ZnVWR2FfdkpDSjlMdlUtY1U4MERSNlVCQUFBLiIsInN1YiI6Ijk0YmUzM2Y1LWE5ZmMtNDZjMi1iMDA3LWVhYTEzZTA3M2Q5YiIsInRpZCI6ImYzNWJmNWZhLTc5NzctNDQ3Yy1hMWFmLTRjNDU3YmFkN2Q3ZSIsInV0aSI6IkVaQTlhMkMwQmttU0JqRlJKMWNQQUEiLCJ2ZXIiOiIxLjAifQ.U2ozTu4qj4LGD_Y-e7N13g_noE5lyWelAyb6OkLJmdf9GZp0FUI41ZPmIk0KaPjFXUR2KICSrzfqG0VNkwh515m-QI9UEBN8xrm0X5FO1JP2N90ft5VJpSyGRJAoNKXab_g0OOAsDiOxfqs7B_0tqJTKXuTdFaP8SsybYrLXlRXm0YMOdX0R_UnDXe6dyUQkRVrQvLdI8HlqucPCUic43zdUqZ9oM4vIvOkxa7MUx9XY0kdQ5T2lQv-6udIbt8L4RrvQmMm8H2oM5PwLifrOjvSOlQ_RDkjEsHML3B6W4uoYtlMEX18FU6m0qVhN3UDdB9aQvU0PBkkGty5YIBAOoA".to_owned())
    }

    async fn request<TResponse>(
        &self,
        method: http::Method,
        uri: &str,
        body: Option<String>,
    ) -> std::io::Result<TResponse>
    where
        TResponse: serde::de::DeserializeOwned,
    {
        let req = hyper::Request::builder().method(method).uri(uri);
        // `req` is consumed by both branches, so this cannot be replaced with `Option::map_or_else`
        //
        // Ref: https://github.com/rust-lang/rust-clippy/issues/5822
        #[allow(clippy::option_if_let_else)]
        let req = if let Some(body) = body {
            // let body = serde_json::to_vec(body)
            //     .expect("serializing request body to JSON cannot fail")
            //     .into();
            req.header(
                hyper::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(body.into())
        } else {
            req.header(
                hyper::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(hyper::Body::default())
        };

        let req = req.expect("cannot fail to create hyper request");

        let connector = match self.device.credentials.clone() {
            aziot_identity_common::Credentials::SharedPrivateKey(key) => {
                return Err(std::io::Error::from(std::io::ErrorKind::Other)); //TODO: Do Error stuff
            }
            aziot_identity_common::Credentials::Tpm => {
                return Err(std::io::Error::from(std::io::ErrorKind::Other)); //TODO: Do Error stuff
            }
            aziot_identity_common::Credentials::X509 {
                identity_cert,
                identity_pk,
            } => {
                // let identity_cert = "aad-id";
                // let identity_pk = "aad-id";
                get_x509_connector(
                    &identity_cert,
                    &identity_pk,
                    &self.key_client,
                    &mut *self.key_engine.lock().await,
                    &self.cert_client,
                    self.proxy_uri.clone(),
                )
                .await?
            }
        };

        let client: hyper::Client<_, hyper::Body> = hyper::Client::builder().build(connector);
        log::debug!("AAD request {:?}", req);
        println!("AAD request {:?}", req);
        let res = client
            .request(req)
            .await
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
        println!("3");

        let (
            http::response::Parts {
                status: res_status_code,
                headers,
                ..
            },
            body,
        ) = res.into_parts();
        log::debug!("AAD response status {:?}", res_status_code);
        log::debug!("AAD response headers{:?}", headers);

        let mut is_json = false;
        for (header_name, header_value) in headers {
            if header_name == Some(hyper::header::CONTENT_TYPE) {
                let value = header_value
                    .to_str()
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
                if value.contains("application/json") {
                    is_json = true;
                }
            }
        }

        let body = hyper::body::to_bytes(body)
            .await
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
        log::debug!("AAD response body {:?}", body);
        println!("AAD response body {:?}", body);

        let res: TResponse = match res_status_code {
            hyper::StatusCode::OK | hyper::StatusCode::CREATED => {
                if !is_json {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "malformed HTTP response",
                    ));
                }
                let res = serde_json::from_slice(&body)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
                res
            }

            res_status_code
                if res_status_code.is_client_error() || res_status_code.is_server_error() =>
            {
                let res: crate::Error = serde_json::from_slice(&body)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;
                return Err(std::io::Error::new(std::io::ErrorKind::Other, res.message));
            }

            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "malformed HTTP response",
                ))
            }
        };
        Ok(res)
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Error {
    #[serde(alias = "Message")]
    pub message: std::borrow::Cow<'static, str>,
}

#[derive(Debug, serde::Serialize)]
pub struct AADRequest<'a> {
    pub grant_type: &'a str,
    pub scope: &'a str,
    pub client_id: &'a str,
    pub external_device_id: &'a str,
}

#[derive(Debug, serde::Deserialize)]
pub struct AADResponse {
    pub access_token: String,
}
