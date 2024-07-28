use icfpc_2024::*;
use reqwest::Client;
use reqwest::Error;
use std::io::{self, Write};

pub async fn run_repl_loop() {
    loop {
        // Prompt the user for input
        print!("\nEnter a string (or type 'exit' to quit):\n");
        io::stdout().flush().expect("Failed to flush stdout");

        // Read input from the user
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        // Trim the input to remove trailing newline
        let input = input.trim();

        // Check for exit condition
        if input.eq_ignore_ascii_case("exit") {
            break;
        }

        // Run the foo function over the input
        let res = run_repl(input).await;
        println!("\n[debug] Result = {:?}", res);
    }
}

pub async fn run_repl(input: &str) -> Result<(), Error> {
    // let send = "get language_test".to_string();
    let send = input.to_string();
    let encoded = encode_string(send);
    let encoded_prefix: String = encoded.iter().collect();
    let encoded_string = "S".to_string() + &encoded_prefix;
    println!("Hello, world!");
    println!("{:?}", parse_token(encoded_string.clone()));

    let client = Client::new();

    let response = client
        .post("http://localhost:8000/communicate")
        .body(encoded_string)
        .header(
            "Authorization",
            "Bearer 00000000-0000-0000-0000-000000000000",
        )
        .send()
        .await?;

    if response.status().is_success() {
        let body = response.text().await?;
        // let chars: Vec<char> = body.chars().collect();
        println!("Response Text: '{}'", body);
        let code = parse_into_ast(body);
        let evaled = eval(code);
        let expr = &*evaled.borrow();
        println!("Decoded: \n\n{}", unwrap_string(expr));
    } else {
        println!("HTTP Request failed with status: {}", response.status());
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // test_unary_operators();
    // test_binary_operators();
    // test_if_operator();
    // test_lambda_operator();
    // test_lambda_operator3();
    // let _ = run_repl().await;
    let _ = run_repl_loop().await;
    // test_language_test()
}
