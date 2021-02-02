use anyhow::Result;
use kimed_types::{ClientRequest, GetGlobalHangulStateReply, IndicatorMessage};
use std::{
    fs::File,
    process::{Child, Command, Stdio},
};
use structopt::StructOpt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct ServerContext {
    global_hangul_state: bool,
    indicator_client: Option<Child>,
    window_client: Option<Child>,
}

static CONTEXT: Mutex<ServerContext> = Mutex::const_new(ServerContext {
    global_hangul_state: false,
    indicator_client: None,
    window_client: None,
});

async fn serve_engine(mut stream: UnixStream) -> Result<()> {
    loop {
        let req = kimed_types::async_deserialize_from(&mut stream).await?;

        log::trace!("client req: {:?}", req);

        match req {
            ClientRequest::GetGlobalHangulState => {
                kimed_types::async_serialize_into(
                    &mut stream,
                    GetGlobalHangulStateReply(CONTEXT.lock().await.global_hangul_state),
                )
                .await?;
            }
            ClientRequest::Indicator(IndicatorMessage::UpdateHangulState(state)) => {
                log::trace!("Update hangul: {}", state);
                let mut ctx = CONTEXT.lock().await;

                if ctx.global_hangul_state != state {
                    ctx.global_hangul_state = state;
                    if let Some(indicator_client) =
                        ctx.indicator_client.as_mut().and_then(|c| c.stdin.as_mut())
                    {
                        kimed_types::serialize_into(
                            indicator_client,
                            IndicatorMessage::UpdateHangulState(state),
                        )?;
                    }
                }
            }
            ClientRequest::Window(msg) => {
                let mut ctx = CONTEXT.lock().await;
                if let Some(window_client) =
                    ctx.window_client.as_mut().and_then(|c| c.stdin.as_mut())
                {
                    kimed_types::serialize_into(window_client, msg)?;
                }
            }
        }
    }
}

async fn daemon_main() -> Result<()> {
    {
        let mut ctx = CONTEXT.lock().await;
        let indicator = Command::new("kime-indicator")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .ok();

        let window = Command::new("kime-window")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .ok();
        ctx.indicator_client = indicator;
        ctx.window_client = window;
    }

    let path = std::path::Path::new("/tmp/kimed.sock");

    if path.exists() {
        std::fs::remove_file(path).ok();
    }

    let server = UnixListener::bind(path).unwrap();

    loop {
        let (stream, _addr) = server.accept().await.expect("Accept");
        log::info!("Connect client");
        tokio::spawn(async move {
            if let Err(err) = serve_engine(stream).await {
                log::trace!("Client error: {}", err);
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
