FROM ghcr.io/musergi/sqlx:latest

CMD ["sqlx", "database", "setup"]
