use structopt::{clap::arg_enum, StructOpt};
use tonic_interop::client;
use tonic::transport::Channel;

#[derive(StructOpt)]
struct Opts {
    #[structopt(
        long = "test_case",
        use_delimiter = true,
        min_values = 1,
        raw(possible_values = r#"&Testcase::variants()"#)
    )]
    test_case: Vec<Testcase>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sub = tracing_fmt::FmtSubscriber::builder().finish();
    tracing::subscriber::set_global_default(sub).unwrap();
    let _ = tracing_log::LogTracer::init();

    let matches = Opts::from_args();

    let test_cases = matches.test_case;

    let addr = "localhost:10000";
    let origin = http::Uri::from_shared(format!("http://{}", addr).into()).unwrap();

    let channel = Channel::builder()
        // .tls(ca)
        // .tls_override_domain("foo.test.google.fr")
        .build(origin)?;

    let mut client = client::TestClient::new(channel.clone());
    let mut unimplemented_client = client::UnimplementedClient::new(channel);

    let mut failures = Vec::new();

    for test_case in test_cases {
        println!("{:?}:", test_case);
        let mut test_results = Vec::new();

        match test_case {
            Testcase::empty_unary => client::empty_unary(&mut client, &mut test_results).await,
            Testcase::large_unary => client::large_unary(&mut client, &mut test_results).await,
            Testcase::client_streaming => {
                client::client_streaming(&mut client, &mut test_results).await
            }
            Testcase::server_streaming => {
                client::server_streaming(&mut client, &mut test_results).await
            }
            Testcase::ping_pong => client::ping_pong(&mut client, &mut test_results).await,
            Testcase::empty_stream => client::empty_stream(&mut client, &mut test_results).await,
            Testcase::status_code_and_message => {
                client::status_code_and_message(&mut client, &mut test_results).await
            }
            Testcase::special_status_message => {
                client::special_status_message(&mut client, &mut test_results).await
            }
            Testcase::unimplemented_method => {
                client::unimplemented_method(&mut client, &mut test_results).await
            }
            Testcase::unimplemented_service => {
                client::unimplemented_service(&mut unimplemented_client, &mut test_results).await
            }
            Testcase::custom_metadata => {
                client::custom_metadata(&mut client, &mut test_results).await
            }
            _ => unimplemented!(),
        }

        for result in test_results {
            println!("  {}", result);

            if result.is_failed() {
                failures.push(result);
            }
        }
    }

    if !failures.is_empty() {
        println!("{} tests failed", failures.len());
        std::process::exit(1);
    }

    Ok(())
}

arg_enum! {
    #[derive(Debug, Copy, Clone)]
    #[allow(non_camel_case_types)]
    enum Testcase {
        empty_unary,
        cacheable_unary,
        large_unary,
        client_compressed_unary,
        server_compressed_unary,
        client_streaming,
        client_compressed_streaming,
        server_streaming,
        server_compressed_streaming,
        ping_pong,
        empty_stream,
        compute_engine_creds,
        jwt_token_creds,
        oauth2_auth_token,
        per_rpc_creds,
        custom_metadata,
        status_code_and_message,
        special_status_message,
        unimplemented_method,
        unimplemented_service,
        cancel_after_begin,
        cancel_after_first_response,
        timeout_on_sleeping_server,
        concurrent_large_unary
    }
}