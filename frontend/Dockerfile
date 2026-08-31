# Dev-only image: runs Vite's dev server with hot reload.
FROM node:24-alpine

WORKDIR /app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .

EXPOSE 3000

# --host makes Vite listen on 0.0.0.0 so the port published in
# docker-compose.yml can actually reach it; without it Vite only
# binds to localhost *inside* the container.
CMD ["npm", "run", "dev", "--", "--host"]
