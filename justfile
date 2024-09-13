build:
	cargo build

schema:
  #!/usr/bin/env sh
  for dir in contracts/*; do
    if [ -d "$dir" ]; then
      echo "Generating schema for $dir"
      (cd "$dir" && cargo schema)
    fi
  done

optimize: build
  #!/usr/bin/env sh
  ./optimize.sh
  if [[ $(uname -m) =~ "arm64" ]]; then
    for file in ./artifacts/*-aarch64.wasm; do
      if [ -f "$file" ]; then
        new_name="${file%-aarch64.wasm}.wasm"
        mv "$file" "./$new_name"
      fi
    done
  fi
