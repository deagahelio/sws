extern crate tiny_http;
extern crate try_exit;
extern crate mime_guess;
extern crate ascii;

use tiny_http::{Server, Response, ResponseBox, Header};
use try_exit::try;
use mime_guess::mime_str_for_path_ext;
use ascii::AsciiString;
use std::fs::File;
use std::io::Read;
use std::path::Path;

// ========== CONFIGURATION ==========

// Server variables
const ADDRESS: &'static str = "0.0.0.0";
const PORT: &'static str = "8080";
// Path to files
const PATH_INDEX: &'static str = "index.html";
const PATH_404: &'static str = "404.html";
// Text to serve if 404 file doesn't exist
const TEXT_404: &'static str = "404 NOT FOUND";
// If `path` can't be found, try serving `path.html`
const TRY_APPEND_HTML: bool = true;

// ===================================

fn main() {
	println!("[EVENT] Starting server at {}:{}", ADDRESS, PORT);

	let server = try(Server::http(format!("{}:{}", ADDRESS, PORT)), "[ERROR] Couldn't create server");

	for request in server.incoming_requests() {
		println!("[EVENT] Received request for {}", request.url());

		let mut response: ResponseBox;

		// Drop first slash in the request path
		let mut path_string = String::from(request.url());
		path_string.remove(0);

		// If file doesn't exist, try path + .html
		if !Path::new(&path_string).exists() && TRY_APPEND_HTML {
			path_string.push_str(".html");
		}

		let mut path = Path::new(&path_string);

		// Serve index.html if no path
		if path_string.is_empty() {
			println!("[EVENT] Serving /index.html instead");
			path = Path::new(PATH_INDEX);
		}

		let file = File::open(path);

		// If file exists, serve it
		if file.is_ok() {
			response = Response::from_file(file.unwrap())
						// Add header specifying MIME type of file
						.with_header(
							Header {
								field: "Content-Type".parse().unwrap(),
								value: AsciiString::from_ascii(mime_str_for_path_ext(path).unwrap_or_else(|| "application/octet-stream")).unwrap(),
							}
						)
						.boxed();
		// Otherwise, serve 404 page
		} else {
			println!("[EVENT] Resource not found, serving 404");

			let mut file = File::open(PATH_404);

			// If 404.html doesn't exist
			if file.is_err() {
				response = Response::from_string(TEXT_404)
							.with_status_code(404)
							.boxed();
			} else {
				// Read 404.html contents and build response
				let mut contents = String::new();
				try(file.unwrap().read_to_string(&mut contents), "[ERROR] Couldn't read resource");

				response = Response::from_string(contents)
							.with_header(
								Header {
									field: "Content-Type".parse().unwrap(),
									value: AsciiString::from_ascii("text/html").unwrap(),
								}
							)
							.with_status_code(404)
							.boxed();
			}
		}

		try(request.respond(response), "[ERROR] Couldn't respond to request");
	}
}