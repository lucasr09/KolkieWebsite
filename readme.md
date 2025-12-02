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
- **HTML/CSS/JS** (frontend)
- Geen frameworks nodig voor fase 1

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



