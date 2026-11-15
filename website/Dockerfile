# Builder stage
FROM node:25-slim AS builder

WORKDIR /app

# Add build argument
ARG VITE_API_BASE_URL
ENV VITE_API_BASE_URL=$VITE_API_BASE_URL

# Install dependencies
COPY package*.json ./
RUN npm install

# Copy source and build
COPY . .
RUN npm run build

# Runtime stage
FROM nginx:alpine

# nginx-Konfiguration als Vorlage: der Einstiegspunkt des nginx-Abbilds macht
# daraus beim Start /etc/nginx/conf.d/default.conf und setzt NGINX_LISTEN ein.
ENV NGINX_LISTEN=5173
COPY nginx.conf.template /etc/nginx/templates/default.conf.template

# Copy build artifacts from builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

# Expose port 80
EXPOSE 5173

CMD ["nginx", "-g", "daemon off;"]
