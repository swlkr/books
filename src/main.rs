#![allow(non_snake_case)]

use std::collections::HashSet;

use ryde::*;

#[router]
fn routes(cx: Cx) -> Router {
    Router::new()
        .route("/", get(get_slash))
        .route("/*files", get(get_files))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(cx)
}

#[derive(Clone)]
struct Cx {
    db: Db,
    x_request: bool
}

impl Cx {
    fn render(&self, component: Component) -> Html {
        match self.x_request {
            true => component,
            false => html! { <View>{component}</View> }
        }
    }
}

#[main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let db = db(dotenv("DATABASE_URL").expect("DATABASE_URL not found in .env")).await?;
    let _x = db.create_books().await?;
    tracing::debug!("listening on 9012");
    serve("127.0.0.1:9012", routes(Cx { db, x_request: false })).await;
    Ok(())
}

#[derive(Deserialize)]
struct Params {
    author: Option<Vec<String>>,
}

use axum_extra::extract::Form;

async fn get_slash(cx: Cx, db: Db, Form(Params { author }): Form<Params>) -> Result<Html> {
    let authors = db.select_authors().await?;
    let res = match author {
        None => {
            let books = db.books().await?;

            html! {
                <>
                    <Authors authors=authors selected_authors=HashSet::default()/>
                    <BookList books=books/>
                </>
            }
        }
        Some(selected_authors) => {
            let selected = selected_authors.clone().into_iter().collect::<HashSet<_>>();
            let mut books = vec![];
            for author in selected_authors {
                let rows = db.books_by_author(Some(author)).await?;
                books.extend(rows);
            }

            html! {
                <>
                    <Authors authors=authors selected=selected/>
                    <BookList books=books/>
                </>
            }
        }
    };

    Ok(cx.render(res))
}

fn View(elements: Elements) -> Component {
    html! {
        <!DOCTYPE html> 
        <html>
            <head>{render_static_files!()}</head>
            <body class="font-sans flex flex-col max-w-screen-xl mx-auto mt-8 gap-8 bg-gray-100 dark:bg-zinc-950 dark:text-white">
                <div class="">
                    <Search/>
                </div>
                <div class="grid grid-cols-12 gap-4">{elements}</div>

            </body>
        </html>
    }
}

fn Search() -> Component {
    html! {
        <input
            type="search"
            name="search"
            class="w-full rounded-md p-2.5 dark:text-black outline-0"
        />
    }
}

fn BookList(books: Vec<Book>) -> Component {
    html! {
        <div id="BookList" class="col-span-9">
            <div class="grid grid-cols-5 gap-4">
                {books
                    .iter()
                    .map(|book| {
                        html! {
                            <img
                                class="object-fit w-full h-full transform transition duration-150 hover:scale-110 cursor-pointer rounded-md"
                                src=book.coverImg
                            />
                        }
                    })}

            </div>
        </div>
    }
}

fn Authors(authors: Vec<SelectAuthors>, selected: HashSet<String>) -> Component {
    html! {
        <div class="col-span-3 bg-white shadow-sm dark:bg-zinc-800 lg:h-[70vh] h-80 overflow-y-auto p-4 rounded-md">
            <h1 class="text-lg font-bold">Authors</h1>
            <form x-get=url!(get_slash) x-replace="BookList">
                // hx-get=url!(get_books)
                // hx-swap="outerHTML"
                // hx-trigger="change"
                // hx-target="#BookList"
                // hx-push-url="true"
                {authors
                    .iter()
                    .map(|author| {
                        html! {
                            <details class="[&_svg]:open:-rotate-180 select-none">
                                <summary class="flex justify-between cursor-pointer dark:hover:bg-zinc-600 hover:bg-zinc-100 p-1 rounded-md items-center">
                                    <div class="flex gap-2">
                                        <div>{&author.letter}</div>
                                        <div>"(" {author.author_count} ")"</div>
                                    </div>
                                    <svg
                                        class="rotate-0 transform transition-all duration-150"
                                        fill="none"
                                        height="20"
                                        width="20"
                                        stroke="currentColor"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        viewBox="0 0 24 24"
                                    >
                                        <polyline points="6 9 12 15 18 9"></polyline>
                                    </svg>
                                </summary>
                                {&author
                                    .authors
                                    .split(",,,")
                                    .map(|a| {
                                        html! {
                                            <div class="flex gap-2">

                                                {match selected.contains(a) {
                                                    true => {
                                                        html! {
                                                            <input
                                                                type="checkbox"
                                                                name="author"
                                                                value=a
                                                                checked="checked"
                                                            />
                                                        }
                                                    }
                                                    false => {
                                                        html! { <input type="checkbox" name="author" value=a/> }
                                                    }
                                                }}
                                                <span class="text-sm">{a}</span>
                                            </div>
                                        }
                                    })}

                            </details>
                        }
                    })}

                <input type="submit" value="submit"/>
            </form>
        </div>
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Db
where
    Cx: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        _parts: &mut http::request::Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let Cx { db, .. } = Cx::from_ref(state);

        Ok(db)
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Cx
where
    Cx: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
                let headers = HeaderMap::from_request_parts(parts, state)
            .await
            .map_err(|_| Error::NotFound)?;
        let x_request = match headers.get("x-request") {
            Some(header_value) => header_value.as_bytes() == b"true",
            None => false,
        };
        let mut cx = Cx::from_ref(state);
        cx.x_request = x_request;

        Ok(cx)
    }
}

embed_static_files!("static");

db!(
    create_books = "
        create table if not exists books (
            bookId text,
            title text,
            series text,
            author text collate nocase,
            rating text,
            description text,
            language text,
            isbn text,
            genres text,
            characters text,
            bookFormat text,
            edition text,
            pages text,
            publisher text,
            publishDate text,
            firstPublishDate text,
            awards text,
            numRatings text,
            ratingsByStars text,
            likedPercent text,
            setting text,
            coverImg text,
            bbeScore text,
            bbeVotes text,
            price text
        );
    " as Book,

    select_authors = "
        select
            distinct substr(author, 1, 1) as letter,
            group_concat(author, ',,,') as authors,
            count(distinct author) as author_count
        from (
            select author, coverImg
            from books
            group by author collate nocase
            order by author collate nocase
        ) as books
        where books.coverImg is not null
        group by letter collate nocase
        order by author collate nocase asc
    ",

    books = "select books.* from books where books.coverImg is not null and books.coverImg != '' order by cast(numRatings as integer) desc limit 30" as Vec<Book>,

    books_by_author = "select books.* from books where author = ? order by cast(numRatings as integer) desc limit 30" as Vec<Book>
);
