# Multi-stage build for SvelteKit (static client + node server fallback)
FROM node:20-bookworm-slim AS builder
WORKDIR /app

# Install deps (use package-lock if present)
COPY package*.json ./
RUN npm install --no-audit --no-fund

# Copy sources and build
COPY . .
RUN npm run build

# Use a lightweight nginx to serve the client build output
FROM nginx:alpine AS runner
COPY --from=builder /app/.svelte-kit/output/server /usr/share/nginx/html
COPY --from=builder /app/static /usr/share/nginx/html

# Default nginx port
EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
