# Multi-stage build for SvelteKit (static client + node server fallback)
FROM node:20-alpine AS builder
WORKDIR /app

# Install deps (use package-lock if present)
COPY package*.json ./
RUN npm install --no-audit --no-fund

# Copy sources and build
COPY . .
RUN npm run build

# Use a lightweight nginx to serve the client build output
FROM nginx:alpine AS runner
COPY --from=builder /app/.svelte-kit/output/client /usr/share/nginx/html
COPY --from=builder /app/static /usr/share/nginx/html

# Default nginx port
EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
# Use official Node.js image as the base
FROM node:20-alpine AS build

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .
RUN npm run build

# Use nginx to serve the built frontend
FROM nginx:alpine

COPY --from=build /app/dist /usr/share/nginx/html
COPY static/ /usr/share/nginx/html

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
