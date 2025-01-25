# Prerequesites:

- OS: Linux or MacOS
- TiUP: `curl --proto '=https' --tlsv1.2 -sSf https://tiup-mirrors.pingcap.com/install.sh | sh` and add `~/.tiup/bin` to your PATH
- Install my [surreal db server](https://github.com/MarlinZapp/wishes-surreal-server)

# Generate autocompletion

- `testing generate > testing.my_shell`
- `source testing.my_shell`

# Run tests

- Get help: `testing --help`, `testing case --help`,...
- `testing case -e ../path/to/surreal_server case_number [opt_args]`
