# SDD Navigator Service


### Требования
- Rust 1.89+
- Docker & docker-compose
- PostgreSQL

### 1. Локальный запуск

```bash
# Клонирование и сборка
git clone <repository-url>
cd "SDD Navigator Service"
cargo build --release

# Запуск сервиса
cargo run
```

Сервис будет доступен по адресу `http://localhost:3000`

### 2. Запуск с Docker

```bash
# Запуск полного стека (API + PostgreSQL + pgAdmin)
docker-compose up -d

# Проверка статуса
docker-compose ps

# Просмотр логов
docker-compose logs -f sdd-navigator-service
```

**Доступные сервисы**:
- **API**: http://localhost:3000
- **Swagger UI**: http://localhost:3000/swagger-ui
- **pgAdmin**: http://localhost:8080
- **PostgreSQL**: localhost:5432

## API Эндпоинты

### Сканирование
- `POST http://localhost:3000/api/v1/scans` - Запустить новое сканирование
- `GET http://localhost:3000/api/v1/scans` - Получить список всех сканирований
- `GET http://localhost:3000/api/v1/scans/{scan_id}` - Получить детали сканирования
- `DELETE http://localhost:3000/api/v1/scans/{scan_id}` - Удалить сканирование

### Покрытие
- `GET http://localhost:3000/api/v1/scans/{scan_id}/coverage` - Метрики покрытия
- `GET http://localhost:3000/api/v1/scans/{scan_id}/coverage/report` - Полный отчет

### Health Check
- `GET http://localhost:3000/health` - Проверка состояния сервиса
