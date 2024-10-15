# Rust Web Scraper

A simple web scraper built in Rust using asynchronous programming with `tokio`. The application allows users to start and stop web scraping tasks, list current tasks, and print collected links from scraped websites via a command-line interface (CLI).

## **Features**

- **Asynchronous Scraping:** Utilizes `tokio` for asynchronous operations, allowing multiple scraping tasks to run concurrently.
- **Interactive CLI:** Provides an interactive command-line interface for users to manage scraping tasks.
- **URL Normalization:** Handles various URL formats and normalizes them for consistent processing.
- **Link Collection:** Collects and stores links from scraped websites, preventing duplicates.
- **Logging:** Includes logging capabilities using the `log` and `env_logger` crates for monitoring and debugging.

## **Project Structure**

- **`scraper_lib`**: A library crate containing the scraping logic and the `ScraperManager`.
- **`server`**: A binary crate that runs the server, handling client connections and commands.
- **`cli`**: A binary crate providing the interactive command-line interface for users.

## **Getting Started**

### **Prerequisites**

- **Rust and Cargo**: Install Rust and Cargo from [rustup.rs](https://rustup.rs/).

### **Building the Project**

Clone the repository:

- clone it

Build the project:

cargo build

Running the Server:

cargo run -p server

Using the CLI

In another terminal, run the CLI:

cargo run -p cli

Start Scraping:

> start <url>

- Starts scraping the specified URL.

Stop Scraping:

> stop <url>

- Stops the scraping task for the specified URL.

List Tasks:

> list

- Lists all current scraping tasks with their statuses.

Print Links:

> print <url>

- Prints the links collected from the specified URL.

Help:

> help

- Displays the list of available commands.

Exit:

> exit

Example:

> start colinrhys.io
Started scraping

> list
http://colinrhys.io: crawling

> stop colinrhys.io
Stopped scraping

> list
http://colinrhys.io: stopped

> print colinrhys.io
http://colinrhys.io/
http://colinrhys.io/about
...

> exit


Logging

The application uses the log and env_logger crates for logging. You can control the log level by setting the RUST_LOG environment variable.

Example:
RUST_LOG=info cargo run -p server