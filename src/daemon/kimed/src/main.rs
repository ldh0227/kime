use anyhow::Result;
use kimed_types::{
    ClientHello, ClientRequest, GetGlobalHangulStateReply, IndicatorMessage, WindowMessage,
};
use std::fs::File;
use structopt::StructOpt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct ServerContext {
    global_hangul_state: bool,
    indicator_client: Option<UnixStream>,
    window_client: Option<UnixStream>,
}

static CONTEXT: Mutex<ServerContext> = Mutex::const_new(ServerContext {
    global_hangul_state: false,
    indicator_client: None,
    window_client: None,
});

async fn serve_engine(mut stream: UnixStream) -> Result<()> {
    loop {
        match kimed_types::async_deserialize_from(&mut stream).await? {
            ClientRequest::GetGlobalHangulState => {
                kimed_types::async_serialize_into(
                    &mut stream,
                    GetGlobalHangulStateReply(CONTEXT.lock().await.global_hangul_state),
                )
                .await?;
            }
            ClientRequest::UpdateHangulState(state) => {
                let mut ctx = CONTEXT.lock().await;
                if ctx.global_hangul_state != state {
                    ctx.global_hangul_state = state;
                    if let Some(indicator_client) = ctx.indicator_client.as_mut() {
                        kimed_types::async_serialize_into(
                            indicator_client,
                            IndicatorMessage::UpdateHangulState(state),
                        )
                        .await?;
                    }
                }
            }
            ClientRequest::SpawnPreeditWindow { x, y, ch } => {
                let mut ctx = CONTEXT.lock().await;
                if let Some(window_client) = ctx.window_client.as_mut() {
                    kimed_types::async_serialize_into(
                        window_client,
                        WindowMessage::SpawnPreeditWindow { x, y, ch },
                    )
                    .await?;
                }
            }
        }
    }
}

async fn daemon_main() -> Result<()> {
    let path = std::path::Path::new("/tmp/kimed.sock");

    if path.exists() {
        std::fs::remove_file(path).ok();
    }

    let server = UnixListener::bind(path).unwrap();

    loop {
        let (mut stream, _addr) = server.accept().await.expect("Accept");
        log::info!("Connect client");
        tokio::spawn(async move {
            match kimed_types::async_deserialize_from(&mut stream).await {
                Ok(hello) => match hello {
                    ClientHello::Window => {
                        log::info!("Register window client");
                        CONTEXT.lock().await.window_client = Some(stream);
                    }
                    ClientHello::Indicator => {
                        log::info!("Register indicator client");
                        CONTEXT.lock().await.indicator_client = Some(stream);
                    }
                    ClientHello::Engine => {
                        log::info!("Register engine client");
                        if let Err(err) = serve_engine(stream).await {
                            log::error!("Client error: {}", err);
                        }
                    }
                },
                Err(err) => {
                    log::error!("Hello failed: {}", err);
                }
            }
        });
    }
}

#[derive(StructOpt)]
#[structopt(about = "kime daemon")]
struct Opts {
    #[structopt(long, short, help = "Show daemon version")]
    version: bool,
    #[structopt(long, help = "Log more messages")]
    verbose: bool,
    #[structopt(long, help = "Run as normal process")]
    not_daemon: bool,
}

fn main() {
    let opt = Opts::from_args();

    if opt.version {
        kime_version::print_version!();
        return;
    }

    if !opt.not_daemon {
        let daemonize = daemonize::Daemonize::new()
            .pid_file("/tmp/kimed.pid")
            .working_directory("/tmp")
            .stdout(File::create("/tmp/kimed.out").unwrap())
            .stderr(File::create("/tmp/kimed.err").unwrap());

        if let Err(err) = daemonize.start() {
            eprintln!("Daemonize Error: {}", err);
            return;
        }
    }

    simplelog::SimpleLogger::init(
        if cfg!(debug_assertions) || opt.verbose {
            log::LevelFilter::Trace
        } else {
            log::LevelFilter::Info
        },
        simplelog::ConfigBuilder::new().build(),
    )
    .ok();
    log::info!("Start daemon");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .expect("Make tokio runtime");

    match rt.block_on(daemon_main()) {
        Ok(_) => {}
        Err(err) => {
            log::error!("Error: {}", err);
        }
    }
}
