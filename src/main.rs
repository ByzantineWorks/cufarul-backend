#[macro_use]
extern crate rocket;
use cufarul::{
    db::Database,
    model::{CollectionKey, CompositionRepr, PersonRepr, TextRepr},
    repo::{Cufarul, Repository},
};
use rocket::State;
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
    }

    let composition = state.repo.compositions(None);
    let texts = state.repo.texts(None);
    let composers = state
        .repo
        .people(None)
        .into_iter()
        .filter(|elem| elem.compositions.len() > 0)
        .collect();

    Template::render(
        "index",
        context! { count: state.repo.db().nodes_iter().count(), data: Context {
            compositions: composition,
            composers: composers,
            texts: texts,
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

    Template::render("text", context! { data: data })
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
            routes![index, compositions, people, texts, publications],
        )
        .attach(Template::fairing())
        .manage(state)
}
