# InspiroMate

A companion server to render InspiroFrames.

## Example `docker-compose.tml`

```yaml
version: '3.8'

services:
  inspiro-mate:
    build: .
    container_name: inspiro-mate
    ports:
      - 8000:8000
    cap_add:
      - SYS_ADMIN
    restart: unless-stopped
```
