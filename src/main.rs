use std::{env, time};

use curl::easy::Easy; // version: 0.4.46
use rand::{ // version: 0.8.5
    distributions::{Alphanumeric, DistString},
    rngs::StdRng,
    SeedableRng
}; 

fn main() {
    // get the URL path from the environment
    let path = get_env_variable("URL_PATH");
    
    // get the session ID from the environment
    let session_id = get_env_variable("SESSION_ID");
    let mut session_id_cookie = make_php_session_cookie(&session_id);

    // create a random number generator
    let mut rng = StdRng::from_entropy();

    // set up stats tracking
    let mut num_requests = 0;
    let initial_time = time::Instant::now();

    // loop forever, generating random IDs and sending requests
    loop {
        // increment the number of requests
        num_requests += 1;
        
        // print some stats every 100 requests to stderr
        if num_requests % 100 == 0 {
            let total_seconds = initial_time.elapsed().as_secs();
            let display_minutes = total_seconds / 60;
            let display_seconds = total_seconds % 60;
            let reqs_per_second = (num_requests as f64) / (total_seconds as f64);

            eprintln!("{num_requests:15} reqs | {display_minutes:5}m{display_seconds:02}s | {reqs_per_second:5.2} req/s");
        }
        
        // generate a random file ID and make a request
        let file_id = generate_id(&mut rng);
        let url = make_request(&path, &file_id);
        let result = send_request(&url, &session_id_cookie);

        // handle the result
        match result {
            // if the request was successful, print the file ID
            Ok(200) => println!("{file_id}"),

            // if the session ID has expired, prompt for a new one
            Ok(302) => {
                let new_session_id = prompt_session_id();
                session_id_cookie = make_php_session_cookie(&new_session_id);
            },
            
            // if the file ID was not found, do nothing
            Ok(404) => (),

            // if the response code was unexpected, print a warning
            Ok(code) => eprintln!("WARNING: unexpected code {code} for file ID {file_id}"),

            // if there was an error, print the error
            Err(e) => eprintln!("ERROR: {e}"),
        }
    }
}

/// generate a random ID in the form of
/// XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX
/// where `X` is an alpha-numeric character
fn generate_id(mut rng: &mut StdRng) -> String {
    let parts: [String; 5] = [
        Alphanumeric.sample_string(&mut rng, 8),
        Alphanumeric.sample_string(&mut rng, 4),
        Alphanumeric.sample_string(&mut rng, 4),
        Alphanumeric.sample_string(&mut rng, 4),
        Alphanumeric.sample_string(&mut rng, 12),
    ];
    format!("{}-{}-{}-{}-{}", parts[0], parts[1], parts[2], parts[3], parts[4])
}

/// send a request to the given URL and return the response code
fn send_request(url: &str, session_id_cookie: &str) -> Result<u32, curl::Error> {
    let mut handle = Easy::new();
    handle.url(url)?;
    handle.cookie(session_id_cookie)?;

    handle.perform()?;
    handle.response_code()
}

/// given a PHPSESSID value, make a cookie
fn make_php_session_cookie(session_id: &str) -> String {
    format!("PHPSESSID={session_id}")
}

/// given a domain, a path, and file ID, make a URL for the request
fn make_request(path: &str, file_id: &str) -> String {
    format!("{path}?fileId={file_id}")
}

/// get the given environment variable, or print and exit
fn get_env_variable(var: &str) -> String {
    let result = env::var(var);
    match result {
        Ok(var) => var,
        Err(_) => {
            eprintln!("ERROR: Must define {var} environment variable!");
            std::process::exit(1);
        }
    }
}

/// prompt for a new session ID
fn prompt_session_id() -> String {
    eprintln!("ERROR: your session ID has expired");
    eprint!("Please input a new session ID (value only): ");

    // read the session ID from stdin
    let mut session_id = String::new();
    std::io::stdin().read_line(&mut session_id).unwrap();
    session_id.trim().to_string()
}
