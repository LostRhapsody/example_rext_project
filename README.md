# Rext Demo Project

This is a simple demo project that showcases all the features Rext *will* incorporate. This project was *not* created by Rext, but it is a good example of what Rext *will* be able to do.

## Getting Started

- In `example_rext_project` run `cargo run`.
- In `example_rext_project/frontend` run `npm i && npm run dev`

This spins up a Rust server listening on port 3000 and a Vite server running on port 5173.

The Rust server has a CORS bypass that allows these two ports to communicate via HTTP on `localhost`. In a real Rext app, we'll use `letsencrypt` self-signed certificates and domains on local host so we can use HTTPS and avoid CORS issues.

## Features

- **Frontend** - Written in Vue with Vue Router
- **Database** - a sqlite database for getting up and running quickly. Support for MySQL, Postgresql, and sqlite will be goals.
- **ORM** - an object relational model for accessing the database via [sea-orm](https://www.sea-ql.org/SeaORM/)
- **Web Server** - powered by [axum](https://crates.io/crates/axum)
- **Routing** - powered by the web server
- **Authentication** - simple yet secure authentication with [argon2](https://crates.io/crates/argon2) and [jsonwebtoken](https://crates.io/crates/jsonwebtoken).
- **SPA Navigation** - reactive authentication state with seamless single-page app experience

## How to use

- Navigate to `http://localhost:5173/` in your browser to view the Demo Vue app.
- Visit `/register` to create an account.
    - Email needs to be in email format, but it doesn't need to be real. Password is hashed with `argon2` on the backend.
    - If you get network errors, be sure you're on `localhost`, not `127.0.0.1`, same thing in concept but the CORS bypass is only set up for `localhost`.
- After registering, you can login at `/login`.
    - This passes you a token that is stored in your browser.
- This should route you to `/profile`.
    - This is a protected route, you can only access it when logged in. Should redirect to `/login` if logged out.
- Logout using the "Logout" button in the navigation bar, which removes the token from your browser and updates the UI in real-time.