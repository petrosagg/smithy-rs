/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

//! Presigned request types and configuration.

/// Presigning config and builder
pub mod config {
    use std::fmt;
    use std::time::{Duration, SystemTime};

    const ONE_WEEK: Duration = Duration::from_secs(604800);

    /// Presigning config values required for creating a presigned request.
    #[non_exhaustive]
    #[derive(Debug, Clone)]
    pub struct PresigningConfig {
        start_time: SystemTime,
        expires_in: Duration,
    }

    impl PresigningConfig {
        /// Creates a `PresigningConfig` with the given `expires_in` duration.
        ///
        /// The `expires_in` duration is the total amount of time the presigned request should
        /// be valid for. Other config values are defaulted.
        ///
        /// Credential expiration time takes priority over the `expires_in` value.
        /// If the credentials used to sign the request expire before the presigned request is
        /// set to expire, then the presigned request will become invalid.
        pub fn expires_in(expires_in: Duration) -> Result<PresigningConfig, Error> {
            Self::builder().expires_in(expires_in).build()
        }

        /// Creates a new builder for creating a `PresigningConfig`.
        pub fn builder() -> Builder {
            Builder::default()
        }

        /// Returns the amount of time the presigned request should be valid for.
        pub fn expires(&self) -> Duration {
            self.expires_in
        }

        /// Returns the start time. The presigned request will be valid between this and the end
        /// time produced by adding the `expires()` value to it.
        pub fn start_time(&self) -> SystemTime {
            self.start_time
        }
    }

    /// `PresigningConfig` build errors.
    #[non_exhaustive]
    #[derive(Debug)]
    pub enum Error {
        /// Presigned requests cannot be valid for longer than one week.
        ExpiresInDurationTooLong,

        /// The `PresigningConfig` builder requires a value for `expires_in`.
        ExpiresInRequired,
    }

    impl std::error::Error for Error {}

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Error::ExpiresInDurationTooLong => {
                    write!(f, "`expires_in` must be no longer than one week")
                }
                Error::ExpiresInRequired => write!(f, "`expires_in` is required"),
            }
        }
    }

    /// Builder used to create `PresigningConfig`.
    #[non_exhaustive]
    #[derive(Default, Debug)]
    pub struct Builder {
        start_time: Option<SystemTime>,
        expires_in: Option<Duration>,
    }

    impl Builder {
        /// Sets the start time for the presigned request.
        ///
        /// The request will start to be valid at this time, and will cease to be valid after
        /// the end time, which can be determined by adding the `expires_in` duration to this
        /// start time. If not specified, this will default to the current time.
        ///
        /// Optional.
        pub fn start_time(mut self, start_time: SystemTime) -> Self {
            self.set_start_time(Some(start_time));
            self
        }

        /// Sets the start time for the presigned request.
        ///
        /// The request will start to be valid at this time, and will cease to be valid after
        /// the end time, which can be determined by adding the `expires_in` duration to this
        /// start time. If not specified, this will default to the current time.
        ///
        /// Optional.
        pub fn set_start_time(&mut self, start_time: Option<SystemTime>) {
            self.start_time = start_time;
        }

        /// Sets how long the request should be valid after the `start_time` (which defaults
        /// to the current time).
        ///
        /// Credential expiration time takes priority over the `expires_in` value.
        /// If the credentials used to sign the request expire before the presigned request is
        /// set to expire, then the presigned request will become invalid.
        ///
        /// Required.
        pub fn expires_in(mut self, expires_in: Duration) -> Self {
            self.set_expires_in(Some(expires_in));
            self
        }

        /// Sets how long the request should be valid after the `start_time` (which defaults
        /// to the current time).
        ///
        /// Credential expiration time takes priority over the `expires_in` value.
        /// If the credentials used to sign the request expire before the presigned request is
        /// set to expire, then the presigned request will become invalid.
        ///
        /// Required.
        pub fn set_expires_in(&mut self, expires_in: Option<Duration>) {
            self.expires_in = expires_in;
        }

        /// Builds the `PresigningConfig`. This will error if `expires_in` is not
        /// given, or if it's longer than one week.
        pub fn build(self) -> Result<PresigningConfig, Error> {
            let expires_in = self.expires_in.ok_or(Error::ExpiresInRequired)?;
            if expires_in > ONE_WEEK {
                return Err(Error::ExpiresInDurationTooLong);
            }
            Ok(PresigningConfig {
                start_time: self.start_time.unwrap_or_else(SystemTime::now),
                expires_in,
            })
        }
    }
}

/// Presigned request
pub mod request {
    use std::fmt::{Debug, Formatter};

    /// Represents a presigned request. This only includes the HTTP request method, URI, and headers.
    ///
    /// **This struct has conversion convenience functions:**
    ///
    /// - [`PresignedRequest::to_http_request<B>`][Self::to_http_request] returns an [`http::Request<B>`](https://docs.rs/http/0.2.6/http/request/struct.Request.html)
    /// - [`PresignedRequest::into`](#impl-From<PresignedRequest>) returns an [`http::request::Builder`](https://docs.rs/http/0.2.6/http/request/struct.Builder.html)
    #[non_exhaustive]
    pub struct PresignedRequest(http::Request<()>);

    impl PresignedRequest {
        pub(crate) fn new(inner: http::Request<()>) -> Self {
            Self(inner)
        }

        /// Returns the HTTP request method.
        pub fn method(&self) -> &http::Method {
            self.0.method()
        }

        /// Returns the HTTP request URI.
        pub fn uri(&self) -> &http::Uri {
            self.0.uri()
        }

        /// Returns any HTTP headers that need to go along with the request, except for `Host`,
        /// which should be sent based on the endpoint in the URI by the HTTP client rather than
        /// added directly.
        pub fn headers(&self) -> &http::HeaderMap<http::HeaderValue> {
            self.0.headers()
        }

        /// Given a body, convert this `PresignedRequest` into an `http::Request`
        pub fn to_http_request<B>(self, body: B) -> Result<http::Request<B>, http::Error> {
            let builder: http::request::Builder = self.into();

            builder.body(body)
        }
    }

    impl Debug for PresignedRequest {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("PresignedRequest")
                .field("method", self.method())
                .field("uri", self.uri())
                .field("headers", self.headers())
                .finish()
        }
    }

    impl From<PresignedRequest> for http::request::Builder {
        fn from(req: PresignedRequest) -> Self {
            let mut builder = http::request::Builder::new()
                .uri(req.uri())
                .method(req.method());

            if let Some(headers) = builder.headers_mut() {
                *headers = req.headers().clone();
            }

            builder
        }
    }
}

/// Tower middleware service for creating presigned requests
#[allow(dead_code)]
pub(crate) mod service {
    use crate::presigning::request::PresignedRequest;
    use aws_smithy_http::operation;
    use http::header::USER_AGENT;
    use std::future::{ready, Ready};
    use std::marker::PhantomData;
    use std::task::{Context, Poll};

    /// Tower [`Service`](tower::Service) for generated a [`PresignedRequest`] from the AWS middleware.
    #[derive(Default, Debug)]
    #[non_exhaustive]
    pub(crate) struct PresignedRequestService<E> {
        _phantom: PhantomData<E>,
    }

    // Required because of the derive Clone on MapRequestService.
    // Manually implemented to avoid requiring errors to implement Clone.
    impl<E> Clone for PresignedRequestService<E> {
        fn clone(&self) -> Self {
            Self {
                _phantom: Default::default(),
            }
        }
    }

    impl<E> PresignedRequestService<E> {
        /// Creates a new `PresignedRequestService`
        pub(crate) fn new() -> Self {
            Self {
                _phantom: Default::default(),
            }
        }
    }

    impl<E> tower::Service<operation::Request> for PresignedRequestService<E> {
        type Response = PresignedRequest;
        type Error = E;
        type Future = Ready<Result<PresignedRequest, E>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: operation::Request) -> Self::Future {
            let (mut req, _) = req.into_parts();

            // Remove user agent headers since the request will not be executed by the AWS Rust SDK.
            req.headers_mut().remove(USER_AGENT);
            req.headers_mut().remove("X-Amz-User-Agent");

            ready(Ok(PresignedRequest::new(req.map(|_| ()))))
        }
    }
}
