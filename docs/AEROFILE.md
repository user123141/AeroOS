# Aerofile

Декларативный файл конфигурации VM для AeroOS. Аналог Dockerfile / Vagrantfile.

## Быстрый старт

    aeroctl init myvm

Создаст `Aerofile` в текущей директории.

## Формат

```toml
[vm]
name = "dev"
image = "alpine-3.20"
cores = 4
ram = 4096
disk_mb = 2048

[network]
mode = "user"   # "user" | "tap" | "none"

[provision]
run = [
  "apk add --no-cache nodejs npm",
  "npm install -g pnpm",
]

[[share]]
host = "C:\\Projects"
guest = "/mnt/projects"
```

## Поля

| Секция | Поле | Обязательно | Описание |
|--------|------|:-----------:|----------|
| vm | name | да | Имя VM |
| vm | image | нет | Образ (alpine-3.20, ubuntu-24.04, ...) |
| vm | cores | нет | Ядер CPU (1..64), default 2 |
| vm | ram | нет | RAM в МБ (128..262144), default 2048 |
| vm | disk_mb | нет | Размер диска в МБ, default 1024 |
| network | mode | нет | `user` (default) / `tap` / `none` |
| provision | run | нет | Команды после загрузки |
| share | host | да | Путь на Windows |
| share | guest | да | Точка монтирования в Linux |

## Валидация

`Aerofile::load()` автоматически проверяет:
- name не пустое
- cores ∈ [1, 64]
- ram ∈ [128, 262144]
- network.mode ∈ {user, tap, none}

## Запуск

    aeroctl up --file Aerofile

## Roadmap

- [ ] Pull образа из AeroRegistry
- [ ] Кэширование провиженинга
- [ ] `aeroctl build` — pre-provision образ
- [ ] Multi-stage builds