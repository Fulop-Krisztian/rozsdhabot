# rozsdhabot

Hatalmas és elsőszámú inspiráció a [PYHABOT](https://github.com/Patrick2562/PYHABOT). Adj rá egy csillagot.

A rozsdhabot egy web scraper ami megadott hardverapró kereséseket figyel, és értesítést küld ha egy új hirdetés kerül fel.
A jövőben majd több platformon is tud majd értesíteni, jelenleg csak a Telegram támogatott, a Discord folyamatban van.

# Konfiguráció

A konfigurációt környezeti változókkal lehet megadni a [docker-compose.yaml](docker-compose.yaml) fájlban.

- Az egyes integrációkat külön-külön lehet bekapcsolni.
- Egyszerre több integrációt is be lehet kapcsolni.
- Ha valamelyik integráció nincs bekapcsolva, akkor a hozzátartozó tokent nem kell megadni

## Telegram

- Kapcsold be a Telegram integrációt a docker-compose fájlban (INTEGRATIONS = ...,telegram)
- Hozz létre egy új botot a [BotFather](https://t.me/BotFather) csatornán. (Itt tudod majd a botod kinézetét és display nevét is állítani)
- A létrehozott bot tokenjét másold be a docker-compose fájlba (TELEGRAM_TOKEN = token)

Ahhoz, hogy a bot Telegramon tudjon kommunikálni, először üzenni kell neki (pl. /start). Ezt a telegram szabályozza.

Ha új botot csinálsz, akkor annak a nem lesz joga ugyan oda üzenni, mint a réginek

## Discord

WIP...

# Futtatás

A bothoz először is (platformtól függően) létre kell hozni egy botot, és a bot tokent majd meg kell adni.

## Docker

A botot legegyszerűbben docker compose-t használva lehet elindítani. A repo klónozása után:

```bash
docker-compose up -d
```

Ha minden rendben ment és helyes a konfiguráció, akkor a bot készen áll fogadni a kéréseket.

```bash
docker-compose up --build -d
```

# Használat

Az elérhető parancsok a `/help` parancs segítségével kérhetők le.

A legfontosabb az `/add` parancs, aminek segítségével hozzáadhatsz egy figyelendő keresést. Pl. `/add "https://hardverapro.hu/aprok/hardver/videokartya/amd_ati/rx_6000/keres.php?stext=RX+6700+XT&stcid_text=&stcid=&stmid_text=&stmid=&minprice=&maxprice=90000&cmpid_text=&cmpid=&usrid_text=&usrid=&__buying=1&__buying=0&stext_none=&noiced=1&__brandnew=1&__brandnew=0"` (ezt az URL-t csak simán ki lehet másolni egy hardverapro keresésből), hogy a rozsdhabot egy videókártyás keresést figyeljen. Hozzáadásokor első találatokat nem fogja listázni, hogy ne spamelje szét a csatornát. Ha viszont felkerül valami új, akkor egy percen belül tudni fogsz róla.
