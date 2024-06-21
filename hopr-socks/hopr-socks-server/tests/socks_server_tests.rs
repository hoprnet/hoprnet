use hopr_socks_server::cli::AuthMode;
use hopr_socks_server::SocksServer;
use reqwest::{Client, Error, Proxy};
use tokio::time::{sleep, Duration};

static SERVER_DELAY: Duration = Duration::from_millis(50);

async fn socks_client(bind_address: String, auth: AuthMode) -> Result<Client, Error> {
    let proxy = Proxy::all(match auth {
        AuthMode::NoAuth => format!("socks5://{}", bind_address),
        AuthMode::Password { username, password } => format!("socks5://{}:{}@{}", username, password, bind_address),
    })
    .unwrap();

    return Client::builder().proxy(proxy).build();
}

async fn get_request(client: &Client, url: String) -> Result<Vec<u8>, Error> {
    let response = client.get(&url).timeout(Duration::from_secs(5)).send().await?;

    match response.bytes().await {
        Ok(body) => Ok(body.to_vec()),
        Err(e) => Err(e),
    }
}

#[tokio::test]
async fn test_connect_client_incorrect_bind_address() {
    let bind_address = "127.0.0.1:1331";

    let server_proc = tokio::spawn(async move {
        SocksServer::new(String::from(bind_address), 10, AuthMode::NoAuth)
            .await
            .expect("Failed to create a SOCKS5 server")
            .run()
            .await
            .expect("SOCKS5 server could not be run");
    });

    // Open TCP connection through proxy (incorrect bind address)
    let client = socks_client(String::from("127.0.0.2:1332"), AuthMode::NoAuth)
        .await
        .expect("Failed to create a SOCKS5 client");

    assert!(get_request(&client, String::from("http://www.example.com"))
        .await
        .is_err());

    server_proc.abort();
    sleep(SERVER_DELAY).await;
}

#[tokio::test]
async fn test_connect_client_correct_bind_address() {
    let bind_address = "127.0.0.1:1332";

    let server_proc = tokio::spawn(async move {
        SocksServer::new(String::from(bind_address), 10, AuthMode::NoAuth)
            .await
            .expect("Failed to create a SOCKS5 server")
            .run()
            .await
            .expect("SOCKS5 server could not be run");
    });

    sleep(SERVER_DELAY).await;

    // Open TCP connection through proxy (correct bind address)
    let client = socks_client(String::from(bind_address), AuthMode::NoAuth)
        .await
        .expect("Failed to create a SOCKS5 client");
    let _response: Vec<u8> = get_request(&client, String::from("http://www.example.com"))
        .await
        .expect("Failed to send HTTP request");

    server_proc.abort();
    sleep(SERVER_DELAY).await;
}

#[tokio::test]
async fn test_http_request_through_socks_proxy() {
    let bind_address = "127.0.0.1:1333";
    let host_address = "http://www.example.com";

    let server_proc = tokio::spawn(async move {
        SocksServer::new(String::from(bind_address), 10, AuthMode::NoAuth)
            .await
            .expect("Failed to create a SOCKS5 server")
            .run()
            .await
            .expect("SOCKS5 server could not be run");
    });

    sleep(SERVER_DELAY).await;

    // Open TCP connection to existing endpoint
    let client = socks_client(String::from(bind_address), AuthMode::NoAuth)
        .await
        .expect("Failed to create a SOCKS5 client");
    let _response: Vec<u8> = get_request(&client, String::from(host_address))
        .await
        .expect("Failed to send HTTP request");

    server_proc.abort();
    sleep(SERVER_DELAY).await;
}

#[tokio::test]
async fn test_multiple_clients() {
    let bind_address = "127.0.0.1:1334";
    let host_address = "http://www.example.com";

    let server_proc = tokio::spawn(async move {
        SocksServer::new(String::from(bind_address), 10, AuthMode::NoAuth)
            .await
            .expect("Failed to create a SOCKS5 server")
            .run()
            .await
            .expect("SOCKS5 server could not be run");
    });

    sleep(SERVER_DELAY).await;

    // Create a vector of Socks5Stream<TcpStream> clients
    let mut clients = Vec::new();
    for _ in 0..5 {
        let client = socks_client(String::from(bind_address), AuthMode::NoAuth)
            .await
            .expect("Failed to create a SOCKS5 client");
        clients.push(client);
    }

    // Send HTTP request to each client
    for client in clients {
        get_request(&client, String::from(host_address))
            .await
            .expect("Failed to send HTTP request");
    }

    server_proc.abort();
    sleep(SERVER_DELAY).await;
}

#[tokio::test]
async fn test_connect_unauthenticated_client_with_auth() {
    let bind_address = "127.0.0.1:1335";
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

    sleep(SERVER_DELAY).await;

    let client = socks_client(
        String::from(bind_address),
        AuthMode::Password {
            username: String::from(auth_username),
            password: String::from("wrong_password"),
        },
    )
    .await
    .expect("Failed to create a SOCKS5 client");

    assert!(get_request(&client, String::from("http://www.example.com"))
        .await
        .is_err());

    server_proc.abort();
    sleep(SERVER_DELAY).await;
}

#[tokio::test]
async fn test_connect_authenticated_client_with_auth() {
    let bind_address = "127.0.0.1:1336";
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

    sleep(SERVER_DELAY).await;

    let client = socks_client(
        String::from(bind_address),
        AuthMode::Password {
            username: String::from(auth_username),
            password: String::from(auth_password),
        },
    )
    .await
    .expect("Failed to create a SOCKS5 client");

    let _response = get_request(&client, String::from("http://www.example.com"))
        .await
        .expect("Failed to send HTTP request");

    server_proc.abort();
    sleep(SERVER_DELAY).await;
}

#[tokio::test]
async fn test_https_request_through_socks_proxy() {
    let bind_address = "127.0.0.1:1337";
    let host_address = "https://www.example.com";

    let server_proc = tokio::spawn(async move {
        SocksServer::new(String::from(bind_address), 10, AuthMode::NoAuth)
            .await
            .expect("Failed to create a SOCKS5 server")
            .run()
            .await
            .expect("SOCKS5 server could not be run");
    });

    sleep(SERVER_DELAY).await;

    let client = socks_client(String::from(bind_address), AuthMode::NoAuth)
        .await
        .expect("Failed to create a SOCKS5 client");
    let _response = get_request(&client, String::from(host_address))
        .await
        .expect("Failed to send HTTP request");

    server_proc.abort();
    sleep(SERVER_DELAY).await;
}
