# The Cooler StreakSaver
This project exits to say "Fuck you taine". It will run through a duolingo course, attempting to complete
it. All questions and answers will be persisted to a database.

# Configuration
## Features
- `default`: `sqlite`
- `sqlite`: Build with support for the SQLite database
- `mysql`: Build with support for the MySql database
- `postgres`: Build with support for the Postgres database

## Environment variables
### Database
- `DATABASE_URL`: The database URL
### Web driver
- `DRIVER_URL`: The URL that the chrome driver is using, defaults to `http://localhost:4444`
- `CHROME_PATH`: The path to the Google Chrome executable
- `HEADLESS`: Enables headless mode in the browser for use in things like docker
### Logging
- `RUST_LOG`: Configure logging for the application