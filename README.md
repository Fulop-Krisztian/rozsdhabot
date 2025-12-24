# rozsdhabot

Hatalmas és elsőszámú inspiráció a [PYHABOT](https://github.com/Patrick2562/PYHABOT). Adj rá egy csillagot.

A rozsdhabot egy web scraper ami megadott hardverapró kereséseket figyel, és értesítést küld ha egy új kerül fel.
A jövőben majd több platformon is tud majd értesíteni, jelenleg csak a Telegram támogatott, de a Discordon már dolgozok.

# Futtatás

## Docker

Beállításokat (környezeti változókat) a [docker-compose.yaml](docker-compose.yaml) fájlban lehet megadni.

A botot legegyszerűbben docker compose-t használva lehet elindítani. A repo klónozása után:
Először lassú lesz az indítás, mert buildelni kell az image-t.

```bash
docker-compose up -d
```

Ha minden rendben ment és helyes a konfiguráció, akkor a bot készen áll fogadni a kéréseket.

Esetleges frissítés után újra kell buildelni a docker image-et, ebben az esetben:

```bash
docker-compose up --build -d
```

# Használat

Az elérhető parancsok a [help.txt](help.txt) fájlban találhatók.
