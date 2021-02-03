// SPDX-License-Identifier: Apache-2.0

use structopt::StructOpt;
use tokio::net::{TcpListener, UnixListener};
use tokio_stream::wrappers::{TcpListenerStream, UnixListenerStream};
use warp::Filter;

#[derive(Debug)]
enum Listener {
    Unix(std::os::unix::net::UnixListener),
    Tcp(std::net::TcpListener),
}

impl std::str::FromStr for Listener {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use nix::sys::socket::{getsockname, SockAddr};
        use std::io::ErrorKind;
        use std::net::TcpListener as Tcp;
        use std::os::unix::io::{FromRawFd, RawFd};
        use std::os::unix::net::UnixListener as Unix;

        if let Ok(fd) = RawFd::from_str(s) {
            return match getsockname(fd).map_err(|_| ErrorKind::InvalidInput)? {
                SockAddr::Unix(..) => Ok(Listener::Unix(unsafe { Unix::from_raw_fd(fd) })),
                SockAddr::Inet(..) => Ok(Listener::Tcp(unsafe { Tcp::from_raw_fd(fd) })),
                _ => Err(ErrorKind::InvalidInput.into()),
            };
        }

        Ok(match s.chars().next() {
            Some('/') => Listener::Unix(Unix::bind(s)?),
            _ => Listener::Tcp(Tcp::bind(s)?),
        })
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "contractmgr", about = "Manages contracts for keepmgr.")]
struct Options {
    /// The listening socket address or fd
    listen: Listener,
}

async fn serve<I>(incoming: I) -> tokio::io::Result<()>
where
    I: futures_core::stream::TryStream + Send,
    I::Ok: tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + 'static + Unpin,
    I::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    warp::serve(hello).run_incoming(incoming).await;
    Ok(())
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    match Options::from_args().listen {
        Listener::Unix(socket) => {
            let listen = UnixListener::from_std(socket)?;
            let stream = UnixListenerStream::new(listen);
            serve(stream).await
        }

        Listener::Tcp(socket) => {
            let listen = TcpListener::from_std(socket)?;
            let stream = TcpListenerStream::new(listen);
            serve(stream).await
        }
    }
}
