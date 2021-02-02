use x11rb::connection::Connection;
use xim::{x11rb::HasConnection, ServerError, XimConnections};

mod handler;

fn main() -> Result<(), ServerError> {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        println!("-h or --help: show help");
        println!("-v or --version: show version");
        println!("--verbose: more verbose log");
        return Ok(());
    }

    if args.contains(["-v", "--version"]) {
        kime_version::print_version!();
        return Ok(());
    }

    let mut log_level = if cfg!(debug_assertions) {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Info
    };

    if args.contains("--verbose") {
        log_level = log::LevelFilter::Trace;
    }

    simplelog::SimpleLogger::init(log_level, simplelog::ConfigBuilder::new().build()).ok();

    log::info!("Start xim server version: {}", env!("CARGO_PKG_VERSION"));

    let config = kime_engine_cffi::Config::new();

    let (conn, screen_num) = x11rb::rust_connection::RustConnection::connect(None)?;
    let root = conn.setup().roots[screen_num].root;
    let mut server = xim::x11rb::X11rbServer::init(conn, screen_num, "kime")?;
    let mut connections = XimConnections::new();
    let mut handler = self::handler::KimeHandler::new(root, config);

    loop {
        let e = server.conn().wait_for_event()?;
        if !server.filter_event(&e, &mut connections, &mut handler)? {
            match e {
                e => {
                    log::trace!("Unfiltered event: {:?}", e);
                }
            }
        }
    }
}
