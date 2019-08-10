use futures::future;
use hyper::client::HttpConnector;
use hyper::rt::{Future, Stream};
use hyper::service::service_fn;
use hyper::{header, Body, Client, Method, Request, Response, Server, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

mod object_types;
mod telegram;

use telegram::Telegram;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type BoxFut = Box<dyn Future<Item = Response<Body>, Error = GenericError> + Send>;

fn callback_route_post(req: Request<Body>, app: &App) -> BoxFut {
    let app = app.clone();

    Box::new(
        req.into_body()
            .concat2() // Concatenate all chunks in the body
            .from_err()
            .and_then(move |entire_body| {
                let str = String::from_utf8(entire_body.to_vec())?;
                let message: object_types::GroupmeMessage = serde_json::from_str(&str)?;

                dbg!(&message);

                app.tg_client.relay_message(message, &app.config.tg_chat_id);

                let json = serde_json::to_string(&json!({
                    "success": true
                }))
                .unwrap();

                let response = Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json))?;

                Ok(response)
            }),
    )
}

/// This is our service handler. It receives a Request, routes on its
/// path, and returns a Future of a Response.
fn router(req: Request<Body>, app: &App) -> BoxFut {
    let callback_path = &app.callback_path;

    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => {
            let body = Body::from("Hello There!");
            Box::new(future::ok(Response::new(body)))
        }

        // Simply echo the body back to the client.
        (&Method::POST, callback_path) => callback_route_post(req, app),

        _ => {
            // Return 404 not found response.
            let body = Body::from("Not Found");
            Box::new(future::ok(
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(body)
                    .unwrap(),
            ))
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Config {
    tg_token: String,
    tg_chat_id: String,
    path_secret: String,
}

#[derive(Debug, Clone)]
struct App {
    config: Arc<Config>,
    client: Client<HttpConnector>,
    tg_client: Arc<Telegram>,
    callback_path: Arc<String>,
}

impl App {
    pub fn new(client: Client<HttpConnector>, config: Config) -> App {
        let callback_path = format!("/{}/callback", config.path_secret);
        let tg_client = Telegram::new(&config.tg_token);

        App {
            config: Arc::new(config),
            client,
            tg_client: Arc::new(tg_client),
            callback_path: Arc::new(callback_path),
        }
    }
}

fn main() {
    let addr = ([0, 0, 0, 0], 4200).into();

    let config = envy::from_env::<Config>()
        .expect("Configuration failed! Make sure have set all required fields.");

    hyper::rt::run(future::lazy(move || {
        let client = Client::new();

        //        let https = HttpsConnector::new(4).unwrap();
        //        let client = Client::builder().build::<_, hyper::Body>(https);

        let app = App::new(client, config);

        let new_service = move || {
            let app = app.clone();
            service_fn(move |req| {
                router(req, &app).map_err(|e| {
                    eprintln!("request handler error: {}", e);
                    e
                })
            })
        };

        let server = Server::bind(&addr)
            .serve(new_service)
            .map_err(|e| eprintln!("server error: {}", e));

        println!("Listening on http://{}", addr);
        server
    }));
}
