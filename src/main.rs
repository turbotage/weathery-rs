use tokio::{task};

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
use serde_json;

mod smhi;

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
    sender: Data<&tokio::sync::broadcast::Sender<String>>,
) -> impl IntoResponse {
    println!("in ws");
    let sender = sender.clone();
    let mut receiver = sender.subscribe();
    ws.on_upgrade(move |socket| async move {
        let (mut sink, mut stream) = socket.split();

        tokio::spawn(async move {
            while let Some(Ok(msg)) = stream.next().await {
                if let Message::Text(text) = msg {
                    println!("{}", text);
                    let json_str = serde_json::to_string(&DataRows { 
						msg: name.clone(), 
						rows: vec![DataRow{time: 10, temp: 0.1}, DataRow{time: 20, temp: 0.3}] }
					);
                    if let Ok(jstr) = json_str {
                        if sender.send(jstr).is_err() {
                            break;
                        }
                    }
                    else {
                        break;
                    }
                }
            }
        });
		
        tokio::spawn(async move {
            while let Ok(msg) = receiver.recv().await {
                if sink.send(Message::Text(msg)).await.is_err() {
                    println!("In Sink!");
                    break;
                }
            }
        });

    })
}

async fn run_webserver() -> Result<(), std::io::Error> {
    //if std::env::var_os("RUST_LOG").is_none() {
    //    std::env::set_var("RUST_LOG", "poem=debug");
    //}
    //tracing_subscriber::fmt::init();

	let app = Route::new().nest(
		"/",
		StaticFilesEndpoint::new("./www/").index_file("index.html"),
	).at(
		"/ws/:name", 
		get(ws.data(tokio::sync::broadcast::channel::<String>(32).0)),
	);

    Server::new(TcpListener::bind("127.0.0.1:3000"))
        .run(app)
        .await
}



#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let web_future = task::spawn(run_webserver());

    let web_fut_ret = web_future.await?;

    return web_fut_ret;
}
