#[macro_use]
extern crate rocket;
use std::io::Cursor;

use cufarul::{
    db::Database,
    model::{CollectionKey, CompositionRepr, ModeRepr, ModelRepr, PersonRepr, TextRepr},
    repo::{Cufarul, Repository},
};
use rocket::{http::ContentType, response::Responder, Response, State};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

struct AppState {
    repo: cufarul::repo::CufarulRepository,
}

#[get("/")]
fn index(state: &State<AppState>) -> Template {
    #[derive(Serialize)]
    struct Context {
        compositions: Vec<CompositionRepr>,
        composers: Vec<PersonRepr>,
        texts: Vec<TextRepr>,
        modes: Vec<ModeRepr>,
    }

    let composition = state.repo.compositions(None);
    let texts = state.repo.texts(None);
    let composers = state
        .repo
        .people(None)
        .into_iter()
        .filter(|elem| elem.compositions.len() > 0)
        .collect();

    let modes = state.repo.modes();

    Template::render(
        "index",
        context! { count: state.repo.db().nodes_iter().count(), data: Context {
            compositions: composition,
            composers: composers,
            texts: texts,
            modes: modes.into_values().collect(),
        }},
    )
}

#[get("/compositions/<id>")]
fn compositions(id: String, state: &State<AppState>) -> Template {
    let data = state
        .repo
        .model_by_id(CollectionKey::Composition(id.into()), None)
        .expect("Sf");

    Template::render("composition", context! { data: data })
}

#[get("/people/<id>")]
fn people(id: String, state: &State<AppState>) -> Template {
    let data = state
        .repo
        .model_by_id(CollectionKey::Person(id.into()), None)
        .expect("Sf");

    Template::render("person", context! { data: data })
}

#[get("/texts/<id>")]
fn texts(id: String, state: &State<AppState>) -> Template {
    let data = state
        .repo
        .model_by_id(CollectionKey::Text(id.into()), None)
        .expect("Sf");

    Template::render("text", context! { data: data })
}

#[get("/publications/<id>")]
fn publications(id: String, state: &State<AppState>) -> Template {
    let data = state
        .repo
        .model_by_id(CollectionKey::Publication(id.into()), None)
        .expect("Sf");

    Template::render("publication", context! { data: data })
}

#[get("/modes/<id>")]
fn modes(id: u8, state: &State<AppState>) -> Template {
    assert!(id >= 1 && id <= 8);
    let data = state.repo.modes().get(&id).expect("ofof").to_owned();
    Template::render("mode", context! { data: data })
}

#[get("/search")]
fn search() -> Template {
    Template::render("search", context! {})
}

#[derive(Serialize)]
struct QueryResponse {
    results: Vec<Box<dyn ModelRepr>>,
}

impl<'o> Responder<'o, 'static> for QueryResponse {
    fn respond_to(self, _request: &'o rocket::Request<'_>) -> rocket::response::Result<'static> {
        let res = serde_json::to_string(&self).expect("sff");
        Response::build()
            .header(ContentType::JSON)
            .sized_body(res.len(), Cursor::new(res))
            .ok()
    }
}

#[post("/search?<query>&<collection>")]
fn search_post(
    query: String,
    collection: Option<String>,
    state: &State<AppState>,
) -> QueryResponse {
    let res = QueryResponse {
        results: state.repo.query(collection, query),
    };

    // Json(res)
    res
}

#[launch]
fn launch() -> _ {
    let mut repo = cufarul::repo::CufarulRepository::from_spec(
        cufarul::repo::RepositorySpec::from_path(
            std::path::PathBuf::from(std::env::args().nth(1).expect("missing")).as_path(),
        )
        .expect("oops"),
    )
    .expect("oops");
    repo.sync().expect("ok");

    // println!("{:#?}", repo.db());

    let state = AppState { repo: repo };

    rocket::build()
        .mount(
            "/",
            routes![
                index,
                compositions,
                people,
                texts,
                publications,
                modes,
                search,
                search_post,
            ],
        )
        .attach(Template::fairing())
        .manage(state)
}
