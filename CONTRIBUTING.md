# Contributing to DropX

Thanks for helping. This file explains how to report problems and how to send a change.

## Report a bug or ask for a feature

Open an issue with one of the [issue templates](https://github.com/ingvarch/dropx/issues/new/choose). For a bug, include the request you sent (a `curl` command is best), the response you got and the response you expected.

Do not report security problems in public issues. See [SECURITY.md](SECURITY.md).

## Set up

Local setup, the database migrations and `wrangler dev` are described in [docs/development.md](docs/development.md). How the parts fit together is in [docs/architecture.md](docs/architecture.md).

## Make a change

1. Create a branch from `main` named `<type>/<short-name>`, for example `fix/inline-download-limit`.
2. Write a failing test that describes the behavior you want, then the code that makes it pass. Changes that alter behavior without a test are not merged.
3. Run the [checks](docs/development.md#checks). The output must have no warnings and no failures. Behavior that goes through HTTP is tested in `tests/e2e/`.

4. Update the files in `docs/` if the change alters what users or operators see.
5. Commit with a [conventional commit](https://www.conventionalcommits.org/) message: `type(scope): description`, imperative, lower case.
   - Types: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `chore`.
   - Scopes: `api`, `storage`, `rate-limit`, `auth`, `admin`, `cleanup`, `ui`, `build`, `ci`.
6. Open a pull request and fill in the template. Link the issue it closes with `Closes #N`.

Pull requests are merged with rebase or squash, so `main` keeps a linear history.

The coding rules for this project (testing, worker-rs patterns, security) are in [AGENTS.md](AGENTS.md).

## Code of conduct

Everyone taking part follows the [Code of Conduct](CODE_OF_CONDUCT.md).

## License

By contributing, you agree that your contributions are licensed under the [MIT License](LICENSE).
