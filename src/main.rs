#[macro_use]
extern crate rocket;

use askama::Template;
use chrono::{Datelike, Local, Weekday};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::form::Form;
use rocket::fs::{FileServer, NamedFile};
use rocket::http::Header;
use rocket::response::content::RawHtml;
use rocket::response::Redirect;
use rocket::{Request, Response, State};
use std::collections::HashMap;
use std::env;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

//hier staan de openingstijden
fn openingstijd_vandaag() -> &'static str {
    match Local::now().weekday() {
        Weekday::Mon => "11:30 – 20:30", //wil je hier iets aanpassen? bijvoorbeeld zaterdag tot 21:00 open in plaats van 20:30?
        Weekday::Tue => "Gesloten",     //of op dinsdag open? dan kan je tussen de dubbelepunten "" je openingstijden zetten.
        Weekday::Wed => "11:30 – 20:30", //bijvoorbeeld: weekday::Wed => "11:30 - 21:00",
        Weekday::Thu => "11:30 – 20:30",
        Weekday::Fri => "11:30 – 20:30",
        Weekday::Sat => "11:30 – 20:30",
        Weekday::Sun => "11:30 – 20:30",
    }
}

// Foto's voor de "Populair bij Kolkie" slider.
const SLIDER_IMAGES: [&str; 17] = [
    "12Uurtje_tall.jpg", "12Uurtje2.jpg", "Appeltaart.jpg", "BroodjeFilet.jpg",
    "BroodjeGezond.jpg", "BroodjeGezond2.jpg", "BroodjeHeteKip.jpg", "BroodjeHeteKip2.jpg",
    "Burger.jpg", "FrietjeMet.jpg", "KidsBox.jpg", "Kipburger.jpg", "Kipburger2.jpg",
    "Kipburger3.jpg", "Koffie.jpg", "KoffieBakkerij.jpg", "Tosti.jpg",
];

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    vandaag_open: &'a str,
    contact_success: bool,
    contact_error: bool,
    // De lijst staat er twee keer in: de CSS-animatie schuift precies -50% en
    // sluit dan naadloos aan op de tweede set, zonder dat er JS aan te pas komt.
    slider_images: Vec<&'static str>,
}

#[derive(FromForm)]
struct ContactForm {
    name: String,
    email: String,
    phone: String,
    message: String,
    // Honeypot-veld: onzichtbaar voor mensen, spambots vullen het vaak toch in.
    website: String,
}

fn contact_form_is_valid(form: &ContactForm) -> bool {
    let name = form.name.trim();
    let email = form.email.trim();
    let message = form.message.trim();

    // "name" komt terecht in de e-mail-Subject: geen controletekens (\r, \n)
    // toestaan, anders kan iemand daarmee extra e-mailheaders injecteren
    // (bv. een eigen Bcc:) via het naam-veld.
    !name.is_empty()
        && name.chars().count() <= 200
        && !name.chars().any(|c| c.is_control())
        && email.contains('@')
        && !email.contains(' ')
        && email.chars().count() <= 320
        && !email.chars().any(|c| c.is_control())
        && !message.is_empty()
        && message.chars().count() <= 5000
}

// Simpele in-memory rate limiter per IP-adres, zodat het contactformulier niet
// eindeloos gespamd kan worden. Geen externe dependency, geen database.
struct RateLimiter {
    hits: Mutex<HashMap<IpAddr, Vec<Instant>>>,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            hits: Mutex::new(HashMap::new()),
        }
    }

    /// Sta maximaal MAX inzendingen per WINDOW toe per IP-adres.
    fn allow(&self, ip: IpAddr) -> bool {
        const MAX: usize = 5;
        const WINDOW: Duration = Duration::from_secs(600); // 10 minuten

        let now = Instant::now();
        let mut map = self.hits.lock().unwrap();

        // Houd de tabel klein: gooi verlopen tijdstippen en lege IP's weg.
        map.retain(|_, times| {
            times.retain(|t| now.duration_since(*t) < WINDOW);
            !times.is_empty()
        });

        let times = map.entry(ip).or_default();
        if times.len() >= MAX {
            return false;
        }
        times.push(now);
        true
    }
}

// Response-fairing die op elke respons de beveiligingsheaders zet die Rockets
// ingebouwde Shield niet meelevert (CSP + Referrer-Policy).
struct SecurityHeaders;

#[rocket::async_trait]
impl Fairing for SecurityHeaders {
    fn info(&self) -> Info {
        Info {
            name: "Security headers",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _req: &'r Request<'_>, res: &mut Response<'r>) {
        res.set_header(Header::new(
            "Content-Security-Policy",
            "default-src 'self'; \
             base-uri 'self'; \
             form-action 'self'; \
             frame-ancestors 'self'; \
             object-src 'none'; \
             img-src 'self' data:; \
             style-src 'self'; \
             script-src 'self'; \
             font-src 'self'; \
             connect-src 'self'; \
             frame-src https://www.google.com",
        ));
        res.set_header(Header::new(
            "Referrer-Policy",
            "strict-origin-when-cross-origin",
        ));
    }
}

fn send_contact_email(form: &ContactForm) -> Result<(), String> {
    let smtp_host = env::var("SMTP_HOST").map_err(|_| "SMTP_HOST ontbreekt".to_string())?;
    let smtp_port: u16 = env::var("SMTP_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(587);
    let smtp_username =
        env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME ontbreekt".to_string())?;
    let smtp_password =
        env::var("SMTP_PASSWORD").map_err(|_| "SMTP_PASSWORD ontbreekt".to_string())?;
    let from_address = env::var("SMTP_FROM").unwrap_or_else(|_| smtp_username.clone());
    let to_address = env::var("CONTACT_TO_EMAIL")
        .unwrap_or_else(|_| "lucasrensen@outlook.com".to_string());

    let phone = if form.phone.trim().is_empty() {
        "Niet opgegeven".to_string()
    } else {
        form.phone.trim().to_string()
    };

    let body = format!(
        "Nieuw bericht via het contactformulier op de website.\n\n\
        Naam: {}\n\
        E-mail: {}\n\
        Telefoon: {}\n\n\
        Bericht:\n{}\n",
        form.name.trim(),
        form.email.trim(),
        phone,
        form.message.trim()
    );

    let email = Message::builder()
        .from(
            from_address
                .parse()
                .map_err(|e| format!("Ongeldig SMTP_FROM adres: {e}"))?,
        )
        .reply_to(
            form.email
                .trim()
                .parse()
                .map_err(|e| format!("Ongeldig afzender e-mailadres: {e}"))?,
        )
        .to(to_address
            .parse()
            .map_err(|e| format!("Ongeldig CONTACT_TO_EMAIL adres: {e}"))?)
        .subject(format!("Nieuw bericht via de website van {}", form.name.trim()))
        .header(ContentType::TEXT_PLAIN)
        .body(body)
        .map_err(|e| format!("Kon e-mail niet opbouwen: {e}"))?;

    let credentials = Credentials::new(smtp_username, smtp_password);

    let mailer = if smtp_port == 465 {
        SmtpTransport::relay(&smtp_host)
    } else {
        SmtpTransport::starttls_relay(&smtp_host)
    }
    .map_err(|e| format!("Kon geen verbinding opzetten met {smtp_host}: {e}"))?
    .port(smtp_port)
    .credentials(credentials)
    .build();

    mailer
        .send(&email)
        .map_err(|e| format!("Versturen via SMTP mislukt: {e}"))?;

    Ok(())
}

#[get("/?<contact>")]
fn index(contact: Option<&str>) -> RawHtml<String> {
    let vandaag_open = openingstijd_vandaag();

    let template = IndexTemplate {
        vandaag_open,
        contact_success: contact == Some("success"),
        contact_error: contact == Some("error"),
        slider_images: SLIDER_IMAGES.iter().chain(SLIDER_IMAGES.iter()).copied().collect(),
    };

    RawHtml(template.render().unwrap_or_else(|e| {
        // Interne fouten (bv. een kapotte template) loggen we server-side, maar
        // geven we niet letterlijk terug aan de bezoeker.
        eprintln!("Kon index-pagina niet renderen: {e}");
        "Er ging iets mis bij het laden van deze pagina. Probeer het zo nog eens.".to_string()
    }))
}

#[post("/send-message", data = "<form>")]
async fn send_message(
    form: Form<ContactForm>,
    limiter: &State<RateLimiter>,
    client_ip: Option<IpAddr>,
) -> Redirect {
    let form = form.into_inner();

    // Bot gedetecteerd via honeypot: doe alsof het gelukt is, verstuur niets.
    if !form.website.trim().is_empty() {
        return Redirect::to("/?contact=success#contact");
    }

    // Rate limit per IP. Onbekend IP valt terug op een gedeelde bucket.
    let ip = client_ip.unwrap_or(IpAddr::from([0, 0, 0, 0]));
    if !limiter.allow(ip) {
        eprintln!("Contactformulier: rate limit bereikt voor {ip}");
        return Redirect::to("/?contact=error#contact");
    }

    if !contact_form_is_valid(&form) {
        return Redirect::to("/?contact=error#contact");
    }

    let result = rocket::tokio::task::spawn_blocking(move || send_contact_email(&form)).await;

    match result {
        Ok(Ok(())) => Redirect::to("/?contact=success#contact"),
        Ok(Err(e)) => {
            eprintln!("Contactformulier: versturen van e-mail mislukt: {e}");
            Redirect::to("/?contact=error#contact")
        }
        Err(e) => {
            eprintln!("Contactformulier: interne fout tijdens versturen: {e}");
            Redirect::to("/?contact=error#contact")
        }
    }
}

#[get("/favicon.ico")]
async fn favicon() -> Option<NamedFile> {
    NamedFile::open(Path::new("public/favicon.ico")).await.ok()
}

#[get("/robots.txt")]
async fn robots() -> Option<NamedFile> {
    NamedFile::open(Path::new("public/robots.txt")).await.ok()
}

#[catch(404)]
fn not_found() -> RawHtml<&'static str> {
    RawHtml(include_str!("../templates/404.html"))
}

#[catch(500)]
fn server_error() -> RawHtml<&'static str> {
    RawHtml(include_str!("../templates/500.html"))
}

#[catch(default)]
fn other_error() -> RawHtml<&'static str> {
    RawHtml(include_str!("../templates/500.html"))
}

#[launch]
fn rocket() -> _ {
    dotenvy::dotenv().ok();

    rocket::build()
        .manage(RateLimiter::new())
        .attach(SecurityHeaders)
        .mount("/", routes![index, send_message, favicon, robots])
        .mount("/public", FileServer::from("public")) // serveert JS, CSS, fonts en images
        .register("/", catchers![not_found, server_error, other_error])
}
