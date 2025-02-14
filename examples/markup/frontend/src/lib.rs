use zoon::{println, *};

#[static_ref]
fn counter() -> &'static Mutable<i32> {
    Mutable::new(0)
}

fn increment() {
    counter().update(|counter| counter + 1)
}

fn decrement() {
    counter().update(|counter| counter - 1)
}

fn root() -> impl Element {
    Column::new()
        .item(html_example())
        .item(markdown_example())
        .item(maud_example())
}

// ------ HTML ------

fn html_example() -> impl Element {
    RawHtmlEl::<web_sys::HtmlDivElement>::from_markup(
        r#"
        <div>
            <button id="btn-decrement">-</button>
            <p id="counter-value"></p>
            <button id="btn-increment">+</button>
        </div>
        "#,
    )
    .unwrap_throw()
    .update_html_child("#btn-decrement", |child| {
        child.event_handler(|_: events::Click| decrement())
    })
    .update_html_child("#counter-value", |child| {
        child.child_signal(counter().signal())
    })
    .update_html_child("#btn-increment", |child| {
        child.event_handler(|_: events::Click| increment())
    })
}

// ------ MARKDOWN ------

fn markdown_to_html(markdown: &str) -> String {
    let options = pulldown_cmark::Options::all();
    let parser = pulldown_cmark::Parser::new_ext(markdown, options);
    let mut html_text = String::new();
    pulldown_cmark::html::push_html(&mut html_text, parser);
    html_text
}

fn markdown_example() -> impl Element {
    Column::new()
        .item(counter_info())
        .item(divider())
        .item(html_page())
}

fn counter_info() -> impl Element {
    fn content(counter_value: i32) -> String {
        markdown_to_html(&format!("Counter value: _**{counter_value}**_"))
    }
    RawHtmlEl::new("div").inner_markup_signal(counter().signal().map(content))
}

fn divider() -> impl Element {
    RawHtmlEl::<web_sys::HtmlElement>::from_markup(markdown_to_html("---"))
        .unwrap_throw()
        .style("width", "100%")
}

fn html_page() -> impl Element {
    RawHtmlEl::new("div").inner_markup(markdown_to_html(include_str!("markdown_page.md")))
}

// ------ MAUD ------

fn maud_example() -> impl Element {
    let template = maud::html! {
        h1 { "Hello from Maud!" }
        p.intro {
            "This is an example of the "
            a href="https://github.com/lambda-fairy/maud" { "Maud" }
            " template language."
        }
    };
    RawHtmlEl::new("div")
        .inner_markup(template.into_string())
        .update_html_child("h1:first-child", |child| {
            child
                .style("cursor", "pointer")
                .event_handler(|_: events::Click| println!("Hello!"))
        })
}

#[wasm_bindgen(start)]
pub fn start() {
    start_app("app", root);
}
