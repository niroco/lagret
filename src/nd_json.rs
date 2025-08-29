use axum::response::IntoResponse;
use bytes::Bytes;

use std::io::{self, Write};

pub struct NdJson<T>(pub Vec<T>);

impl<T> From<NdJson<T>> for Bytes
where
    T: serde::Serialize,
{
    fn from(nd_json: NdJson<T>) -> Self {
        let mut buf = io::Cursor::new(Vec::new());

        for e in nd_json.0 {
            serde_json::to_writer(&mut buf, &e).expect("serializing NdJson");
            let n = buf.write(b"\n").expect("writing line feed");

            assert_eq!(1, n);
        }

        Bytes::from(buf.into_inner())
    }
}

impl<T> IntoResponse for NdJson<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> axum::response::Response {
        Bytes::from(self).into_response()
    }
}
