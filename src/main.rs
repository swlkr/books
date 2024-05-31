use ryde::*;

fn routes() -> Router {
    Router::new().route("/", get(get_slash)).route("/*files", get(get_files))
}

#[main]
async fn main() {
    serve("127.0.0.1:9012", routes()).await;
}

async fn get_slash() -> Html {
    html! {
        <!DOCTYPE > 
        <html>
            <head>{render_static_files!()}</head>
            <body class="grid place-content-center dark:bg-slate-950 dark:text-white">
                <div class="text-4xl">hello</div>
            </body>
        </html>
    }
}

embed_static_files!("static");
