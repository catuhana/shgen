mod shweb 'shweb/Justfile'

set windows-shell := ["pwsh.exe", "-c"]

clean: shweb::clean
build: shweb::build

fmt:
  cargo fmt --all --check

check:
  cargo check --workspace

clippy +args:
  cargo clippy --workspace -- -W clippy::nursery {{args}}

