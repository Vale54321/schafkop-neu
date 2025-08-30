######## Root Combined Frontend + Backend Build ########
## Build args (can be overridden)
ARG NODE_VERSION=20-bookworm-slim

########################
# 1) Frontend build     #
########################
FROM node:${NODE_VERSION} AS frontend-build
WORKDIR /frontend
COPY frontend/package*.json ./
RUN npm install --no-audit --no-fund
COPY frontend/ .
RUN npm run build

########################
# 2) Backend build      #
########################
FROM node:${NODE_VERSION} AS backend-build
WORKDIR /app
COPY backend/package*.json ./
RUN npm install --no-audit --no-fund
COPY backend/ .
# Copy built frontend assets
COPY --from=frontend-build /frontend/dist ./public
RUN npm run build

########################
# 3) Production image   #
########################
FROM node:${NODE_VERSION} AS runner
ENV NODE_ENV=production
WORKDIR /app

# Install production deps
COPY backend/package*.json ./
RUN npm install --omit=dev --no-audit --no-fund

# Copy backend dist + public
COPY --from=backend-build /app/dist ./dist
COPY --from=backend-build /app/public ./public

ENV PORT=3000 \
    SERIAL_BAUD=115200

EXPOSE 3000
CMD ["node", "dist/main.js"]