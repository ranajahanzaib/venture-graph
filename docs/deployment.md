# Deployment

Build the API with the provided Dockerfile and deploy PostgreSQL with persistent storage. The API applies database migrations at startup. Set a private `DATABASE_URL`, `API_ADDR`, and explicit `CORS_ORIGINS` allow-list before public deployment. The local defaults allow only the Compose web origin and the Vite development origin. The web app can be built with `pnpm build` and served as static files with `VITE_API_URL` set at build time.

Production deployments require deployment-specific configuration; this repository does not include a production deployment profile. Public deployments also need authentication and authorization for administrative operations, TLS termination, rate limiting, backup and restore procedures, and operational monitoring.
