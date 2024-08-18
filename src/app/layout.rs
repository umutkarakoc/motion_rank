use axum::response::Html;
use maud::{html, Markup, DOCTYPE};

pub fn page(headers: Markup, page: Markup) -> Html<String> {
    let t = html! {
        (DOCTYPE)
        head {
            meta charset="UTF-8";
            meta http-equiv="X-UA-Compatible" content="IE=edge";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            title {"MotionRank"}
            link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-QWTKZyjpPEjISv5WaRU9OFeRpok6YctnYmDr5pNlyT2bRjXh0JMhjY6hW+ALEwIH" crossorigin="anonymous";
            link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bootstrap-icons@1.11.3/font/bootstrap-icons.min.css";
            link rel="stylesheet" href="/style.css";
            link rel="icon" href="/icon.png" type="image/x-icon";
            script src="https://unpkg.com/htmx.org@2.0.2" {}
            script src="https://unpkg.com/htmx-ext-multi-swap@2.0.0/multi-swap.js" {}
            script src="https://ajax.googleapis.com/ajax/libs/jquery/3.7.1/jquery.min.js" {}
            script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-YvpcrYf0tY3lHB60NNkmXc5s9fDVZLESaAA55NDzOxhy9GkcIdslK1eN7N6jIeHz" crossorigin="anonymous" {}
            script src="https://cdn.jsdelivr.net/npm/@popperjs/core@2.11.8/dist/umd/popper.min.js" integrity="sha384-I7E8VVD/ismYTF4hNIPjVp/Zjvgyol6VFvRkX/vR+Vc4jQkC+hVqc2pM8ODewa9r" crossorigin="anonymous" {}

            script src="/index.js" {}

            (headers)
        }
        body hx-ext="multi-swap" {
            (page)


        }
    };

    Html(t.into_string())
}
