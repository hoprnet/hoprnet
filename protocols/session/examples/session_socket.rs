use clap::{Parser, Subcommand};
use hopr_network_types::udp::{ConnectedUdpStream, ForeignDataMode, UdpStreamBuilder};
use hopr_protocol_session::{AcknowledgementState, SessionSocketExt};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    select, signal,
};
use tokio_util::compat::TokioAsyncReadCompatExt;

const BUFFER_SIZE: usize = 4096;
const SESSION_MTU: usize = 1458;

#[derive(Parser)]
#[command(name = "session-sock")]
#[command(about = "Session protocol over UDP")]
#[command(version = "1.0")]
struct Cli {
    /// If specified, a reliable Session socket is created (allowing acknowledgements and frame retransmissions).
    #[clap(
        global = true,
        value_name = "SESSION_SOCK_RELIABLE",
        long,
        short = 'r',
        default_value = "false"
    )]
    reliable: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a UDP server with Session protocol
    Server {
        /// Address and port to bind to (e.g., 127.0.0.1:8080)
        #[arg(
            value_name = "SESSION_SOCK_BIND_ADDRESS",
            long,
            short = 'a',
            default_value = "0.0.0.0:0"
        )]
        address: String,
    },
    /// Connect as a UDP client with Session protocol
    Client {
        /// Server address and port to connect to (e.g., 127.0.0.1:8080)
        #[arg(
            value_name = "SESSION_SOCK_SERVER_ADDRESS",
            long,
            short = 'a',
            default_value = "127.0.0.1:8080"
        )]
        address: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { address } => {
            println!("Socket bound to {address}");
            let socket = create_udp_socket(&address, None)?;
            if cli.reliable {
                let session_socket = socket.compat().reliable_session::<SESSION_MTU>(
                    AcknowledgementState::new("session_sock", Default::default()),
                    Default::default(),
                )?;

                handle_connection(session_socket).await?;
            } else {
                let session_socket = socket
                    .compat()
                    .unreliable_session::<SESSION_MTU>("session_sock", Default::default())?;

                handle_connection(session_socket).await?;
            }
        }
        Commands::Client { address } => {
            println!("Connected to {address}");
            let socket = create_udp_socket("0.0.0.0:0", Some(&address))?;
            if cli.reliable {
                let session_socket = socket.compat().reliable_session::<SESSION_MTU>(
                    AcknowledgementState::new("session_sock", Default::default()),
                    Default::default(),
                )?;

                handle_connection(session_socket).await?;
            } else {
                let session_socket = socket
                    .compat()
                    .unreliable_session::<SESSION_MTU>("session_sock", Default::default())?;

                handle_connection(session_socket).await?;
            }
        }
    }

    Ok(())
}

fn create_udp_socket(
    bind_addr: &str,
    counterparty: Option<&str>,
) -> Result<ConnectedUdpStream, Box<dyn std::error::Error>> {
    let mut builder = UdpStreamBuilder::default()
        .with_buffer_size(BUFFER_SIZE)
        .with_foreign_data_mode(ForeignDataMode::Discard);

    if let Some(addr) = counterparty {
        builder = builder.with_counterparty(addr.parse()?);
    }

    Ok(builder.build(bind_addr)?)
}

async fn handle_connection<S>(socket: S) -> Result<(), Box<dyn std::error::Error>>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let mut socket = BufReader::new(socket);
    let mut stdin = BufReader::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();

    let mut tcp_buf = [0u8; BUFFER_SIZE];
    let mut stdin_buf = [0u8; BUFFER_SIZE];

    loop {
        select! {
            // Read from TCP socket in chunks
            result = socket.read(&mut tcp_buf) => {
                match result {
                    Ok(0) => {
                        eprintln!("Connection closed");
                        break;
                    }
                    Ok(n) => {
                        stdout.write_all(&tcp_buf[..n]).await?;
                        stdout.flush().await?;
                    }
                    Err(e) => {
                        eprintln!("Connection error: {}", e);
                        break;
                    }
                }
            }
            // Read from stdin in chunks
            result = stdin.read(&mut stdin_buf) => {
                match result {
                    Ok(0) => {
                        eprintln!("Stdin closed");
                        break;
                    }
                    Ok(n) => {
                        socket.write_all(&stdin_buf[..n]).await?;
                        socket.flush().await?;
                    }
                    Err(e) => {
                        eprintln!("Stdin error: {}", e);
                        break;
                    }
                }
            }
            // Handle Ctrl+C
            _ = signal::ctrl_c() => {
                println!("\nReceived Ctrl+C, closing TCP connection...");
                // The socket will be automatically closed when it goes out of scope
                break;
            }

        }
    }

    Ok(())
}
