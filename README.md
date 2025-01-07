# Project Executor (px)

A unified way of running user defined scripts for any language.

Run scripts with `px SCRIPT_NAME`.

## Installation

- Download archive for your platform from the [latest release](https://github.com/theboxer/px/releases/latest)
- Symlink it to `/usr/local/bin` or your favourite alternative

### Mac OS

When running it for the first time, it'll show an error message. You'll have to enable the `px` from `System Settings` -> `Privacy & Security` -> Near bottom, you'll see `Allow px`.

## Defining user scripts

- In a project's root create a config file `px.json` or `px.toml`
- User defined scripts are defined under `scripts` in the config file

### Short example

```json
{
  "scripts": {
    "run": "cargo run"
  }
}
```

### Long example

```json
{
  "scripts": {
    "run": {
      "cmd": "echo \"TEST\"",
      "description": "Output test message"
    }
  }
}
```

## Built-in support for languages

PX can parse other config files, which let's you use the default way of defining user scripts for given language, while using `px` as the unified runner.

PX parses known config files in following order: `px.json`, `px.toml`, `package.json`, `Cargo.toml`, `composer.json`. Discovered scripts are merged into all scripts available through `px` for execution. Script name defined sooner will take a priority (defining script `run` in `px.json` and in `Cargo.toml` will execute only the one from `px.json`).

### package.json

If `package.json` is detected, scripts are parsed from the `scripts` object are made available for execution via `px`. If you're using different package manager than `npm`, make sure to fill the `packageManager` property in the `package.json` to tell `px` how to execute your scripts. the are executed as `npm run SCRIPT_NAME` / `yarn run SCRIPT_NAME` / `pnpm run SCRIPT_NAME`.

### Cargo.toml

Scripts are read from `package.metadata.scripts` table from `Cargo.toml`. You can use the short or long syntax as in the example above.

### composer.json

If `composer.json` is detected, scripts are parsed from the `scripts` object and executed via `composer run-script SCRIPT_NAME`.

## Executors

Scripts defined in different files are executed through different commands.

For example, script defined in `composer.json` will execute as `composer run-script SCRIPT_NAME`, script defined in `package.json` will execute as `npm run SCIPRT_NAME` and script defined in `px.json` will execute directly the command.

`composer run-script` and `npm run` is called executor and can be adjusted, if needed.

### Default executors

```json
{
  "executor": {
    "composer": "composer run-script",
    "npm": "npm run",
    "pnpm": "pnpm run",
    "yarn": "yarn run"
  }
}
```

### Adjusting executors

Executor can be adjusted either from the `px.json`/`px.toml` or from the language specific config file.

#### PX file

Example of adjusting `composer` executor:

```json
{
  "executor": {
    "composer": "ddev composer run-script"
  }
}
```

### Language config file

When adjusting the executor from config file like `package.json` or `composer.json`, you can only adjust the executor specific for this file and it's done via `px.executor` property.

For example, when using `ddev` in the PHP project, you'll want to adjust the executor in the `composer.json` as follows:

```json
{
  "px": {
    "executor": "ddev composer run-script"
  }
}
```

This will ensure that scripts will run as `ddev composer run-script SCRIPT_NAME` instead of `composer run-script SCRIPT_NAME`.
