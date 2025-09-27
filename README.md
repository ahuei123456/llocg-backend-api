# LLOCG Backend API

This is a backend API for a Love Live! Official Card Game (LLOCG) database, built with Rust, Axum, and SQLx.

## Prerequisites

Before you begin, you'll need to have the following installed:

*   **Rust**: If you don't have it, you can install it via [rustup](https://rustup.rs/).
*   **`sqlx-cli`**: A command-line tool for managing database migrations with SQLx.

    ```sh
    cargo install sqlx-cli --no-default-features --features rustls,sqlite
    ```

## Local Setup

Follow these steps to get the application running on your local machine.

### 1. Clone the Repository

```sh
git clone <repository-url>
cd llocg-backend-api
```

### 2. Create the Environment File

The application requires a `.env` file for configuration.

1.  Create a new file named `.env` in the root of the project.
2.  Add the following line to it. This defines the location of the SQLite database file.

    ```env
    DATABASE_URL="sqlite:llocg.db"
    ```

### 3. Set Up the Database

Use `sqlx-cli` to create the database and run the migrations.

```sh
# Creates the database file (e.g., llocg.db)
sqlx database create

# Applies the migrations from the `migrations/` directory
sqlx migrate run
```

### 4. Run the Application

You can now build and run the server.

```sh
cargo run
```

The API will be available at `http://127.0.0.1:3000`.

## Running Tests

To run the test suite, which uses an in-memory SQLite database, use the following command:

```sh
cargo test
```