#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate asciii;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
extern crate itertools;
extern crate linked_hash_map;

extern crate base64;
extern crate openssl_probe;
extern crate ring;
extern crate rocket;
extern crate rocket_contrib;
extern crate rocket_cors;

use ring::{digest, test as digest_test};

use rocket::http::Method;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::response::content;
use rocket::Outcome;
use rocket_cors::{AllowedHeaders, AllowedOrigins};

use asciii::actions;

use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Mutex;
use std::thread;

lazy_static! {
    pub static ref PROJECTS: Mutex<server::ProjectLoader> = Mutex::new(server::ProjectLoader::new());
    pub static ref CHANNEL: SyncSender<()> = {
        let (tx, rx) = sync_channel::<()>(1);

        thread::spawn(move || {
            println!("background thread");
            let mut count = 0;
            loop {
                rx.recv().unwrap();
                count += 1;
                if count % 6 == 0 {
                    debug!("updating projects");
                    PROJECTS.lock().unwrap().update();
                }
                debug!("callcount: {}", count);
            }
        });
        tx
    };
}

pub mod server;

#[derive(Debug, Serialize, Deserialize)]
pub enum Authorization<'a> {
    ApiKey(&'a str),
    UsernamePassword(String, String),
    Dev  // TODO: remove!
}

impl<'a, 'r> FromRequest<'a, 'r> for Authorization<'a> {
    type Error = ();
    fn from_request(request: &'a Request<'r>) -> request::Outcome<Authorization<'a>, ()> {
        Outcome::Success(Authorization::Dev) // TODO: dev only, remove

        // if let Some(key) = request.headers().get("x-api-key").nth(0) {
        //     return validate_authorization(Authorization::ApiKey(key));
        // }

        // let auth = request
        //     .headers()
        //     .get("Authorization")
        //     .nth(0)
        //     .map(|s| s.split_whitespace().nth(1).unwrap())
        //     .and_then(|s| base64::decode(s.as_bytes()).ok())
        //     .and_then(|v| String::from_utf8(v).ok())
        //     .map(|s| s.split(':').map(ToOwned::to_owned).collect::<Vec<_>>());

        // if let Some(auth) = auth {
        //     let authorization = match (auth.get(0), auth.get(1)) {
        //         (Some(user), Some(pass)) => Authorization::UsernamePassword(user.to_owned(), pass.to_owned()),
        //         _ => return Outcome::Failure((Status::BadRequest, ())),
        //     };

        //     return validate_authorization(authorization);
        // } else {
        //     Outcome::Failure((Status::Unauthorized, ()))
        // }
    }
}

fn validate_authorization<'a>(authorization: Authorization<'a>) -> request::Outcome<Authorization<'a>, ()> {
    // TODO: load keys at const intervals
    let credentials = match actions::meta_store::get_api_keys() {
        Ok(keys) => keys,
        Err(e) => {
            error!("No Users Stored: {}", e);
            return Outcome::Failure((Status::InternalServerError, ()));
        }
    };

    let is_valid = match authorization {
        Authorization::ApiKey(ref key) => credentials.keys.iter().any(|k| k == key),
        Authorization::UsernamePassword(ref user, ref password) => credentials.users.iter().any(|(u, expected_hex)| {
            let hashed_password = digest::digest(&digest::SHA256, password.as_bytes());
            if let Ok(expected) = digest_test::from_hex(expected_hex) {
                u == user && hashed_password.as_ref() == expected.as_slice()
            } else {
                println!("bad hash: {}", expected_hex);
                false
            }
        }),
        _ => unreachable!(),
    };

    if is_valid {
        Outcome::Success(authorization)
    } else {
        Outcome::Failure((Status::Unauthorized, ()))
    }
}

#[get("/authorization")]
fn authorization(authorization: Authorization) -> content::Json<String> {
    match &authorization {
        Authorization::ApiKey(key) => {
            let hashed_key = digest::digest(&digest::SHA256, key.as_bytes());
            println!("{}: {:?}", key, hashed_key);
        }
        Authorization::UsernamePassword(_username, password) => {
            let hashed_pass = digest::digest(&digest::SHA256, password.as_bytes());
            println!("{}: {:?}", password, hashed_pass);
        }
        _ => unreachable!()
    }
    content::Json(serde_json::to_string(&authorization).unwrap())
}

#[get("/features")]
fn features() -> content::Json<&'static str> {
    content::Json(asciii::ENABLED_FEATURES)
}

// TODO: test with drill and/or noir
fn main() {
    openssl_probe::init_ssl_cert_env_vars();

    use server::endpoints;
    use server::endpoints::{calendar, projects};

    let server = rocket::ignite()
        .mount("/", routes![
            endpoints::static_index,
            endpoints::static_files
            ]) // TODO: might be a rocket bug

        .mount("/cal/plain", routes![calendar::cal_plain, calendar::cal_plain_params])
        .mount("/cal", routes![calendar::cal, calendar::cal_params])
        .mount("/api", routes![projects::years,
                               projects::by_year,
                               projects::full_by_year,
                               projects::all_names,
                               projects::all_full,
                               projects::by_name,
                               authorization,
                               features,
        ]);

    if let Ok(env_cors) = std::env::var("CORS_ALLOWED_ORIGINS") {
        println!("Adding CORS Data {}", env_cors);
        let env_allowed_origins = &[env_cors.as_str()];
        let (allowed_origins, failed_origins) = AllowedOrigins::some(env_allowed_origins);
        assert!(failed_origins.is_empty());
        let options = rocket_cors::Cors {
            allowed_origins: allowed_origins,
            allowed_methods: vec![Method::Get].into_iter().map(From::from).collect(),
            allowed_headers: AllowedHeaders::some(&["Authorization", "Accept"]),
            allow_credentials: true,
            ..Default::default()
        };

        server.attach(options)
    } else {
        server
    }.launch();
}
