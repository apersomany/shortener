use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::{net::Ipv4Addr, sync::atomic::AtomicU64};

use anyhow::{Error, Result};
use dashmap::DashMap;
use hyper::{
    Request, Response, body::Incoming, header::LOCATION,
    server::conn::http1::Builder as ConnectionBuilder, service::service_fn,
};
use hyper_util::rt::TokioIo;
use serde::{Deserialize, Serialize};
use tokio::{net::TcpListener, runtime::Builder as RuntimeBuilder, spawn, time::sleep};
use tracing::info;

#[derive(Serialize, Deserialize)]
struct Data {
    location: String,
    visitors: AtomicU64,
}

impl Data {
    pub fn new(location: String) -> Arc<Self> {
        Arc::new(Data {
            location,
            visitors: AtomicU64::new(0),
        })
    }
}

type Map = Arc<DashMap<String, Arc<Data>>>;

macro_rules! respond {
    (302, $location:expr) => {
        return Response::builder()
            .status(302)
            .header(LOCATION, $location)
            .body(String::new())
            .map_err(Error::from)
    };
    ($status:expr, $body:expr) => {
        return Response::builder()
            .status($status)
            .body($body.to_string())
            .map_err(Error::from)
    };
    ($status:expr) => {
        return Response::builder()
            .status($status)
            .body(String::new())
            .map_err(Error::from)
    };
}

async fn handle_request(request: Request<Incoming>, map: Map) -> Result<Response<String>> {
    let mut path = request.uri().path().splitn(4, '/').skip(1);
    let Some(op) = path.next() else {
        respond!(404);
    };
    match op {
        "i" => {
            let Some(slug) = path.next() else {
                respond!(400, "slug not provided");
            };
            let Some(location) = path.next() else {
                respond!(400, "location not provided");
            };
            map.insert(slug.to_string(), Data::new(location.to_string()));
            respond!(200);
        },
        "v" => {
            let Some(slug) = path.next() else {
                respond!(400, "slug not provided");
            };
            if let Some(data) = map.get(slug).as_deref().cloned() {
                respond!(200, data.visitors.load(Ordering::Relaxed).to_string());
            } else {
                respond!(404);
            }
        }
        slug => {
            if let Some(data) = map.get(slug).as_deref().cloned() {
                data.visitors.fetch_add(1, Ordering::Relaxed);
                respond!(302, &data.location);
            } else {
                respond!(404);
            }
        }
    }
}

async fn worker(listener: Arc<TcpListener>, map: Map) -> Result<()> {
    let mut http = ConnectionBuilder::new();
    http.keep_alive(true);
    loop {
        let (stream, _) = listener.accept().await?;
        let connection = http.serve_connection(
            TokioIo::new(stream),
            service_fn(|request| handle_request(request, map.clone())),
        );
        drop(connection.await);
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build runtime");
    runtime.block_on(async {
        let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 80))
            .await
            .expect("failed to bind listener");
        let listener = Arc::new(listener);
        let map = Arc::new(DashMap::new());
        info!("server started on 0.0.0.0:80");
        for _ in 0..1024 {
            spawn(worker(listener.clone(), map.clone()));
        }
        sleep(Duration::MAX).await;
    })
}
