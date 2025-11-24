# DAW Backend (Rust + Actix-Web)

## Overview
Ride-hailing backend with multi-tenant support, Redis GEO, Surge engine, PostgreSQL (SeaORM), and WebSocket real-time updates.

## Features
- Multi-tenant users (rider/driver)
- Driver online/offline + GEO updates
- Ride lifecycle: request → dispatch → accept/reject → start → complete
- WebSocket notifications
- Background jobs (Qrush)
- Event-store (`driver_event`, `ride_event`)


## Env
```
APP_SERVER_HOST="0.0.0.0"
APP_SERVER_PORT="8080"
APP_DATABASE_URL="postgres://username:password@localhost:5432/databasename"
APP_ENVIRONMENT="development"
APP_RUST_LOG="actix_web=debug,info"
APP_REDIS_URL="redis://localhost:6379"
JWT_SECRET="some-long-random-string-change-me"
REDIS_URL="redis://localhost:6379"
QRUSH_BASIC_AUTH="qrush:password"
PUSHER_URL="http://<>:<>@api-<>.pusher.com/apps/<>"
CORS_ALLOWED_ORIGINS="http://localhost:3000,http://127.0.0.1:3000"
```


## Test
```
cargo test
```

## Build
```
cargo build
```

## Run
```
cargo run
```

## Health Check
```
http://localhost:8080
```

## Endpoints
```
/api/v1/
```

