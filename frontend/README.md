# frontend

The frontend for a Rext demo project.

[Rext](https://rextstack.org), the fullstack, rust powered, batteries-included framework.

## What is this?

This is a simple demo project that showcases all the features Rext *will* incorporate. This project was *not* created by Rext, but it is a good example of what Rext *will* be able to do.

## Features

- **Database** - a sqlite database for getting up and running quickly. Suport for MySQL, Postgresql, and sqlite will be goals.
- **ORM** - an object relational model for accessing the database via [sea-orm](https://www.sea-ql.org/SeaORM/)
- **Web Server** - powered by [axum](https://crates.io/crates/axum)
- **Routing** - powered by the web server
- **Authentication** - simple yet secure authentication with [argon2](https://crates.io/crates/argon2) and [jsonwebtoken](https://crates.io/crates/jsonwebtoken).

## Project Setup

```sh
npm install
```

### Compile and Hot-Reload for Development

```sh
npm run dev
```

### Type-Check, Compile and Minify for Production

```sh
npm run build
```

### Run Unit Tests with [Vitest](https://vitest.dev/)

```sh
npm run test:unit
```

### Run End-to-End Tests with [Playwright](https://playwright.dev)

```sh
# Install browsers for the first run
npx playwright install

# When testing on CI, must build the project first
npm run build

# Runs the end-to-end tests
npm run test:e2e
# Runs the tests only on Chromium
npm run test:e2e -- --project=chromium
# Runs the tests of a specific file
npm run test:e2e -- tests/example.spec.ts
# Runs the tests in debug mode
npm run test:e2e -- --debug
```

### Lint with [ESLint](https://eslint.org/)

```sh
npm run lint
```
