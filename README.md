# builder

- [builder](#builder)
  - [Overview](#overview)
- [Build plugins](#build-plugins)
  - [Configuration](#configuration)
  - [Assemblies](#assemblies)
- [Code generation](#code-generation)
- [Installer plugins](#installer-plugins)
  - [Installation directory](#installation-directory)
- [Githooks plugins](#githooks-plugins)
- [Writing a plugin](#writing-a-plugin)
  - [Working directory](#working-directory)
  - [Inputs](#inputs)
  - [Outputs](#outputs)

## Overview

The builder can be seen as a plugin system for cargo and git that makes it very easy to extend the
build process with your own plugins or use an existing one.

Design goals:

- Minimal
- Seamless integration with cargo and git. I.e. it works when running `cargo build` and `git commit`.
- Easy to use your own plugins for:
  - **Building** - run before or after the build of a package.
  - **Githooks** - like pre-commit, pre-push, etc.
  - **Installers** - install binaries, rust targets, etc. if not already installed or if the version is wrong.
- Code generation using templates to create rust code, html, .env files etc, using the output from the plugins.
- Ecosystem: there's already plugins for many things, and it's easy to contribute your own.
- CI friendly

# Build plugins

The **builder** handles running per-package pre and post build plugins, via the
[build.rs](https://doc.rust-lang.org/cargo/reference/build-scripts.html) which runs before
a build. Each pre-build will automatically start by running it's dependencies' post-builds,
which means that the for the top-level crate, the post-build will have to be run manually, but for all
dependencies it will run automatically.

You'll have to add the following to _build.rs_ for all your packages that uses or depends on packages that uses builder:

```rust
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match Command::new("builder").status() {
        Ok(status) if status.success() => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("Failed to run builder: {e}");
            ExitCode::FAILURE
        }
    }
}
```

## Configuration

The configuration of each pre- and post-build plugin is done package-level _Builder.toml_ files.
This is done to keep the configuration close to the source.

The table names are composed as:

`[prebuild|postbuild].<assembly?>.<target-triple?><profile?>.<plugin>.<action?>`

Where:

- `[prebuild|postbuild]` is the build phase.
- `<assembly?>` is the optional assembly name. More regarding assemblies [below](#assemblies).
- `<target-triple?>` is the optional platform [triple](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
  that is compiled for, used to run a plugin only for that platform.
- `<profile?>` is the optional [profile](https://doc.rust-lang.org/cargo/reference/profiles.html) name.
  If omitted the plugin will run for all profiles.
- `<plugin>` is the name of the plugin.
- `<action?>` is the action that the plugin should run, in case it has multiple actions. It can also be
  used to run the plugin multiple times with different configurations or when running it at both the pre- and post phases.

Example:

```toml
[prebuild.mobile.release.sass.compile]
file = "style/main.scss"
optimize = true
out = { brotli = true }
```

## Assemblies

Often when building assets different assemblies are needed, Say when building a cross-platform web-based
product, the assets for the different platforms are different, say mobile, web and desktop. This can be
handled by using assemblies.

# Code generation

**generate** is a built-in plugin used to generate files, like rust code, html, env files, etc. It uses
[handlebars](https://handlebarsjs.com/) as templating engine. It is given the input (context) the content
of _Input.yaml_ which contains all the output from the plugins that have run before it. See [Inputs](#inputs).

Configure the plugin to run for your package, build once, and take a look at the _Input.yaml_ file in
the working directory and you'll see what you can use in your templates.

Custom helpers can be written in [rhai](https://rhai.rs) script or you can extend the plugin with your own
implementation where you can define all the helpers you need.

Example configuration:

```toml
[postbuild.release.generate]
template = "src/index.html.hbs"
# Any handlebar helpers
helpers = ["src/helpers.rhai"]
generate = "gen/index.html"
```

Example _index.html.hbs_ template:

```html
<html>
  <head>
    <!-- Using the file's checksum in the URL so caching safely can be turned on -->
    <link rel="stylesheet" href="{{assets.mobile.sass.generated-url}}" />
  </head>
</html>
```

# Installer plugins

These are used to install both plugins and other binaries and dependencies that are needed by the workspace
or the packages in the workspace. You can of course create your own installers.

The builder checks if plugins and other binaries are installed and their version:

- at the first build
- after a `cargo clean`
- when running `builder install`
- when any _Cargo.toml_ or _Builder.toml_ file is changed
- when a watched file is changed (see the `watch` configuration below)

This information is cached in `target/builder/cache.yaml`). Then when a plugin is needed by a package build
or explicitly installed with `builder install`, it will be installed or upgraded if necessary.
Note that a plugin that is defined on workspace level, but not used by any package, will always be installed.

The configuration table names are composed as:

`install.<host-triple?><plugin>`

Where:

- `<host-triple?>` is the optional triple of the platform that the plugin run on.
- `<plugin>` is the name of the plugin.

The options are:

- `<cmd>: <arg>` where:

  - `<cmd>` can be:

    - `install` - uses [cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html).
    - `binstall` - uses [cargo binstall](https://docs.rs/cargo-binstall/latest/cargo_binstall/).
    - `shell` - uses a custom [command](https://doc.rust-lang.org/std/process/struct.Command.html) to install the plugin.
    - `<plugin>` - your custom installer plugin. This plugin needs to be previously installed with any of the above methods.

  - `<arg>`: are the command line arguments passed to the `<cmd>`

- `<version?>`: optional, the version that the plugin should have. A version is considered already installed if the output
  of the command with the `version-arg` contains this string. This is not needed for `install` and `binstall` as
  they read the version after the `@` in the command args.
- `<version-arg>`: is the command line argument to get the version of the plugin. Defaults to `--version`.
- `<watch>`: is a list of additional configuration files that are watched for changes.

Example:

```toml
[install.nextest]
binstall = "cargo-nextest@v0.9.72"

[install.my-prog]
version = "1.0.0"
version-cmd = "-V"
install = "curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/myuser/myrepo/main/install.sh | bash"
```

## Installation directory

The [install.root](https://doc.rust-lang.org/cargo/reference/config.html#installroot) in the cargo configuration is used
as the installation directory for the plugins. It is recommended to set it to `.builder/bin` in order to have workspace
specific versions. This folder is suitable for caching in CI.

If the `install.root` is set then `install` and `binstall` runs with the environment variable `CARGO_INSTALL_ROOT={install.root}`
and builder will first search that directory for plugin binaries, and if not found, it will use `$PATH`.

# Githooks plugins

Builder can handle setting up the git hooks for you. All you need to do is to specify which plugins to run for a hook.

Configuration:

`[[githook.<hook-name>]]` where `<hook-name>` is the name as defined in the [githook doc](https://git-scm.com/docs/githooks).

With the parameter `<plugin>` the name of the plugin to call

# Writing a plugin

Plugins are simple executables that are called by **builder**.

## Working directory

The working directories inside of `target/builder` are:

- builder: `<package>/<assembly?>/<target-triple?>/<profile?>/<plugin>/[pre|post]/<action?>`
- installer: `installer/<host-triple?>/<plugin>`
- githook: `githook/<host-triple?>/<plugin>`

Where:

- `target` is the workspace's `target` folder
- `<package>` is the name of the package
- `<assembly?>` is an optional assembly name.
- `<target-triple?>` is the optional platform [triple](https://doc.rust-lang.org/nightly/rustc/platform-support.html) that is compiled for.
- `<host-triple?>` is the optional [triple](https://doc.rust-lang.org/nightly/rustc/platform-support.html) of the build host.
- `<profile>` is the [cargo profile](https://doc.rust-lang.org/cargo/reference/profiles.html) used for the build.
  It has to be `dev` or `release` or correspond to a profile defined in the workspace's _Cargo.toml_.
- `<plugin>` is the name of the plugin that was run.
- `<action?>` is the plugin action, if any.

## Inputs

- **args**. A plugin is executed with two arguments, the phase (one of `prebuild`,
  `postbuild`, `install`, `--version`, `<githook name>`) and the path to its working directory,
  see [Working directory](#working-directory) above.
- **Input.yaml** prepared by the builder, with the contents:
  - `envs`: (build plugins only) Environment variables defined
    [here](https://doc.rust-lang.org/cargo/reference/environment-variables.html).
  - `configuration`: The content of the plugin's configuration in the _Builder.toml_ file.
  - `runtime`: The current runtime configuration, including the action invoked, profile, target, etc.
  - Content of the _Output.yaml_ from previously run plugins of the same category:
    - build plugins: `<package>.<assembly?>.<plugin>.<action?>`
    - githook plugins: `githook.<hook-name>.<plugin>`
    - install plugins: `install.<host-triple?>.<plugin>`

## Outputs

- **Output.yaml** in the working directory with any information that can be used by templates and by other plugins.
- **artifacts** any files generated by a plugin should be in:
  - for install plugins, the [installation directory](#installation-directory) or in a directory present in `$PATH`.
  - for builder plugins, it's [builder directory](#builder-directory).
- **stdout & stderr** are captured and written to a _log.txt_ file in the plugin folder. For build plugins it's also forwarded to cargo.
  That means that everything described in cargo's [Outputs of the Build Script](https://doc.rust-lang.org/cargo/reference/build-scripts.html#outputs-of-the-build-script) is also valid for the build plugins.
- **exit code** is used to determine if the plugin was successful or not.
