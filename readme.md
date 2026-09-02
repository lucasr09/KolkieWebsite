# Kolkie Website (Rocket + Static Frontend)

Dit project is een redesign van de website van **Cafetaria/Lunchroom Kolkie (Dieren)**.  
De focus ligt op een moderne, mooie en volledig **responsive** website.  
De huidige menukaart blijft voorlopig beschikbaar als **PDF**, zodat de originele content behouden blijft.

## Features
- Moderne homepage met duidelijke call-to-actions
- Responsive layout (mobile-first)
- Foto’s en “Populair bij Kolkie” sectie
- Openingstijden, adres en contactinformatie
- Menukaart opent nog steeds via PDF (`menu.pdf`)
- Rocket serveert alle bestanden als static site

## Tech stack
- **Rust** + **Rocket** (webserver)
- **Askama** (compile-time gecheckte HTML-templates, i.p.v. Tera)
- **lettre** (verzenden van contactformulier-mails via SMTP)
- **HTML/CSS/JS** (frontend)
- Geen frameworks nodig voor fase 1

## Contactformulier / e-mail configureren
Het contactformulier op `/send-message` verstuurt een e-mail via SMTP naar het adres
in `CONTACT_TO_EMAIL`. Dit moet je zelf configureren via environment variables (lokaal
via een `.env` bestand, in productie via echte env vars van je hostingprovider):

1. Kopieer `.env.example` naar `.env`.
2. Maak een gratis account bij een transactional e-mail service (bv.
   [Brevo](https://www.brevo.com), Resend of Mailgun) en maak daar SMTP-credentials aan.
   Gebruik **niet** je eigen Outlook-wachtwoord.
3. Vul `SMTP_HOST`, `SMTP_PORT`, `SMTP_USERNAME`, `SMTP_PASSWORD` en `SMTP_FROM` in
   `.env` in met de gegevens van die dienst.
4. `CONTACT_TO_EMAIL` staat standaard op `lucasrensen@outlook.com`.

Zolang SMTP niet geconfigureerd is, toont het formulier netjes een foutmelding
("Er ging iets mis...") in plaats van te crashen.

## Beveiliging

- **Rate limiting**: max. 5 formulierinzendingen per 10 minuten per IP-adres
  (in-memory, geen database nodig).
  ⚠️ **Let op bij deployen achter een reverse proxy** (nginx, Caddy,
  Cloudflare, een PaaS-load balancer, etc.): Rocket ziet dan standaard het
  IP-adres van de proxy, niet van de echte bezoeker, waardoor alle bezoekers
  in dezelfde emmer vallen en de limiter zijn nut verliest. Configureer in
  dat geval een `Rocket.toml` met de juiste `ip_header` (bv.
  `X-Forwarded-For`) die past bij jouw proxy - zie de
  [Rocket-configuratiedocs](https://rocket.rs/guide/v0.5/configuration/#configuration).
  Draait de site direct op het internet zonder proxy ertussen? Dan is de
  huidige instelling (geen `ip_header`) juist de veilige default.
- **Contactformulier**: honeypot-veld tegen bots, server-side validatie
  (lengtes, verplichte velden, geen controletekens in naam/e-mail om
  e-mailheader-injectie te voorkomen).
- **HTTP-headers**: Content-Security-Policy en Referrer-Policy via een eigen
  fairing; X-Frame-Options en X-Content-Type-Options komen van Rockets
  ingebouwde Shield-fairing.
- **Secrets**: SMTP-gegevens staan alleen in `.env` (genegeerd door git),
  nooit hardcoded in de broncode.

## Installatie
1. Zorg dat je **Rust** hebt geïnstalleerd:  
    https://rustup.rs

2. Clone/download dit project en ga naar de map:
```bash
cd kolkie-site
```

## Applicatie starten

```bash
cargo run
```

1. Rocket start op het standaard domein http://localhost:8000. Open die link in je browser om de website te bekijken.

2. Voor een aangepaste poort:
```bash
ROCKET_PORT=8080 cargo run
```

3. Dan wordt de site beschikbaar op http://localhost:8080.

4. Voor een snellere build:
```bash
cargo run --release
```



