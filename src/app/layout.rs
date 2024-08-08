use maud::{DOCTYPE, html, Markup};
use axum::response::Html;


pub fn page(headers: Markup, page: Markup) -> Html<String> {
    let t = html! {
        (DOCTYPE)
        head {
            meta charset="UTF-8";
            meta http-equiv="X-UA-Compatible" content="IE=edge";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            title {"MotionRank"}
            script src="https://kit.fontawesome.com/c018f1996d.js" crossorigin="anonymous" {}
            link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css";
            link rel="stylesheet" href="/style.css";
            link rel="icon" href="/icon.png" type="image/x-icon";
            // script[src="https://unpkg.com/htmx.org@1.8.4"] {}
            script src="https://unpkg.com/htmx.org@1.8.4/dist/htmx.js" {}
            script src="https://unpkg.com/htmx.org/dist/ext/multi-swap.js" {}
            script src="https://cdnjs.cloudflare.com/ajax/libs/cash/8.1.2/cash.min.js" {}
            script src="/index.js" {}

            (headers)
        }
        body hx-ext="multi-swap" {
            (page)
        }
    };

    Html(t.into_string())
}