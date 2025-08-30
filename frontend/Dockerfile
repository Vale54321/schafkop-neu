######## Combined Frontend + Backend build (single Node runtime) ########
# This Dockerfile now builds BOTH the Svelte frontend and the NestJS backend
# and serves the compiled frontend through the backend (Express static).

########################
# 1) Frontend build     #
########################
FROM node:20-bookworm-slim AS frontend-build
WORKDIR /frontend
COPY frontend/package*.json ./
RUN npm install --no-audit --no-fund
COPY frontend/ .
RUN npm run build

########################
# 2) Backend build      #
########################
FROM node:20-bookworm-slim AS backend-build
WORKDIR /app
COPY backend/package*.json ./
RUN npm install --no-audit --no-fund
COPY backend/ .
# Copy compiled frontend into backend/public (served statically by Nest)
COPY --from=frontend-build /frontend/dist ./public
RUN npm run build

########################
# 3) Production image   #
########################
FROM node:20-bookworm-slim AS runner
ENV NODE_ENV=production
WORKDIR /app

# Install only production deps (reuse original package.json)
COPY backend/package*.json ./
RUN npm install --omit=dev --no-audit --no-fund

# Copy backend dist + public assets
COPY --from=backend-build /app/dist ./dist
COPY --from=backend-build /app/public ./public

# Environment (override at runtime as needed)
ENV PORT=3000 \
	SERIAL_BAUD=115200

EXPOSE 3000

CMD ["node", "dist/main.js"]
