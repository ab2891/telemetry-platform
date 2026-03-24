FROM node:20-bookworm-slim

WORKDIR /app

# Copy dependency manifests first for caching
COPY package.json package-lock.json* pnpm-lock.yaml* yarn.lock* ./

# Install dependencies (choose one package manager)
RUN npm ci

# Copy the rest of the app
COPY . .

EXPOSE 5173

# IMPORTANT: bind to 0.0.0.0 so it's reachable outside the container
CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0", "--port", "5173"]