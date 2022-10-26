use futures_util::{SinkExt, StreamExt};
use poem::{
	get, handler,
	web::Json,
	endpoint::StaticFilesEndpoint,
    listener::TcpListener,
    web::{
        websocket::{Message, WebSocket},
        Data, Html, Path,
    },
    EndpointExt, IntoResponse, Route, Server,
};

use serde::{ Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
struct DataRow {
	time: i64,
	temp: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DataRows {
	msg: String,
	rows: Vec<DataRow>,
}


#[handler]
fn ws(
    Path(name): Path<String>,
    ws: WebSocket,
    sender: Data<&tokio::sync::broadcast::Sender<Json<DataRows>>>,
) -> impl IntoResponse {
    let sender = sender.clone();
    let mut receiver = sender.subscribe();
    ws.on_upgrade(move |socket| async move {
        let (mut sink, mut stream) = socket.split();

        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                if let Message::Text(text) = msg {
					if sender.send(Json(DataRows { 
						msg: String::from("Hello"), 
						rows: vec![DataRow{time: 10, temp: 0.1}, DataRow{time: 20, temp: 0.3}] }
					)).is_err()
					{
						break;
					}
                }
            }
        });
		
        tokio::spawn(async move {
            while let Ok(msg) = receiver.recv().await {
                if sink.send(msg).await.is_err() {
                    break;
                }
            }
        });

    })
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

	let app = Route::new().nest(
		"/",
		StaticFilesEndpoint::new("./www/").index_file("index.html"),
	).at(
		"/ws/:name/", 
		get(ws.data(tokio::sync::broadcast::channel::<String>(32).0)),
	);

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}
