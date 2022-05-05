use bytes::Bytes;
use bytes::{buf::BufMut, BytesMut};
use futures::task::{Context, Poll};
use futures::StreamExt;
use pyo3::iter::IterNextOutput;
use pyo3::pyclass::IterANextOutput;
use pyo3::types::PyString;
use pyo3::{exceptions, PyErr};
use pyo3::{pyclass, pymethods, Py, PyAny, PyObject, PyRef, PyRefMut, PyResult, Python};
use serde::de::DeserializeOwned;
use std::convert::TryFrom;
use std::io::Cursor;
use std::lazy::SyncOnceCell;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::OwnedTrustAnchor;
use tokio_rustls::rustls::{ClientConfig, RootCertStore, ServerName};
use tokio_rustls::TlsConnector;
use tokio_util::codec::Framed;
use tokio_util::codec::{Decoder, Encoder};

use super::error::Error as BfApiError;
use crate::ids::ConnectionID;

use super::stream_types::{AuthMessage, ConnectionMessage, Request, StatusCode, StatusMessage};

static CONFIG: SyncOnceCell<TlsConnector> = SyncOnceCell::new();
const STREAM_CAPACITY: usize = 2000000;

#[pyclass]
pub struct Connection {
    id: ConnectionID,
    // conn: TlsStream<TcpStream>,
    stream: Framed<TlsStream<TcpStream>, NewLineCodec>,
}

impl Connection {
    /// connect to the stream
    /// unlike bfapi::Connection, here we dont subscribe to any markets as this Connection type is designed to be used
    /// as a proxy for the user we let the user pass the expected value through.
    pub async fn new(app_key: &str, auth: &str) -> PyResult<Connection> {
        let config = CONFIG.get_or_init(|| {
            let mut root_store = RootCertStore::empty();
            root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(
                |ta| {
                    OwnedTrustAnchor::from_subject_spki_name_constraints(
                        ta.subject,
                        ta.spki,
                        ta.name_constraints,
                    )
                },
            ));

            let config = ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(root_store)
                .with_no_client_auth();

            TlsConnector::from(Arc::new(config))
        });

        let dnsname = ServerName::try_from("stream-api.betfair.com").unwrap();
        let tcp_stream = TcpStream::connect("stream-api.betfair.com:443").await?;
        let mut tls_stream = config.connect(dnsname, tcp_stream).await?;

        // read buffer, we might read more then 1 message here if 2 are ready. Keep track of head value
        // which is any extra value we read from the last read, we move it to the front of the buffer so the
        // next read can be appended to it
        let mut rbuff: [u8; 256] = [0; 256];
        let mut head: usize = 0;

        // listen for the connection message
        let con_id = {
            let (cm, h) = Connection::read_response::<ConnectionMessage>(
                &mut tls_stream,
                head,
                &mut rbuff[..],
            )
            .await?;
            head = h;

            cm.connection_id
        };

        // send auth message and listen for the status request
        {
            // write buffer, we will always start from 0 and write until an n - so no need to track
            let mut wbuff: [u8; 1024] = [0; 1024];
            let auth_req = Request::Authentication(AuthMessage {
                id: None,
                session: auth,
                app_key,
            });
            Connection::write_request(&mut tls_stream, &mut wbuff, &auth_req).await?;

            let (msg, _h) =
                Connection::read_response::<StatusMessage>(&mut tls_stream, head, &mut rbuff[..])
                    .await?;

            match (msg.status_code, msg.error_code, msg.error_message) {
                (StatusCode::Failure, Some(code), Some(msg)) => {
                    return Err(BfApiError::Stream { code, msg }.into());
                }
                (StatusCode::Failure, _, _) => {
                    return Err(BfApiError::Unexpected.into());
                }
                _ => {}
            }
        }

        Ok(Connection {
            id: con_id,
            stream: Framed::with_capacity(tls_stream, NewLineCodec::new(), STREAM_CAPACITY),
        })
    }

    async fn read_response<T: DeserializeOwned>(
        stream: &mut TlsStream<TcpStream>,
        mut head: usize,
        buff: &mut [u8],
    ) -> Result<(T, usize), BfApiError> {
        loop {
            match stream.read(&mut buff[head..]).await {
                Err(err) => return Err(BfApiError::IO(err)),
                Ok(0) => return Err(BfApiError::EmptyMessage),
                Ok(read) => match buff[head..head + read].iter().position(|&r| r == b'\n') {
                    Some(mut i) => {
                        i += head;

                        let msg: T = serde_json::from_slice(&buff[head..i])?;

                        buff.copy_within(i + 1..head + read, 0); // +1 to overide the \n
                        return Ok((msg, head + read - i - 1));
                    }
                    None => head += read,
                },
            }
        }
    }

    async fn write_request<'a>(
        stream: &mut TlsStream<TcpStream>,
        buff: &mut [u8],
        req: &Request<'a>,
    ) -> Result<usize, BfApiError> {
        let (mut n, buff) = {
            let mut cursor = Cursor::new(buff);
            serde_json::to_writer(&mut cursor, req)?;
            (cursor.position() as usize, cursor.into_inner())
        };

        // add new line to end of msg
        buff[n] = b'\n';
        n += 1;

        stream.write_all(&buff[..n]).await?;

        Ok(n)
    }

    pub fn id(&self) -> ConnectionID {
        self.id
    }

    pub async fn shutdown(self) -> Result<(), BfApiError> {
        let mut tcp_stream = self.stream.into_inner();

        tcp_stream.shutdown().await.map_err(|err| err.into())
    }
}

#[derive(Debug, Eq, Clone, PartialEq, Ord, PartialOrd, Hash)]
pub struct NewLineCodec {
    /// this just helps us not scan through the same part of the buffer looking for the \n repeatedly
    next_index: usize,
}

impl NewLineCodec {
    pub fn new() -> Self {
        Self { next_index: 0 }
    }
}

impl Decoder for NewLineCodec {
    type Item = Bytes;
    type Error = BfApiError;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline_offset = buf[self.next_index..buf.len()]
            .iter()
            .position(|b| *b == b'\n');

        match newline_offset {
            Some(offset) => {
                let newline_index = offset + self.next_index;
                self.next_index = 0;

                let line = buf.split_to(newline_index + 1).freeze();
                Ok(Some(line))
            }
            None => {
                self.next_index = buf.len();
                Ok(None)
            }
        }
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        Ok(self.decode(buf)?)
    }
}

impl Encoder<Bytes> for NewLineCodec {
    type Error = BfApiError;

    fn encode(&mut self, msg: Bytes, buf: &mut BytesMut) -> Result<(), Self::Error> {
        buf.put(msg);
        buf.put_u8(b'\n');
        Ok(())
    }
}

#[pymethods]
impl Connection {
    fn __aiter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __anext__(slf: PyRef<Self>) -> IterANextOutput<PyRef<Self>, ()> {
        IterANextOutput::Yield(slf)
    }

    fn __await__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__<'py>(&mut self, py1: Python<'py>) -> Option<&'py PyAny> {
        pyo3_asyncio::tokio::future_into_py(py1, async {
            let a = self.stream.next().await.map(|bs| {
                bs.map(|bs| {
                    Python::with_gil(|py| {
                        let s: Py<PyString> =
                            PyString::new(py, simdutf8::basic::from_utf8(&bs).unwrap()).into();
                        s
                    })
                })
            });

            match a {
                Some(v) => v.map_err(|err| PyErr::new::<exceptions::PyRuntimeError, _>("derp")),
                None => Python::with_gil(|py| Ok(PyString::new(py, "derp").into())),
            }
        })
        .ok()
    }
}
