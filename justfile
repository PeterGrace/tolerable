commit := `git rev-parse HEAD`
transport := "docker://"
registry := "docker.io"
image := "petergrace/tolerable"


build:
  cross build --release --target aarch64-unknown-linux-gnu
  cross build --release --target x86_64-unknown-linux-gnu
make-image:
  docker buildx build --no-cache --push --platform linux/amd64,linux/arm64/v8 -t {{registry}}/{{image}}:latest .

release-patch:
  cargo release --no-publish --no-verify patch --execute
release-minor:
  cargo release --no-publish --no-verify minor --execute
release-major:
  cargo release --no-publish --no-verify minor --execute
