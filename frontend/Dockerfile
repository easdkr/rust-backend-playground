# -----------------------------------------------------------------------------
# Frontend container image for k3s / k8s deployments.
# Uses Bun to build and run the TanStack Start SSR server, with nginx in the
# same container to serve the static Vite assets and proxy everything else
# to the SSR server.
# -----------------------------------------------------------------------------

FROM oven/bun:1-alpine AS builder

WORKDIR /app

# Install dependencies first so Docker can cache this layer.
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

# Copy the rest of the source and build the production bundle.
COPY . .
ENV NODE_ENV=production
RUN bun run build

# -----------------------------------------------------------------------------
# Runtime stage
# -----------------------------------------------------------------------------

FROM oven/bun:1-alpine

WORKDIR /app

RUN apk add --no-cache nginx

ENV NODE_ENV=production
# The SSR server listens on 3001; nginx listens on 3000 and proxies to it.
ENV PORT=3001

# The SSR server bundle still resolves peer deps from node_modules at runtime,
# so we copy the production build output together with the installed packages.
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/dist ./dist
COPY --from=builder /app/node_modules ./node_modules

# Nginx configuration: static assets from dist/client, everything else proxied
# to the SSR server.
COPY nginx.conf /etc/nginx/http.d/default.conf

EXPOSE 3000

# `RUST_API_BASE_URL` must be injected at deploy time; see k8s/frontend-deployment.yaml.
CMD ["sh", "-c", "bun dist/server/server.js & nginx -g 'daemon off;'"]
