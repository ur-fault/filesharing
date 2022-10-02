use std::path::{Path, PathBuf};

use rocket::{
    self,
    data::{Limits, ToByteUnit},
    form::Form,
    fs::NamedFile,
    fs::TempFile,
    get, launch, post,
    response::Redirect,
    routes, uri, Build, Config, FromForm, Rocket,
};
use rocket_dyn_templates::{context, Template};
use uuid::Uuid;

#[derive(FromForm)]
struct File<'a> {
    file: TempFile<'a>,
}

#[get("/<name>/<age>")]
async fn hello(name: String, age: u8) -> String {
    format!("Hello, {} year old named {}!", age, name)
}

#[get("/image")]
async fn image() -> NamedFile {
    NamedFile::open("./content/image.jpg").await.unwrap()
}

#[post("/upload", data = "<form>")]
async fn upload(mut form: Form<File<'_>>) -> Result<Redirect, String> {
    let id = Uuid::new_v4();
    let name = form.file.name().unwrap_or("unkown");
    let ext = form
        .file
        .content_type()
        .map(|typ| typ.extension().map(|ext| ext.as_str()).unwrap_or("unk"))
        .unwrap_or("unk");

    let name = format!("{}-{}.{}", id, name, ext,);

    let path = format!("./sharing/{}", name);

    if let Err(err) = form.file.persist_to(&path).await {
        return Err(err.to_string());
    }

    Ok(Redirect::to(
        uri! {download_page(file = PathBuf::from(name))},
    ))
}

#[get("/uppage")]
async fn upload_page() -> Template {
    Template::render("upload", context! {})
}

#[get("/download/<file..>")]
async fn download(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("./sharing/").join(file))
        .await
        .ok()
}

#[get("/downpage/<file..>")]
async fn download_page(file: PathBuf) -> Option<Template> {
    Some(Template::render(
        "download",
        context! {file: uri! {download(file = file)}},
    ))
}

#[get("/")]
async fn index() -> Redirect {
    Redirect::to(uri! {upload_page})
}

#[launch]
fn rocket() -> Rocket<Build> {
    rocket::build()
        .mount("/hello", routes![hello])
        .mount("/", routes![image])
        .mount("/", routes![upload, upload_page])
        .mount("/", routes![download_page, download])
        .mount("/", routes![index])
        .configure(
            Config {
                limits: Limits::default()
                    .limit("file", 1024.megabytes())
                    .limit("data-form", 1024.megabytes()),
                ..Config::default()
            },
        )
        .attach(Template::fairing())
}
