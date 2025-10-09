use super::request::{CnRequest, Method};
use cncommon::profile::cnprofile::{CnProfile, Profiler};
use futures::io::{AsyncBufRead, AsyncBufReadExt};
/// Copyright (C) 2024-present WD Studios L.L.C. & CassandraNet Contributors.
use std::collections::HashMap;

impl TryFrom<&str> for Method {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "GET" => Ok(Method::Get),
            _ => Err(anyhow::anyhow!("unsupported method: {value}")),
        }
    }
}

// TODO: im assuming the returned CnRequest's id is currently unset
pub async fn parse_request(mut stream: impl AsyncBufRead + Unpin) -> anyhow::Result<CnRequest> {
    CnProfile::start_event("parse_request");
    let mut line_buffer = String::new();

    stream.read_line(&mut line_buffer).await?;

    let mut parts = line_buffer.split_whitespace();

    let method: Method = parts
        .next()
        .ok_or(anyhow::anyhow!("missing method"))
        .and_then(TryInto::try_into)?;

    let path: String = parts
        .next()
        .ok_or(anyhow::anyhow!("missing path"))
        .map(Into::into)?;

    let mut headers = HashMap::new();

    loop {
        line_buffer.clear();
        stream.read_line(&mut line_buffer).await?;

        if line_buffer.is_empty() || line_buffer == "\n" || line_buffer == "\r\n" {
            break;
        }

        let mut comps = line_buffer.split(":");
        let key = comps.next().ok_or(anyhow::anyhow!("missing header name"))?;
        let value = comps
            .next()
            .ok_or(anyhow::anyhow!("missing header value"))?
            .trim();

        headers.insert(key.to_string(), value.to_string());
    }

    Ok(CnRequest::new(0, method, Some(path), headers))
}
