use futures::{
    Stream, StreamExt,
    future::ready,
    stream::{BoxStream, SelectAll},
};
#[cfg(unix)]
use std::path::PathBuf;
use std::{
    io::ErrorKind,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader, WriteHalf};
use tokio_stream::wrappers::LinesStream;

#[cfg(unix)]
use tokio::net::UnixListener;
#[cfg(unix)]
use tokio::net::UnixStream;

#[cfg(windows)]
use tokio::net::TcpListener;
#[cfg(windows)]
use tokio::net::TcpStream;

use crate::{FrontendEvent, FrontendRequest, IpcError, IpcListenerCreationError};

pub struct AsyncFrontendListener {
    #[cfg(windows)]
    listener: TcpListener,
    #[cfg(unix)]
    listener: UnixListener,
    #[cfg(unix)]
    socket_path: PathBuf,
    line_streams: SelectAll<BoxStream<'static, Result<FrontendRequest, IpcError>>>,
    #[cfg(unix)]
    tx_streams: Vec<WriteHalf<UnixStream>>,
    #[cfg(windows)]
    tx_streams: Vec<WriteHalf<TcpStream>>,
}

impl AsyncFrontendListener {
    pub async fn new() -> Result<Self, IpcListenerCreationError> {
        #[cfg(unix)]
        return Self::new_at(crate::default_socket_path()?).await;
        #[cfg(windows)]
        Self::new_windows().await
    }

    #[cfg(unix)]
    async fn new_at(socket_path: PathBuf) -> Result<Self, IpcListenerCreationError> {
        let (socket_path, listener) = {
            log::debug!("remove socket: {socket_path:?}");
            if socket_path.exists() {
                // try to connect to see if some other instance
                // of mousehop is already running
                match UnixStream::connect(&socket_path).await {
                    // connected -> mousehop is already running
                    Ok(_) => return Err(IpcListenerCreationError::AlreadyRunning),
                    // mousehop is not running but a socket was left behind
                    Err(e) => {
                        log::debug!("{socket_path:?}: {e} - removing left behind socket");
                        let _ = std::fs::remove_file(&socket_path);
                    }
                }
            }
            let listener = match UnixListener::bind(&socket_path) {
                Ok(ls) => ls,
                // some other mousehop instance has bound the socket in the meantime
                Err(e) if e.kind() == ErrorKind::AddrInUse => {
                    return Err(IpcListenerCreationError::AlreadyRunning);
                }
                Err(e) => return Err(IpcListenerCreationError::Bind(e)),
            };
            (socket_path, listener)
        };
        Ok(Self {
            listener,
            socket_path,
            line_streams: SelectAll::new(),
            tx_streams: vec![],
        })
    }

    #[cfg(windows)]
    async fn new_windows() -> Result<Self, IpcListenerCreationError> {
        let listener = match TcpListener::bind("127.0.0.1:5252").await {
            Ok(ls) => ls,
            // some other mousehop instance has bound the socket in the meantime
            Err(e) if e.kind() == ErrorKind::AddrInUse => {
                return Err(IpcListenerCreationError::AlreadyRunning);
            }
            Err(e) => return Err(IpcListenerCreationError::Bind(e)),
        };

        Ok(Self {
            listener,
            line_streams: SelectAll::new(),
            tx_streams: vec![],
        })
    }

    pub async fn broadcast(&mut self, notify: FrontendEvent) {
        // encode event
        let mut json = serde_json::to_string(&notify).unwrap();
        json.push('\n');

        let mut keep = vec![];
        // TODO do simultaneously
        for tx in self.tx_streams.iter_mut() {
            // write len + payload
            if tx.write(json.as_bytes()).await.is_err() {
                keep.push(false);
                continue;
            }
            keep.push(true);
        }

        // could not find a better solution because async
        let mut keep = keep.into_iter();
        self.tx_streams.retain(|_| keep.next().unwrap());
    }
}

#[cfg(unix)]
impl Drop for AsyncFrontendListener {
    fn drop(&mut self) {
        log::debug!("remove socket: {:?}", self.socket_path);
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

impl Stream for AsyncFrontendListener {
    type Item = Result<FrontendRequest, IpcError>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Poll::Ready(Some(request)) = self.line_streams.poll_next_unpin(cx) {
            return Poll::Ready(Some(request));
        }
        let mut sync = false;
        while let Poll::Ready(Ok((stream, _))) = self.listener.poll_accept(cx) {
            let (rx, tx) = tokio::io::split(stream);
            self.line_streams.push(requests(rx));
            self.tx_streams.push(tx);
            sync = true;
        }
        if sync {
            Poll::Ready(Some(Ok(FrontendRequest::Sync)))
        } else {
            Poll::Pending
        }
    }
}

/// Parse one request per line. The stream ends at the first line that is not
/// a valid request: on Windows the IPC port is plain loopback TCP, so a web
/// page can POST to it, and without this the JSON in the HTTP body would run.
/// The request line (`POST / HTTP/1.1`) always comes first and ends it.
fn requests<R: AsyncRead + Send + 'static>(
    rx: R,
) -> BoxStream<'static, Result<FrontendRequest, IpcError>> {
    LinesStream::new(BufReader::new(rx).lines())
        .map(|line| Ok(serde_json::from_str(&line?)?))
        .scan(false, |failed, request: Result<_, IpcError>| {
            if *failed {
                return ready(None);
            }
            *failed = request.is_err();
            ready(Some(request))
        })
        .boxed()
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Drain requests until the listener goes quiet.
    async fn collect(
        listener: &mut AsyncFrontendListener,
    ) -> Vec<Result<FrontendRequest, IpcError>> {
        let mut out = vec![];
        while let Ok(Some(item)) =
            tokio::time::timeout(Duration::from_millis(200), listener.next()).await
        {
            out.push(item);
        }
        out
    }

    #[tokio::test]
    async fn http_request_body_is_not_executed() {
        let dir = std::env::temp_dir().join(format!("mousehop-ipc-http-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut listener = AsyncFrontendListener::new_at(dir.join("http.sock"))
            .await
            .unwrap();
        let mut stream = UnixStream::connect(dir.join("http.sock")).await.unwrap();
        // What a browser sends for a no-cors text/plain POST to the
        // Windows loopback port: the body line is valid request JSON.
        stream
            .write_all(b"POST / HTTP/1.1\r\nHost: 127.0.0.1:5252\r\n\r\n{\"Enumerate\":[]}\n")
            .await
            .unwrap();
        let requests = collect(&mut listener).await;
        assert!(
            !requests
                .iter()
                .any(|r| matches!(r, Ok(FrontendRequest::Enumerate()))),
            "request in HTTP body was executed: {requests:?}"
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn json_requests_are_accepted() {
        let dir = std::env::temp_dir().join(format!("mousehop-ipc-json-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut listener = AsyncFrontendListener::new_at(dir.join("json.sock"))
            .await
            .unwrap();
        let mut stream = UnixStream::connect(dir.join("json.sock")).await.unwrap();
        stream
            .write_all(b"{\"Enumerate\":[]}\n{\"Enumerate\":[]}\n")
            .await
            .unwrap();
        let requests = collect(&mut listener).await;
        let enumerates = requests
            .iter()
            .filter(|r| matches!(r, Ok(FrontendRequest::Enumerate())))
            .count();
        assert_eq!(enumerates, 2, "{requests:?}");
        let _ = std::fs::remove_dir_all(dir);
    }
}
