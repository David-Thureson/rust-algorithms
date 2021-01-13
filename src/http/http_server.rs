
use rocket;
use rocket::http::Method;
use rocket::response::content;
use rocket_cors::{AllowedHeaders, AllowedOrigins, Error};
use rocket_contrib::json::Json;
use rocket::response::status::Custom;
use serde::{Serialize, Deserialize};
use std::fmt::Debug;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/data_1")]
fn data_1() -> content::Json<&'static str> {
    content::Json("{\"label\":23}")
}

#[get("/data_2")]
fn data_2() -> rocket_contrib::json::Json<CustomStruct> {
    let a = CustomStruct {
        name: "Abel".to_string(),
        id: 4323,
    };
    rocket_contrib::json::Json(a)
}

#[derive(Serialize, Deserialize, Debug)]
struct CustomStruct {
    name: String,
    id: u32,
}

pub fn start(routes: Vec<rocket::Route>) -> Result<(), Error> {

    let allowed_origins = AllowedOrigins::some_exact(&["http://localhost:63342"]);

    // You can also deserialize this
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept", "Access-Control-Allow-Origin"]),
        allow_credentials: true,
        ..Default::default()
    }.to_cors()?;

    rocket::ignite()
        .mount("/", routes)
        .attach(cors)
        .launch();

    Ok(())
}

/*
fn main() -> Result<(), Error> {

    let allowed_origins = AllowedOrigins::some_exact(&["http://localhost:63342"]);

    // You can also deserialize this
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Accept", "Access-Control-Allow-Origin"]),
        allow_credentials: true,
        ..Default::default()
    }.to_cors()?;

    rocket::ignite()
        // .mount("/", routes![index, data_1, data_2])
        .mount("/", routes![index, data_1, data_2])
        .attach(cors)
        .launch();

    // let a:usize = data_2;
    let a:usize = routes![index, data_1, data_2];

    Ok(())
}
*/