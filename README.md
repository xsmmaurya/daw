# DAW Backend (Rust + Actix-Web)

## Overview
Ride-hailing backend with multi-tenant support, Redis GEO, Surge engine, PostgreSQL (SeaORM), and WebSocket real-time updates.



## Features

### ✅ Authentication & Tenant Context
- JWT-based auth  
- Tenant scoping at DB level  
- `get_current_user()` auto-resolves session context

### ✅ Drivers
- `/drivers/online` — go online + record event  
- `/drivers/offline` — go offline + record event  
- `/drivers/location` — update location  
- Redis GEO integration for proximity search  
- Event logging in `driver_event` table  

### ✅ Riders & Rides
- `/rides/request` — request a ride  
- Surge pricing using demand/supply keys  
- `/rides/{id}/accept`, `/start`, `/complete`  
- Ride event timeline saved in `ride_event` table  

### ✅ Dispatch System
- Pushes jobs to Qrush (dispatch_ride_job)  
- Selects nearest driver via Redis GEO  
- WebSocket notifications to driver & rider  

### ✅ Event APIs
```
GET /events/drivers/{id}/events
GET /events/rides/{id}/events
```
Paginated, sorted by created_at.

---

## Database Schema (Core Tables)
- `user`
- `tenant`
- `driver`
- `ride`
- `driver_event`
- `ride_event`
- `seaql_migrations`


## WebSocket Events

### Rider events
- `ride_assigned`
- `ride_accepted`
- `ride_started`
- `ride_completed`
- `ride_rejected_by_driver`

### Driver events
- `ride_assigned_to_driver`
- `ride_accepted_for_driver`
- `ride_started_for_driver`
- `ride_completed_for_driver`



## Setup

### 1. Environment Variables
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
