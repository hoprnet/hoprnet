use fast_socks5::client::{Config as ClientConfig, Socks5Stream};
use fast_socks5::Result;
use hopr_socks_server::cli::AuthMode;
use hopr_socks_server::SocksServer;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

async fn get_client(
    bind_address: String,
    target_host: String,
    target_port: u16,
    auth: AuthMode,
) -> Result<Socks5Stream<TcpStream>> {
    let config = ClientConfig::default();

    // Creating a SOCKS stream to the target address through the socks server
    let stream = match auth {
        AuthMode::NoAuth => match Socks5Stream::connect(bind_address.clone(), target_host, target_port, config).await {
            Ok(stream) => stream,
            Err(err) => {
                eprintln!("Failed to connect to target address: {}", err);
                return Err(err);
            }
        },
        AuthMode::Password { username, password } => {
            match Socks5Stream::connect_with_password(
                bind_address.clone(),
                target_host,
                target_port,
                username,
                password,
                config,
            )
            .await
            {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Failed to connect to target address: {}", err);
                    return Err(err);
                }
            }
        }
    };

    Ok(stream)
}

async fn http_request<T: AsyncRead + AsyncWrite + Unpin>(stream: &mut T, domain: String) -> Result<[u8; 1024]> {
    // construct our request, with a dynamic domain
    let mut headers = vec![];
    headers.extend_from_slice("GET / HTTP/1.1\n".as_bytes());
    headers.extend_from_slice(format!("Host: {domain}\n").as_bytes());
    headers.extend_from_slice("User-Agent: hopr-socks/0.1.0\n".as_bytes());
    headers.extend_from_slice("Accept: */*\n\n".as_bytes());

    // flush headers
    stream.write_all(&headers).await.expect("Can't write HTTP Headers");

    let mut result: [u8; 1024] = [0u8; 1024];
    stream.read(&mut result).await.expect("Can't read HTTP Response");

    Ok(result)
}

#[tokio::test]
async fn test_connect_client() {
    let bind_address = "127.0.0.1:1331";
    let host_address = "hoprnet.org";

    let server_proc = tokio::spawn(async move {
        SocksServer::new(String::from(bind_address), 10, AuthMode::NoAuth)
            .await
            .expect("Failed to create a SOCKS5 server")
            .run()
            .await
            .expect("SOCKS5 server could not be run");
    });

    sleep(Duration::from_millis(200)).await;

    // Open TCP connection through proxy (incorrect bind address)
    let result = get_client(
        String::from(bind_address),
        String::from("127.0.0.2:1331"),
        80,
        AuthMode::NoAuth,
    )
    .await;
    assert!(result.is_err());

    // Open TCP connection through proxy (correct bind address)
    get_client(
        String::from(bind_address),
        String::from(host_address),
        80,
        AuthMode::NoAuth,
    )
    .await
    .expect("Failed to create a SOCKS5 client");

    server_proc.abort();
    sleep(Duration::from_millis(200)).await;
}

#[tokio::test]
async fn test_http_request_through_socks_proxy() {
    let bind_address = "127.0.0.1:1332";
    let host_address = "hoprnet.org";

    let server_proc = tokio::spawn(async move {
        SocksServer::new(String::from(bind_address), 10, AuthMode::NoAuth)
            .await
            .expect("Failed to create a SOCKS5 server")
            .run()
            .await
            .expect("SOCKS5 server could not be run");
    });

    sleep(Duration::from_millis(200)).await;

    // Open TCP connection to existing endpoint
    let mut client = get_client(
        String::from(bind_address),
        String::from(host_address),
        80,
        AuthMode::NoAuth,
    )
    .await
    .expect("Failed to create a SOCKS5 client");

    let results = http_request(&mut client, String::from(host_address))
        .await
        .expect("Failed to send HTTP request");

    assert!(results.starts_with(b"HTTP/1.1"));

    server_proc.abort();
    sleep(Duration::from_millis(200)).await;
}

#[tokio::test]
async fn test_multiple_clients() {
    let bind_address = "127.0.0.1:1333";
    let host_address = "hoprnet.org";

    let server_proc = tokio::spawn(async move {
        SocksServer::new(String::from(bind_address), 10, AuthMode::NoAuth)
            .await
            .expect("Failed to create a SOCKS5 server")
            .run()
            .await
            .expect("SOCKS5 server could not be run");
    });

    sleep(Duration::from_millis(200)).await;

    // Create a vector of Socks5Stream<TcpStream> clients
    let mut clients = Vec::new();
    for _ in 0..20 {
        clients.push(
            get_client(
                String::from(bind_address),
                String::from(host_address),
                80,
                AuthMode::NoAuth,
            )
            .await
            .expect("Failed to create a SOCKS5 client"),
        );
    }

    // Send HTTP request to each client
    for mut client in clients {
        let results = http_request(&mut client, String::from(host_address))
            .await
            .expect("Failed to send HTTP request");

        assert!(results.starts_with(b"HTTP/1.1"));
    }

    server_proc.abort();
    sleep(Duration::from_millis(200)).await;
}

#[tokio::test]
async fn test_connect_client_with_auth() {
    let bind_address = "127.0.0.1:1334";
    let host_address = "hoprnet.org";
    let auth_username = "admin";
    let auth_password = "opensesame";

    let server_proc = tokio::spawn(async move {
        SocksServer::new(
            String::from(bind_address),
            10,
            AuthMode::Password {
                username: String::from(auth_username),
                password: String::from(auth_password),
            },
        )
        .await
        .expect("Failed to create a SOCKS5 server")
        .run()
        .await
        .expect("SOCKS5 server could not be run");
    });

    sleep(Duration::from_millis(200)).await;

    // Open TCP connection to endpoint without authentification
    let client = get_client(
        String::from(bind_address),
        String::from(host_address),
        80,
        AuthMode::NoAuth,
    )
    .await;
    assert!(client.is_err());

    let _client = get_client(
        String::from(bind_address),
        String::from(host_address),
        80,
        AuthMode::Password {
            username: String::from(auth_username),
            password: String::from(auth_password),
        },
    )
    .await
    .expect("Failed to create a SOCKS5 client");

    server_proc.abort();
    sleep(Duration::from_millis(200)).await;
}
